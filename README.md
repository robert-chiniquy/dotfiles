### local dotfiles
`git clone https://github.com/robert-chiniquy/dotfiles.git && cd dotfiles && ./install.sh`

## New Machine Setup

Full bootstrap for a fresh Mac, in order:

1. **Prerequisites**
   ```bash
   xcode-select --install                 # Command Line Tools (git, cc, make)
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```
2. **Clone + install** — symlinks every dotfile and runs `brew bundle` off the
   committed `Brewfile` (CLI tools, casks, taps):
   ```bash
   git clone https://github.com/robert-chiniquy/dotfiles.git && cd dotfiles && ./install.sh
   ```
3. **Language-toolchain globals** — the `Brewfile` covers Homebrew packages,
   but tools installed via language toolchains are not in it. After installing
   the toolchains (rustup, go, node via brew), reinstall your global tools:
   ```bash
   rustup default stable
   # cargo install <your tools>   — see your private migration notes for the list
   # go install <your tools>@latest
   # npm i -g <your global CLIs>
   ```
4. **Shell** — `.zshenv` (universal env: PATH, GOCACHE, locale) and `.zshrc`
   (interactive) are symlinked by `install.sh`. Open a new shell to load them.
   Note: `.zshenv` pins `GOMAXPROCS` and the lint shims pin `--concurrency`
   to a 12-core box — retune for the new machine's core count.
5. **Applications** — GUI apps are casks in the `Brewfile`, so `brew bundle`
   (step 2) installs them: iTerm2, VS Code, Cursor, Chrome, 1Password,
   Rectangle, Slack, Notion, Linear, Spotify, Zoom, Twingate, GitHub Desktop,
   JetBrains Toolbox, Hammerspoon, Karabiner, Übersicht, Ghostty, and more.
   - **VS Code** — extensions + settings are captured under `.config/vscode/`.
     Restore with: `~/.config/vscode/restore.sh` (installs all extensions from
     `extensions.txt`, symlinks `settings.json`).
   - **App Store apps** (not in Brewfile): Xcode, Keynote/Numbers/Pages —
     install from the App Store, or `brew install mas` then script them.
   - **Internal apps** (not public casks): install from their internal sources.
6. **Manual / GUI steps** — see "Manual Restores" below (iTerm2 plist, overlay
   LaunchAgent + Screen Recording permission) and run the macOS `defaults`
   tweaks from `.claude/QOL.md`.
7. **Re-auth, don't copy** — credentials do not live in this repo and must be
   re-established on the new machine: `gh auth login`, `atuin login`, VPN,
   editor/agent sign-ins, cloud CLIs. Rotate any long-lived tokens rather than
   transplanting them.

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
