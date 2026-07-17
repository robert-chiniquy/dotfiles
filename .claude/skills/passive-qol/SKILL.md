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

## Consistency across the whole computer is a QoL axis

Divergent behavior across contexts is friction. Every time the same
intent produces a different result — depending on which shell, which
terminal, which editor, which LLM harness, which app, which login
context — the user pays a cognitive tax. Treat inconsistency itself
as a QoL problem worth removing, not just an aesthetic concern.

Dimensions where consistency counts:
* **Aesthetic** — same palette, cursor, selection color, prompt accent,
  wallpaper aesthetic across terminal / OS / editor / statusline / dock
* **Keyboard** — same keybindings across terminals (iTerm, Ghostty,
  Alacritty), across editors, across REPLs; the same modifier does the
  same thing everywhere
* **Fonts** — same font family + size + feature settings across every
  surface that renders text
* **Shell** — same env vars, same PATH ordering, same completion
  behavior, same history semantics across every shell instance
  (interactive, non-interactive, login, subprocess)
* **LLM harnesses** — same MCP servers, same skills, same permissions,
  same slash commands, same statusline, same model choices across
  Claude Code / opencode / codex / pi.dev / anything else
* **Tool config** — same rc file honored in every context git/rg/bat/
  eza/etc. is invoked (interactive vs agent-driven vs cron)
* **File layout** — same paths for logs, caches, configs, scratchpads;
  no divergent locations for the same class of thing
* **Behavior** — same command produces the same result whether invoked
  by hand, by an agent, by a script, or by a launchd job

Prefer QoL changes that:
* Sync a config that's already correct in one place into the others
* Collapse two divergent settings into one shared source of truth
* Extend a passive rule (color, hook, prompt segment, gc daemon) so
  it fires everywhere the class of thing occurs

When suggesting an op that only lands in ONE context, note the parity
gap in the suggestion and, if a matching setting exists in the sibling
contexts, offer to apply it there too.

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

## Verify config paths before editing

Config files in dotfiles-style setups are often symlinked from an unusual
location. Before editing what looks like a config file, resolve the symlink
that the tool actually reads:

```bash
readlink ~/.config/<tool>.toml   # or wherever the tool is documented to read
```

If the file you're editing isn't the resolved target, edits go into a
stale sibling and the tool never sees them — leading to hours of
"why isn't my change taking effect".
