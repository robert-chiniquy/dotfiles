# scorecard

Render a dark, one-screen readiness scorecard TUI from a structured markdown
file. Truecolor ANSI, adapts to terminal width, one line per criterion, fits to
terminal height. No external dependencies — pure `std`.

```
scorecard [--width N] [--height N] [--mode fit|all] [--no-actions] <file.md>
scorecard --action scorecard://remove/<id>?file=<path>
scorecard install-handler | uninstall-handler        # macOS URL-scheme handler
scorecard prime                                       # agent-facing primer
```

`--width`/`--height` fall back to `$COLUMNS`/`$LINES`. Width is clamped 84–170.
Pass `--width "$COLUMNS" --height "$LINES"` from a prompt hook so it tracks the
live terminal.

## Build & install

```sh
cargo install --path . --root ~/.local     # installs to ~/.local/bin/scorecard
scorecard install-handler                    # register scorecard:// (macOS, once)
```

## Modes

- **fit** (default) — drop whole content-groups until the card fits in
  `height − 3` lines (the 3 leaves room for a shell prompt). Groups are dropped
  **lowest priority first**, ties **bottom-most first**. By default *chrome*
  (`header`, `titles`, `meter`, `tiles`, `callouts`, `footer`) is shed before any
  line item, so the list of items survives longest.
- **all** — render everything. `--mode all`.

If height is unknown (no `--height`, no `$LINES`), fit falls back to `all`.

## Markdown schema

Front matter — `key: value` lines before the first `##`:

| key | meaning |
|-----|---------|
| `title` | headline (an `# H1` works too) |
| `sub` | one-line subtitle |
| `meta` | context line (deadline, pass bar, weights) |
| `score` | `projected/max` (e.g. `155/215`) — drives the meter |
| `pass` | threshold; defaults to 70% of max |
| `note` | short line appended to the tiles row |
| `footer` | dim footer (a trailing `> quote` line also works) |
| `groups` | `name=priority, …` — group priorities (higher = kept longer) |

Sections are `## Label (xN)` followed by a GFM table. **Criteria** rows have five
cells (header/`---` rows are ignored):

```
| id | state | score | criterion | note |
| M1 | risk  | 3 | Ship the build | rides [T-1](https://…) | grp:topic |
```

`state` → color/pill: `solid`/`ok`/`done` = green **SOLID** · `risk` (or
anything else) = gold **AT RISK** · `gap`/`crit`/`blocked`/`fail` = red **GAP**.
The tiles row (solid / at-risk / gap / at-zero counts) is computed automatically.

A `## Callouts` (or `Notes`/`Banners`) section holds two-cell rows rendered as
labelled banners; the tag colors the label (`STANDOUT`/`RISK` pink,
`DECIDE`/`BLOCK`/`GAP` red, else cyan):

```
## Callouts
| STANDOUT | the single most important thing |
| NEXT | the next actions |
```

## Content-groups

A line can belong to **many** groups.

- **Built-in "type" groups** are auto-assigned: `header` (title/sub/meta),
  `titles` (section headers), `meter`, `tiles`, `callouts` (banners), `footer`.
- **Topic groups** are explicit: add one or more `grp:<name>` cells to a row
  (`… | grp:cursor | grp:monday |`). The `grp:` cells are stripped from the note.
- **Priorities** live in the front-matter `groups:` line and cover both kinds
  (`groups: header=100, callouts=5, cursor=8`). Default 0 for chrome, 10 for line
  items, so items outrank chrome; set explicit values to override. Negatives drop
  first.

Removing any row in a topic group removes every row sharing it (see below).

## Hyperlinks

`[text](url)` in the id / note / callout fields renders as an OSC 8 terminal
hyperlink — clickable in iTerm2, WezTerm, kitty, Windows Terminal, and VTE.
Terminals without OSC 8 (e.g. macOS Terminal.app) show plain text; inside tmux it
needs a recent tmux with `allow-passthrough`. Link width is measured by the
visible text, so alignment is unaffected.

## Actions & the close box

When a source file is known, each criterion row gets a clickable `✕` linking to
`scorecard://remove/<id>?file=<path>` — the **id (first column) is the anchor**.
Clicking removes that row, plus every row sharing any of its topic groups. The
change shows on the next render (no live redraw). Suppress with `--no-actions`.

The `scorecard://` scheme is handled by the binary itself:

```sh
scorecard install-handler     # register scorecard:// -> scorecard --action …
scorecard uninstall-handler
```

`install-handler` builds a small AppleScript app in `~/Applications` that
forwards scheme opens to this binary; `--action` runs the action and posts a
macOS notification.

## prime

`scorecard prime` prints an agent-facing primer — what the tool is, the schema,
groups, modes, actions, and the write/preserve convention. It's the canonical
"how to use this" text; point agents at it.

## Teaching agents

`scorecard install-agents` points every installed agent harness at `scorecard
prime`. It writes an idempotent, marker-delimited block into each harness's
global instructions — Claude Code, Codex, Cursor, pi, opencode, Goose — telling
it to run `scorecard prime` when asked to summarize a tactical
code/PR/incident/deadline/milestone situation. Only harnesses that actually
exist are touched; `uninstall-agents` removes the blocks. (Claude Code also has a
skill that auto-triggers and defers to `scorecard prime`.)

## New-terminal greeting

`.zshrc` renders `$SCORECARD_FILE` (default `~/.config/scorecard/status.md`) on a
new interactive shell, if present, passing `--width "$COLUMNS" --height "$LINES"`
so it fits the window. Disable with `SCORECARD_GREETING=0`; point elsewhere with
`SCORECARD_FILE=…`; edit/delete the status file to change what shows.

By convention, preserve the prior scorecard before writing a new one:

```sh
mv ~/.config/scorecard/status.md ~/.config/scorecard/status-$(date +%F).md
```
