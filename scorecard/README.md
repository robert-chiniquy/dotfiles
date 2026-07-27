# scorecard

Render a dark, one-screen readiness scorecard TUI from a structured markdown
file. Truecolor ANSI, adapts to terminal width, one line per criterion, fits to
terminal height, and can turn typed data tables into terminal charts. The Rust
binary and chart renderer have no crate dependencies.

```
scorecard [--width N] [--height N] [--mode fit|all]
          [--charts auto|image|text|off] [--no-actions] [--no-refresh] <file.md>
scorecard demo [file]                                 # feature tour above current card
scorecard --action scorecard://remove/<id>?file=<path>
scorecard install-handler | uninstall-handler        # macOS URL-scheme handler
scorecard prime [--srs]                               # agent-facing primer (full text needs --srs)
scorecard list [file]                                 # remaining items as markdown
scorecard refresh [--quiet] [file]                    # prune terminal tracker rows
scorecard install-agents | uninstall-agents           # point harnesses at prime --srs
```

`--width`/`--height` fall back to `$COLUMNS`/`$LINES`. Width is clamped 84–170.
Pass `--width "$COLUMNS" --height "$LINES"` from a prompt hook so it tracks the
live terminal.

## Demo

`scorecard demo [file]` renders the built-in, feature-complete demo first, then
the selected scorecard. With no file argument, it uses `$SCORECARD_FILE` or
`~/.config/scorecard/status.md`; if that default does not exist, the demo still
renders by itself. An explicitly named missing file is an error.

The demo always renders in `all` mode so every severity, link, content group,
callout color, and chart type remains visible. Width and chart options apply to
both cards. The selected scorecard keeps the requested fit mode, clickable
actions, and passive tracker refresh. The built-in demo has no backing file, so
it has no close boxes and never runs tracker lookups.

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

## Data charts

A dedicated `## Chart: title` section is parsed into a typed chart model. The
Markdown table is only its serialization: the first column supplies x/category
labels and every remaining numeric column becomes a named series.

```markdown
## Chart: Reveal latency
type: time-series
| day | p50 | p95 |
| --- | ---: | ---: |
| Mon | 10 | 20 |
| Tue | 15 | 24 |
| Wed | 12 | 31 |
```

`type:` accepts `sparkline` (default), `histogram`, or `time-series`.
Comma-separated numbers, percentages, and parenthesized negatives are accepted;
`n/a`, `null`, `-`, and malformed cells remain missing data rather than becoming
zero. A one-column histogram table is treated as raw observations and binned
automatically; a label-plus-value histogram uses the supplied bins/categories.

In `auto` mode, an interactive iTerm2 session outside tmux gets a
dependency-free generated PNG through iTerm2's
[OSC 1337 inline-image protocol](https://iterm2.com/documentation-images.html).
Redirected output, other terminals, and tmux get a compact Unicode rendering.
The protocol can also display agent-supplied animated GIFs; generated charts
stay static PNGs so startup output is deterministic and bounded.
Use `--charts image` to force inline images when the downstream terminal handles
the protocol, `--charts text` for deterministic text, or `--charts off`.
`SCORECARD_CHARTS` sets the default. Fit mode bounds charts to at most one third
of the usable pane.

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

## Passive tracker refresh

Criterion rows containing Markdown links to `github.com/.../pull/...` or
`linear.app/.../issue/...` are checked passively. A normal render never waits on
the network: when the source changed or the hourly cache expired, it launches a
quiet background `scorecard refresh`. The updated rows disappear on the next
render.

- GitHub uses one batched `gh api graphql` request per 50 links. Authenticate
  with `gh auth login`, or set `GH_TOKEN`/`GITHUB_TOKEN`.
- Linear uses batched GraphQL through `curl` and parses the response with `jq`.
  Set `LINEAR_API_KEY`; OAuth callers may set `LINEAR_ACCESS_TOKEN`.
- GitHub `CLOSED`/`MERGED` and Linear `completed`/`canceled` are terminal.
- A row with several recognized tracker links is removed only when all are
  conclusively terminal. Missing tools, credentials, access, or malformed
  responses retain the row.

Removal is structural: scorecard reparses the live document, applies provider
states to typed criterion/link nodes, and serializes the remaining rows. Note or
whitespace edits do not act as identity, and automatic pruning never expands
through `grp:` siblings. A concurrent source change aborts the write.

Run `scorecard refresh [file]` for a visible result. Disable scheduling with
`--no-refresh`, `SCORECARD_AUTO_REFRESH=0`, or
`SCORECARD_REFRESH_INTERVAL=0`; otherwise the interval variable is in seconds
and defaults to 3600.

## prime

Bare `scorecard prime` prints a short nudge (this is for the shell startup
scorecard banner — only dump the full primer if you mean it). Full primer:

```sh
scorecard prime --srs      # also --srrs, --srrrs, … any number of r's
```

That text is the canonical "how to use this" guide: schema, charts, groups,
modes, actions, refresh, and the write/preserve convention. Point agents at
`prime --srs`.

## Listing remaining items

`scorecard list [file]` prints the current line items (default
`~/.config/scorecard/status.md`) as a markdown list — id, name, state, groups.
Because items can be removed (via the ✕), an agent tracking work it wrote should
re-run this periodically and diff against what it wrote. `prime` tells agents to
do exactly that, and to keep items forward-looking (no finished/past-tense
entries) and anchored to a ticket/repo/system when there's room.

## Teaching agents

`scorecard install-agents` points every installed agent harness at `scorecard
prime --srs`. It writes an idempotent, marker-delimited block into each harness's
global instructions — Claude Code, Codex, Cursor, pi, opencode, Goose — telling
it to run `scorecard prime --srs` when asked to summarize a tactical
code/PR/incident/deadline/milestone situation. Only harnesses that actually
exist are touched; `uninstall-agents` removes the blocks. (Claude Code also has a
skill that auto-triggers and defers to `scorecard prime --srs`.)

## New-terminal greeting

`.zshrc` renders `$SCORECARD_FILE` (default `~/.config/scorecard/status.md`) on a
new interactive shell, if present, passing `--width "$COLUMNS" --height "$LINES"`
so it fits the window. Rendering stays local and immediate; any tracker lookup
runs in the detached refresh process. Disable with `SCORECARD_GREETING=0`; point
elsewhere with `SCORECARD_FILE=…`; edit/delete the status file to change what
shows.

By convention, preserve the prior scorecard before writing a new one:

```sh
mv ~/.config/scorecard/status.md ~/.config/scorecard/status-$(date +%F).md
```
