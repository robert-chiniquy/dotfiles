### local dotfiles
`git clone https://github.com/robert-chiniquy/dotfiles.git && cd dotfiles && ./install.sh`

## Manual Restores

Some configs need manual restore (not symlinked by install.sh):

### iTerm2

Vaporwave color scheme with hot pink active tab.

```bash
cp ~/.config/iterm2/com.googlecode.iterm2.plist ~/Library/Preferences/
# Restart iTerm2
```

To re-export after changes:
```bash
cp ~/Library/Preferences/com.googlecode.iterm2.plist ~/.config/iterm2/
```

### Vaporwave Overlay

Metal shader overlay that renders vaporwave effects on unfocused windows. Detects purple content and reacts with glitchy scanlines.

```bash
# Install launch agent
cp ~/repo/dotfiles/LaunchAgents/com.rch.vaporwave-overlay.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.rch.vaporwave-overlay.plist

# Stop/start
launchctl unload ~/Library/LaunchAgents/com.rch.vaporwave-overlay.plist
launchctl load ~/Library/LaunchAgents/com.rch.vaporwave-overlay.plist
```

Requires Screen Recording permission in System Preferences for purple detection.

Source: `~/repo/research/shell/metal-overlay/`

## Maintenance

### Tool Updates (check every ~3 months)

Claude should periodically check for updates to these tools:

- **Shell**: zsh, starship, atuin, zoxide, eza, bat, ripgrep, fd, fzf, yazi, erdtree, glow, git-delta
- **Window management**: yabai, skhd, sketchybar, hammerspoon
- **Dev tools**: nushell, direnv, jq, yq
- **Apps**: iTerm2, Ghostty

Check for:
- Breaking changes in configs
- New features worth enabling
- Deprecated options to remove
- Security updates
