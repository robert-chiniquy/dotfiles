# Passive QoL Improvements

This skill enables Claude to proactively surface quality-of-life improvements for the user's computing environment.

## When to Apply

Proactively suggest improvements when:
- Working in dotfiles, shell config, or system configuration
- The user mentions friction, annoyance, or inefficiency
- You notice something could be better during normal work
- A session has natural pauses or transitions

**Before suggesting any shell changes:** Always read ~/.zshrc first to:
1. Avoid conflicts (e.g., don't define `rm()` if `alias rm` exists)
2. Avoid suggesting things already implemented
3. Understand the user's existing patterns and preferences

## Constraints

Suggestions MUST be:
- **Passive** - No new keystrokes to learn, no new commands to remember, no aliases
- **Automatic** - Works without user intervention once set up
- **Invisible** - Doesn't add UI clutter or notifications unless critical
- **One-time setup** - Configure once, benefit forever

Suggestions MUST NOT:
- Require learning new workflows
- Add shortcuts, aliases, or keybindings
- Require ongoing maintenance
- Add visible widgets, menu bar items, or notifications
- Be productivity theater (timers, trackers, GTD systems)
- Be corporate/enterprise tooling
- Be niche single-purpose apps
- Use significant CPU or I/O (no polling loops, heavy background processes, or frequent disk writes)

## Aesthetic Compatibility

The user has a vaporwave aesthetic. Suggestions should:
- Use the color palette: hot pink (#ff0099), cyan (#5cecff), magenta (#ff00f8), gold (#fbb725), purple (#aa00e8)
- Prefer dark backgrounds
- Avoid cutesy or corporate visual design
- Respect minimalism - no unnecessary visual elements

## User's Existing Setup

Reference these before suggesting:
- **Window management**: yabai + skhd (vim-style, bsp tiling, focus follows mouse)
- **Shell**: zsh with starship prompt, vaporwave colors throughout
- **Status bar**: sketchybar with occult/esoteric items
- **Widgets**: Ubersicht (I Ching, grimoire, pomodoro)
- **Automation**: Hammerspoon, Karabiner-Elements
- **Terminal**: iTerm2, Ghostty

## Good Suggestion Examples

- macOS defaults that improve behavior (like `defaults write ...`)
- System settings that reduce friction
- Automatic cleanup scripts (run via launchd, no user action)
- Performance tweaks
- Better default behaviors for apps already in use
- Privacy/security hardening that doesn't add friction
- Faster animations or no animations
- Auto-dark mode, auto-night shift
- Disk cleanup automation
- Git config improvements
- Shell performance optimizations

## Bad Suggestion Examples

- New apps to install (unless replacing something worse)
- Keyboard shortcuts
- Aliases
- Menu bar items
- Notification systems
- Productivity apps
- Time tracking
- Todo lists
- Any "system" that requires buy-in

## Format

When suggesting, be brief:
```
Passive QoL: [one-line description]
[single command or short explanation]
```

Only suggest one thing at a time. Don't overwhelm.

## Documentation

Every QoL change applied MUST be documented in `~/.claude/QOL.md`. Format:

```markdown
## YYYY-MM-DD: [description]
[command or change]
[brief explanation of benefit]
```

This creates a record of all passive improvements for reproducibility on new machines.

## Rejections

When a suggestion is declined, add it to the `# Rejects` section in `~/.claude/QOL.md`. Format:

```markdown
- [description] - [reason if given]
```

**NEVER suggest anything in the Rejects list.** Always read QOL.md before making suggestions.
