# scorecard

Render a dark, one-screen readiness scorecard TUI from a structured markdown
file. Truecolor ANSI, adapts to terminal width, one line per criterion. No
external dependencies — pure `std`.

```
scorecard [--width N] <file.md>
```

`--width` is optional; without it the tool reads `$COLUMNS`, then falls back to
120. Width is clamped to 84–170. Pass `--width "$COLUMNS"` from a prompt hook so
it tracks the live terminal size.

## Build & install

```sh
cargo build --release                      # ./target/release/scorecard
cargo install --path . --root ~/.local     # installs to ~/.local/bin/scorecard (on PATH)
```

## Markdown schema

Front matter is `key: value` lines before the first `##` header:

| key | meaning |
|-----|---------|
| `title` | headline (accent). An `# H1` works too. |
| `sub` | one-line subtitle |
| `meta` | context line (deadline, pass bar, weights) |
| `score` | `projected/max`, e.g. `148/175` — drives the meter |
| `pass` | pass threshold; defaults to 70% of max |
| `note` | short line appended to the auto-computed tiles row |
| `footer` | dim footer; a trailing `> quote` line also works |

Each `## Section (xN)` starts a group; `(xN)` is a cosmetic weight tag. Rows are
GFM tables. **Criteria** rows have five cells; the header/`---` rows are ignored:

```
| id | state | score | criterion | note |
| R1 | solid | 5 | Data migration is reversible | rollback under 5 min |
```

`state` maps to a severity color and pill:

- `solid` / `ok` / `green` / `done` → green **SOLID**
- anything else, or `risk` / `at-risk` / `warn` → gold **AT RISK**
- `gap` / `crit` / `blocked` / `fail` → red **GAP**

The tiles row (counts of solid / at-risk / gap / at-zero) is computed
automatically from the criteria.

A section named `Callouts` / `Notes` / `Banners` holds two-cell rows rendered as
labelled banners:

```
## Callouts
| STANDOUT | the single most important thing |
| NEXT | the next actions |
```

Banner color keys off the tag: `STANDOUT`/`RISK` → pink, `DECIDE`/`BLOCK`/`GAP`
→ red, otherwise cyan.

See [`examples/sample.md`](examples/sample.md) for a complete file.

## Hyperlinks

`[text](url)` in the **id**, **note**, and **callout** fields renders as an OSC 8
terminal hyperlink — clickable in iTerm2, WezTerm, kitty, Windows Terminal, and
VTE terminals. Terminals without OSC 8 (e.g. macOS Terminal.app) show the link
text as plain, un-clickable text; inside tmux it needs a recent tmux with
`allow-passthrough`. Link width is measured by the visible text, so box
alignment is unaffected.

## Actions & the close box

When a source file is known, each criterion row gets a clickable close box (`✕`)
linking to `scorecard://remove/<id>?file=<path>` — the **row id (first table
column) is the anchor**. Clicking removes that row from the markdown; the change
shows on the next render (there is no live redraw). Suppress the close boxes with
`--no-actions`.

The `scorecard://` scheme is handled by the binary itself — no separate script:

```sh
scorecard install-handler     # register scorecard:// -> scorecard --action …  (macOS)
scorecard uninstall-handler   # remove it
```

`install-handler` builds a small AppleScript app in `~/Applications` that
forwards scheme opens to this binary; `--action <url>` runs the action (currently
`remove`) and posts a macOS notification. Clicking works in OSC-8 terminals
(iTerm2 / WezTerm / kitty / …); inside tmux it needs `allow-passthrough`.

## New-terminal greeting

`.zshrc` renders `$SCORECARD_FILE` (default `~/.config/scorecard/status.md`) on
a new interactive shell, if that file exists. Disable with
`export SCORECARD_GREETING=0`. Point it elsewhere with `export SCORECARD_FILE=…`.
Edit the status file to change what shows; delete it to show nothing.
