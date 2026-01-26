# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Purpose

Personal dotfiles for macOS/zsh environment. Symlinked to `$HOME` via `install.sh`.

## Installation

```bash
./install.sh
```

Creates symlinks from home directory to this repo:
- `~/.zshrc` -> `.zshrc`
- `~/.zprofile` -> `.zprofile`
- `~/.vim` -> `.vim`
- `~/.vimrc` -> `.vimrc`
- `~/.claude` -> `.claude` (Claude Code global config)
- `~/.config/starship.toml` -> `starship.toml`
- `~/.bash_login`, `~/.inputrc`

## Architecture

### Shell Configuration (zsh)

**`.zprofile`** - Login shell setup (PATH, environment variables). Sources once per login.

**`.zshrc`** - Interactive shell config. Key sections:
- Modern CLI tool setup (eza, bat, ripgrep, zoxide, fzf, atuin)
- Vaporwave color theme throughout (`#ff00f8` hot pink, `#5cecff` cyan, `#fbb725` gold)
- Completion system with smart TAB behavior
- Custom widgets: markdown browser (double-Esc), fuzzy history (Ctrl+P)
- Auto-ls on cd, command duration display, iTerm2 badge updates

**Key keybindings:**
- `Esc Esc` - Markdown file browser with fzf
- `Ctrl+P` - Fuzzy history search
- `Ctrl+T` - Fuzzy file search
- `Alt+C` - Fuzzy cd

### Claude Code Configuration (`.claude/`)

**`.claude/CLAUDE.md`** - Global instructions for Claude Code (applies to all projects).

**`.claude/skills/`** - Reusable skill definitions organized by category:
- `default/` - Always-applied skills
- `design/` - Feature design methodologies
- `engineering/` - Architectural patterns
- `meta/` - Process skills

### Git Configuration

Uses `delta` as pager with vaporwave colors. Key settings in `.gitconfig`:
- `push.autoSetupRemote = true`
- Side-by-side disabled, line numbers enabled
- Diff colors coordinated with shell theme

### Starship Prompt

Right-aligned git status/branch. Uses butterfly/squid symbols for success/error.

## Theme Consistency

All tools share the vaporwave palette:
- Hot pink: `#ff00f8`
- Cyan: `#5cecff`
- Gold: `#fbb725`
- Purple: `#aa00e8`
- Light pink: `#ffb1fe`

When modifying configs, maintain color consistency.

## Security

**Never commit secrets.** Before any git commit in this repo, verify that no tokens, API keys, passwords, or other secrets are being staged. This includes:
- `.claude/mcp.json` - may contain literal token values
- Any config file that could have credentials embedded
- Environment variable values that were expanded into files

If secrets are found in staged changes, abort the commit and help the user remove them.

## Dependencies

Modern CLI tools (install via homebrew):
- eza, bat, ripgrep, fd, fzf, zoxide, atuin, delta, starship, glow
