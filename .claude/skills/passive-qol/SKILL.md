---
name: passive-qol
description: |
  Proactive quality-of-life for the computing environment. Activates when
  working in dotfiles, shell config, system settings, or when the user
  mentions friction or inefficiency. Suggests only passive, automatic changes.
---

# Passive QoL

Surface improvements that require zero ongoing effort from the user.

## Constraints

Suggestions MUST be:
* Passive — no new keystrokes, commands, or aliases
* Automatic — works without intervention once configured
* Invisible — no UI clutter unless critical
* One-time — configure once, benefit forever

Suggestions MUST NOT be:
* New apps (unless replacing something broken)
* Shortcuts, aliases, or keybindings
* Widgets, menu bar items, or notifications
* Productivity theater (timers, trackers, GTD)
* CPU/IO heavy (no polling loops, frequent disk writes)

## Before Suggesting

Read `~/.claude/QOL.md` first. Never suggest anything in the Rejects list.
Read `~/.zshrc` to avoid conflicts with existing setup.

## Format

One suggestion at a time:
```
Passive QoL: [one-line description]
[single command or short explanation]
```

## Documentation

Every change applied MUST be logged in `~/.claude/QOL.md`:
```markdown
## YYYY-MM-DD: [description]
[command or change]
[brief benefit]
```

Declined suggestions go in the `# Rejects` section.
