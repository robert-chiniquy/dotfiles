---
name: readiness-scorecard
description: Write a scorecard — a dark, skimmable one-screen readout of the current situation in the user's work, rendered as a terminal (TUI) artifact by the `scorecard` binary from a structured markdown file. Use when the user asks for a scorecard, a status/situation readout, "where do things stand", a readiness read, a go/no-go, or a coverage snapshot. NOT a web page — output is the dark TUI. Preserves older scorecards by datestamp. Triggers on: scorecard, status readout, situation readout, where things stand, readiness, go/no-go, coverage snapshot, skimmable status.
---

# Scorecard (dark TUI readout)

A scorecard is a **dark, skimmable, one-screen readout of the current situation**
in the user's work — criteria/gates with a state, a projected/target number, and
a one-line "why" each. It is rendered in the **terminal** by the `scorecard`
binary from a structured markdown file. It is **not** a web page — the user
prefers the TUI.

The binary owns all rendering (dark ground, vaporwave accents, green/gold/red
severity, meter, box, adaptive width). This skill's job is to (1) assemble a
skimmable situation, (2) write well-formed scorecard markdown, (3) **preserve the
previous scorecard**, and (4) render it.

## Pipeline

1. **Assemble** the situation from whatever's relevant (the conversation,
   trackers, git, docs), then **scope and group** it (see below). Keep every
   note to one skimmable line.
2. **Preserve, then write** (see the datestamp rule below).
3. **Render** to show the user:
   ```sh
   scorecard --width "$COLUMNS" ~/.config/scorecard/status.md
   ```
   If the binary is missing:
   `cargo install --path ~/repo/dotfiles/scorecard --root ~/.local`

## Scope & grouping

- **Only the user's surface.** Include only what depends on the user or centers
  on their own work — what they own, drive, are blocked on, or must decide. Drop
  work that sits elsewhere and doesn't gate the user; a shared program's full
  backlog is not the scorecard. Scoping narrows the readout, it does not soften
  it — be just as honest inside the narrowed set. (Describe items by what they
  are, never by whose they are — no names.)
- **Group by theme, not by ceremony.** Make the `##` sections the *themes* of the
  work — the coherent clusters the user actually thinks in — so related items sit
  together and the shape of the situation reads at a glance. Reach for weight
  tiers like "Must / Should" only when the underlying rubric is genuinely
  weighted (e.g. a scored POC); otherwise theme sections beat generic tiers.
- Keep sections few and named for what they are. A theme with a single item is a
  smell — fold it in or rename the theme.

## Files & the datestamp-preserve rule

- **Live file** = `~/.config/scorecard/status.md` — what the new-terminal-window
  greeting renders (wired in `.zshrc`).
- **Archives** = `~/.config/scorecard/status-<YYYY-MM-DD>.md`.

**Never overwrite an existing scorecard.** Before writing a new one, move the
current live file aside with a datestamp suffix, then write the new content to
`status.md`. This follows the standing "versioning, not overwriting" rule.

```sh
f=~/.config/scorecard/status.md
if [ -e "$f" ]; then
  a=~/.config/scorecard/status-$(date +%F).md
  [ -e "$a" ] && a=~/.config/scorecard/status-$(date +%F)-$(date +%H%M).md
  mv "$f" "$a"        # preserve the outgoing scorecard
fi
# now write the new scorecard to $f, then render it
```

Archives accumulate as a dated history; the greeting always reads `status.md`.

## Markdown schema (summary)

Full spec + a complete example: `~/repo/dotfiles/scorecard/README.md` and
`~/repo/dotfiles/scorecard/examples/sample.md`. In brief:

Front matter — `key: value` lines before the first `##`:

- `title` — headline (an `# H1` also works)
- `sub` — one-line subtitle
- `meta` — context (deadline, pass bar, weights)
- `score` — `projected/max` (e.g. `155/215`); drives the meter
- `pass` — threshold (defaults to 70% of max)
- `note` — short line appended to the auto-computed tiles row
- `footer` — dim footer (a trailing `> quote` line also works)

Sections — `## Label (xN)` then a GFM table. **Criteria** rows have five cells;
the header/`---` rows are ignored:

```
| id | state | score | criterion | note |
| M1 | solid | 5 | Secrets never in LLM context | broker resolves at the process layer |
```

`state` → color/pill: `solid`/`ok`/`done` = green SOLID · `risk`/`at-risk`/
anything else = gold AT RISK · `gap`/`crit`/`blocked`/`fail` = red GAP. The tiles
row (solid / at-risk / gap / at-zero counts) is computed automatically. Keep the
`id` column **unique** — it's also the anchor the row's ✕ close box removes by.

A `## Callouts` (or `Notes`/`Banners`) section holds two-cell rows rendered as
labelled banners — put the single most important thing, the next actions, and any
decision here:

```
## Callouts
| STANDOUT | the one thing that matters most right now |
| NEXT | the next actions |
```

Banner color keys off the tag: `STANDOUT`/`RISK` → pink, `DECIDE`/`BLOCK`/`GAP`
→ red, else cyan.

**Links.** `[text](url)` in the id, note, and callout fields becomes a clickable
terminal hyperlink (OSC 8) — use it for ticket ids, PRs, and docs. Width is
measured by the visible text; unsupported terminals just show the text.

## Content rules

- **Succinct, weighted by importance.** Trim every note to the fewest words that
  carry it, and let the screen-space a statement takes be proportional to how
  much it matters: a low-risk solid might be three words, the one real gap earns
  its whole line, and the STANDOUT callout gets the most room. Uniform-length
  notes flatten the signal — the eye should land on what matters *because* it
  occupies more space. Lead with the fact, not prose.
- **Honest severities.** Don't inflate a gap to at-risk or an at-risk to solid —
  the value is a truthful glance.
- **Dry, terse, no emoji.** Factual, low-drama. Real content only, never lorem.
- **No names / PII / proprietary ids** in anything that could be shared;
  anonymize to roles unless it's a private personal scorecard the user directs
  otherwise.
- Label any projected number as directional if it's an estimate; timestamp the
  footer when the data came from a live query.
