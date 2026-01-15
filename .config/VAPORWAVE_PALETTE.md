# Vaporwave Color Palette

Canonical color definitions for the vaporwave theme across all tools.

## Core Palette

| Name | Hex | RGB | Usage |
|------|-----|-----|-------|
| **Background** | `#000000` | 0, 0, 0 | Terminal/editor background |
| **Foreground** | `#f0f0f0` | 240, 240, 240 | Default text (bright, readable) |
| **Selection BG** | `#4e4e8f` | 78, 78, 143 | Selected text background |
| **Selection FG** | `#eeeeee` | 238, 238, 238 | Selected text foreground |

## Accent Colors

| Name | Hex | RGB | Usage |
|------|-----|-----|-------|
| **Hot Magenta** | `#ff00f8` | 255, 0, 248 | Primary accent, keywords, cursor |
| **Hot Pink** | `#ff0099` | 255, 0, 153 | Headings, constants, errors |
| **Electric Cyan** | `#5cecff` | 92, 236, 255 | Functions, links, info |
| **Gold** | `#fbb725` | 251, 183, 37 | Strings, warnings, paths |
| **Light Pink** | `#ffb1fe` | 255, 177, 254 | Classes, types, emphasis |
| **Deep Purple** | `#aa00e8` | 170, 0, 232 | Secondary accent, line numbers |
| **Neon Purple** | `#ab60ed` | 171, 96, 237 | Regex, special |
| **Soft Purple** | `#c080d0` | 192, 128, 208 | Punctuation, ANSI magenta |

## ANSI 16-Color Palette

| # | Name | Hex | Notes |
|---|------|-----|-------|
| 0 | Black | `#030c33` | Dark blue-black |
| 1 | Red | `#ff6b9d` | Soft pink-red |
| 2 | Green | `#58e8a3` | Mint green |
| 3 | Yellow | `#fff9d9` | Cream yellow |
| 4 | Blue | `#7b91e0` | Periwinkle |
| 5 | Magenta | `#c080d0` | Soft purple |
| 6 | Cyan | `#bafffd` | Light cyan |
| 7 | White | `#e0e0e0` | Light grey (UPDATED from #cfbfad) |
| 8 | Bright Black | `#4e4e8f` | Purple-grey |
| 9 | Bright Red | `#ff8bff` | Bright pink |
| 10 | Bright Green | `#00ff8b` | Neon green |
| 11 | Bright Yellow | `#ffcd8b` | Peach |
| 12 | Bright Blue | `#808bed` | Light purple-blue |
| 13 | Bright Magenta | `#ab60ed` | Neon purple |
| 14 | Bright Cyan | `#8b8bff` | Lavender |
| 15 | Bright White | `#ffffff` | Pure white |

## Config File Locations

| Tool | Config Path |
|------|-------------|
| Ghostty | `~/.config/ghostty/config` |
| Kitty | `~/.config/kitty/kitty.conf` |
| iTerm2 | `~/Library/Application Support/iTerm2/DynamicProfiles/vaporwave.json` |
| VSCode | `~/.vscode/extensions/vaporwave-theme/themes/vaporwave-color-theme.json` |
| Cursor | `~/.cursor/extensions/vaporwave-theme/themes/vaporwave-color-theme.json` |
| bat | `~/.config/bat/themes/vaporwave.tmTheme` |
| glow | `~/.config/glow/vaporwave.json` |
| yazi | `~/.config/yazi/theme.toml` |
| starship | `~/.config/starship.toml` |
| git delta | `~/.gitconfig` (delta section) |
| ripgrep | `~/.ripgreprc` |
| fzf/zsh | `~/.zshrc` (inline color definitions) |

## Changes from Original

- **Foreground**: `#fefeef` -> `#f0f0f0` (brighter, more neutral)
- **ANSI White (7)**: `#cfbfad` -> `#e0e0e0` (much brighter, was too tan/dim)
