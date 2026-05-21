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

## Emergency: Windows Stuck Offscreen

PaperWM (in Hammerspoon) occasionally tiles a window past the screen edge so it can't be reached with the mouse. Escalate in this order — stop as soon as your windows are back.

**1. PaperWM retile hotkey**

```
cmd+ctrl+-
```

Bound to `refresh_windows` in `.hammerspoon/init.lua`. Retiles every window in the current space.

**2. Force retile from the CLI**

If the hotkey is unreachable (e.g. wrong app focused):

```bash
hs -c 'PaperWM.windows.refreshWindows()'
```

**3. Reload Hammerspoon**

```
cmd+ctrl+r
```

Or from a terminal:

```bash
hs -c 'hs.reload()'
```

**4. Forcibly drag every offscreen window onto the main display**

Last-resort rescue when a window's frame is so far offscreen that PaperWM's tiling logic refuses to touch it. Walks every standard window, checks if its frame is past the main screen's bounds, and resets it to a 800×600 box at (100,100):

```bash
hs -c '
local screen = hs.screen.mainScreen()
local sf = screen:frame()
local moved = 0
for _, win in ipairs(hs.window.allWindows()) do
    if win:isStandard() then
        local f = win:frame()
        local offscreen = f.x + f.w < sf.x + 50
                       or f.x > sf.x + sf.w - 50
                       or f.y + f.h < sf.y + 50
                       or f.y > sf.y + sf.h - 50
        if offscreen then
            win:setFrame({x = sf.x + 100, y = sf.y + 100, w = math.min(800, sf.w - 200), h = math.min(600, sf.h - 200)})
            moved = moved + 1
        end
    end
end
pcall(function() PaperWM.windows.refreshWindows() end)
return "rescued " .. moved .. " offscreen windows"
'
```

**5. Disable PaperWM for the session**

If PaperWM keeps re-misplacing windows after rescue:

```bash
hs -c 'PaperWM:stop()'
```

Re-enable with `hs -c 'PaperWM:start()'` or just `cmd+ctrl+r`.

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
