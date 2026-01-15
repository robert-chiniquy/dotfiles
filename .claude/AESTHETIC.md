# Aesthetic Guidelines

## Core Principles

**Information density over decoration.** Every pixel earns its place. No padding for padding's sake. No rounded corners. No drop shadows. No gradients unless functional.

**Data over text.** Prefer symbols, glyphs, numbers, sparklines, progress bars. When text is necessary, abbreviate. `3m` not `3 minutes ago`. `~/r/p` not `/Users/rch/repo/project`.

**No branding.** No product names, no logos, no "Powered by", no attributions in UI. The tool is invisible; only the work matters.

**Black background, always.** Pure `#000000`. Not "dark grey", not "almost black". Black.

**Color is signal.** Every color means something. Don't use color for decoration.

## Color Palette

| Role | Hex | When to use |
|------|-----|-------------|
| Background | `#000000` | Always |
| Foreground | `#f0f0f0` | Default text |
| Primary accent | `#ff00f8` | Active/selected/important, cursor |
| Secondary accent | `#5cecff` | Links, functions, info |
| Tertiary accent | `#fbb725` | Warnings, strings, paths |
| Muted | `#aa00e8` | Inactive, line numbers, secondary |
| Success | `#58e8a3` | Added, passing, good |
| Error | `#ff6b9d` | Removed, failing, bad |
| Selection | `#4e4e8f` | Highlighted regions |

## Typography

- Monospace only
- No ligatures (code should look like what it is)
- 18px baseline, scale down for density not up for "readability"
- No bold unless semantic (headings, keywords)
- No italic unless semantic (comments, emphasis)

## UI Elements

**Status bars**: Single line. Top preferred (eye moves down to content). Show only: mode, location, key metrics. No clock unless relevant.

**Borders**: 1px solid accent color or none. No double borders. No rounded corners.

**Icons**: Unicode glyphs over icon fonts. Nerd font symbols acceptable. No emoji in UI (acceptable in content).

**Whitespace**: Minimal margins. Dense packing. Let color and structure separate elements, not empty space.

**Animation**: None, or purely functional (loading states). No transitions. No easing. Instant feedback.

## Status Indicators

Prefer symbols over words:

| Meaning | Symbol |
|---------|--------|
| Success/yes | `+` or `*` |
| Failure/no | `-` or `x` |
| Warning | `!` |
| Info | `>` |
| Active | `*` |
| Pending | `.` |
| Modified | `~` |
| Added | `+` |
| Deleted | `-` |

## Examples

**Good status line:**
```
NORMAL | main~3 | src/app.go:142 | 3E 1W
```

**Bad status line:**
```
-- NORMAL MODE --  |  Branch: main (3 files modified)  |  File: src/app.go  |  Line 142, Column 8  |  3 Errors, 1 Warning  |  UTF-8  |  LF  |  Go
```

**Good path:**
```
~/r/agents/cmd
```

**Bad path:**
```
/Users/username/repositories/agents/cmd/
```

## Anti-patterns

- Progress bars with percentages AND time remaining AND file counts
- "Loading..." text (just show a spinner or nothing)
- Confirmation dialogs for non-destructive actions
- Tooltips that repeat what's already visible
- Status messages that fade out (either persist or don't show)
- "Welcome to [Product Name]!" messages
- Version numbers in UI (put in --version)
- Hamburger menus, dropdowns, or any hidden navigation
- Light themes
- White or light backgrounds for UI chrome (status bars, tabs, etc.)
- High-contrast inverse video for mode indicators

## Tool Configuration Rules

When configuring tools, always:

1. **Disable branding** - Remove product names, logos, "powered by" from UI
2. **Use vaporwave palette** - No white/light backgrounds in chrome; use black bg with colored text
3. **Reject tools with hardcoded branding** - If branding can't be removed, don't use the tool
4. **Prefer terminal-native** - Tools that respect $TERM colors over tools with custom UI
5. **Focus follows mouse** - Panes, windows, splits should focus on hover, not click

Tools rejected for branding/aesthetic violations:
- zellij (hardcoded "Zellij" branding, white bg mode indicators)
