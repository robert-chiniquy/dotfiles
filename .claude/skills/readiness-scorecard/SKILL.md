---
name: readiness-scorecard
description: Write a scorecard — a dark, skimmable one-screen readout of the current situation in the user's work, rendered as a terminal (TUI) artifact by the `scorecard` binary from a structured markdown file. Use ONLY when the user explicitly asks for a scorecard (or a "readiness card" / "go/no-go card" / "coverage snapshot card"). A plain "status", "status?", or other status/situation question gets a prose answer, never this skill. NOT a web page — output is the dark TUI. Preserves older scorecards by datestamp. Triggers on: scorecard, readiness card, go/no-go card, coverage snapshot card.
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

Link every PR reference: `repo#123` must be `[repo#123](https://github.com/org/repo/pull/123)`
everywhere it appears — id cells, notes, and especially Callouts/prose (the usual
miss). Anchor text names the PR or action, never "link"/"here". A bare PR number can't be clicked, and the scorecard exists to be acted on.
