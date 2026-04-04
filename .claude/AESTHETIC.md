# Aesthetic Guidelines

## Core Principles

**Information density over decoration.** Every pixel earns its place. No padding for padding's sake. No rounded corners. No drop shadows. No gradients unless functional.

**Data over text.** Prefer symbols, glyphs, numbers, sparklines, progress bars. When text is necessary, abbreviate. `3m` not `3 minutes ago`. `~/r/p` not `/Users/rch/repo/project`.

**No branding.** No product names, no logos, no "Powered by", no attributions in UI. The tool is invisible; only the work matters.

**Black background, always.** Pure `#000000`. Not "dark grey", not "almost black". Black.

**Color is signal.** Every color means something. Don't use color for decoration.

## Color Palette

| Role | Hex | Swatch | When to use |
|------|-----|--------|-------------|
| Background | `#000000` | ![#000000](https://img.shields.io/badge/%23000000-%23000000?style=flat-square) | Always |
| Foreground | `#f0f0f0` | ![#f0f0f0](https://img.shields.io/badge/%23f0f0f0-%23f0f0f0?style=flat-square) | Default text |
| Primary accent | `#ff00f8` | ![#ff00f8](https://img.shields.io/badge/%23ff00f8-%23ff00f8?style=flat-square) | Active/selected/important, cursor |
| Secondary accent | `#5cecff` | ![#5cecff](https://img.shields.io/badge/%235cecff-%235cecff?style=flat-square) | Links, functions, info |
| Tertiary accent | `#fbb725` | ![#fbb725](https://img.shields.io/badge/%23fbb725-%23fbb725?style=flat-square) | Warnings, strings, paths |
| Muted | `#aa00e8` | ![#aa00e8](https://img.shields.io/badge/%23aa00e8-%23aa00e8?style=flat-square) | Inactive, line numbers, secondary |
| Success | `#58e8a3` | ![#58e8a3](https://img.shields.io/badge/%2358e8a3-%2358e8a3?style=flat-square) | Added, passing, good |
| Error | `#ff6b9d` | ![#ff6b9d](https://img.shields.io/badge/%23ff6b9d-%23ff6b9d?style=flat-square) | Removed, failing, bad |
| Selection | `#4e4e8f` | ![#4e4e8f](https://img.shields.io/badge/%234e4e8f-%234e4e8f?style=flat-square) | Highlighted regions |

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
