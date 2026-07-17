---
name: readiness-scorecard
description: Write a scorecard — a dark, skimmable one-screen readout of the current situation in the user's work, rendered as a terminal (TUI) artifact by the `scorecard` binary from a structured markdown file. Use when the user asks for a scorecard, a status/situation readout, "where do things stand", a readiness read, a go/no-go, or a coverage snapshot. NOT a web page — output is the dark TUI. Preserves older scorecards by datestamp. Triggers on: scorecard, status readout, situation readout, where things stand, readiness, go/no-go, coverage snapshot, skimmable status.
---

# Scorecard — run `scorecard prime`

The canonical guide lives in the binary. Run:

```sh
scorecard prime
```

and follow it. It covers the markdown schema, content-groups (topic + built-in
type groups), fit/all modes, the remove / close-box / install-handler actions,
and the compose-well + preserve-with-datestamp conventions. This skill is just
the trigger; `prime` is the source of truth (so it can't drift).

The short version: write the scorecard as markdown to
`~/.config/scorecard/status.md`, **preserving the prior one first**:

```sh
mv ~/.config/scorecard/status.md ~/.config/scorecard/status-$(date +%F).md   # if it exists
```

then render:

```sh
scorecard --width "$COLUMNS" --height "$LINES" ~/.config/scorecard/status.md
```

If the binary is missing: `cargo install --path ~/repo/dotfiles/scorecard --root ~/.local`.
Output is the dark terminal TUI — never a web page.
