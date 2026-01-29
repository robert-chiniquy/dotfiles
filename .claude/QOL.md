# Passive QoL Improvements

macOS defaults and system tweaks applied for quality of life. All changes are passive (no new keystrokes or workflows required).

**Important:** Before suggesting shell or config changes, always check existing setup (~/.zshrc, aliases, functions) to avoid conflicts and to avoid suggesting things already implemented.

**Principles:** Passive only, no new keystrokes, no lifestyle changes, low CPU/IO.

# Rejects

Never suggest these again:
- Raycast/Alfred - don't need launcher
- Menu bar managers (Bartender, Hidden Bar) - not needed
- Clipboard managers (Maccy) - not needed
- Alt-Tab - not needed
- Productivity timers (Horo, etc.) - GTD freak stuff
- Fortune/cowsay - bells and whistles
- Screensavers (Brooklyn, pipes, etc.) - bells and whistles
- Terminal notifier - bells and whistles
- cava audio visualizer - bells and whistles
- Custom system sounds - bells and whistles
- Quick capture tools - not needed
- Any GTD/productivity apps
- Any new aliases or keystrokes
- Anything requiring lifestyle change
- Disable press-and-hold accent menu - user likes it
- Quick Look anything - user hates Quick Look
- Sudo hint on permission denied - annoying

# Applied

## 2026-01-28: Disable screenshot shadows
```bash
defaults write com.apple.screencapture disable-shadow -bool true && killall SystemUIServer
```
Screenshots are cleaner without the drop shadow, easier to use in docs and chat.

## 2026-01-28: Screenshots go to ~/Screenshots
```bash
mkdir -p ~/Screenshots
defaults write com.apple.screencapture location ~/Screenshots && killall SystemUIServer
```
Keeps desktop clean, screenshots in dedicated folder.

## 2026-01-28: Expand save dialogs by default
```bash
defaults write -g NSNavPanelExpandedStateForSaveMode -bool true
defaults write -g NSNavPanelExpandedStateForSaveMode2 -bool true
```
Save dialogs show full file browser instead of collapsed view.

## 2026-01-28: Disable smart quotes and dashes
```bash
defaults write -g NSAutomaticQuoteSubstitutionEnabled -bool false
defaults write -g NSAutomaticDashSubstitutionEnabled -bool false
```
Prevents curly quotes and em-dashes when typing. Better for code/terminal.

## 2026-01-28: Disable .DS_Store on network and USB drives
```bash
defaults write com.apple.desktopservices DSDontWriteNetworkStores -bool true
defaults write com.apple.desktopservices DSDontWriteUSBStores -bool true
```
No more .DS_Store files littering shared drives and USB sticks.

## 2026-01-28: Faster dock auto-hide
```bash
defaults write com.apple.dock autohide-delay -float 0
defaults write com.apple.dock autohide-time-modifier -float 0.15
killall Dock
```
Dock appears instantly when hidden, no delay.

## 2026-01-28: Faster Mission Control animations
```bash
defaults write com.apple.dock expose-animation-duration -float 0.1 && killall Dock
```
Mission Control transitions are snappier.

## 2026-01-28: Disable Finder animations
```bash
defaults write com.apple.finder DisableAllAnimations -bool true && killall Finder
```
No more slow folder open/close animations.

## 2026-01-28: Folders sort first in Finder
```bash
defaults write com.apple.finder _FXSortFoldersFirst -bool true && killall Finder
```
Folders always appear before files in any sort order.

## 2026-01-28: Show full path in Finder title bar
```bash
defaults write com.apple.finder _FXShowPosixPathInTitle -bool true && killall Finder
```
Finder title shows /full/path/to/folder instead of just folder name.

## 2026-01-28: Disable Time Machine new disk prompts
```bash
defaults write com.apple.TimeMachine DoNotOfferNewDisksForBackup -bool true
```
No more "use this disk for backup?" dialogs when plugging in USB drives.

## 2026-01-28: Disable auto-capitalization
```bash
defaults write -g NSAutomaticCapitalizationEnabled -bool false
```
No automatic capitalization at start of sentences.

## 2026-01-28: Show all file extensions in Finder
```bash
defaults write -g AppleShowAllExtensions -bool true && killall Finder
```
All files show their true extensions (.app, .dmg, etc).

## 2026-01-28: TextEdit plain text by default
```bash
defaults write com.apple.TextEdit RichText -int 0
```
New TextEdit documents are plain text instead of rich text.

## 2026-01-28: Photos doesn't auto-open on device plug
```bash
defaults -currentHost write com.apple.ImageCapture disableHotPlug -bool true
```
No more Photos launching when plugging in phone/camera.

## 2026-01-28: Faster spring loading
```bash
defaults write -g com.apple.springing.delay -float 0.3
```
Drag file over folder, opens quicker (0.3s instead of default).

## 2026-01-28: Git rerere enabled
```bash
git config --global rerere.enabled true
```
Git remembers how you resolved merge conflicts and reuses the resolution.

## 2026-01-28: Faster window resize animation
```bash
defaults write -g NSWindowResizeTime -float 0.001
```
Window resizing is instant instead of animated.

## 2026-01-28: Higher Bluetooth audio quality
```bash
defaults write com.apple.BluetoothAudioAgent "Apple Bitpool Min (editable)" -int 40
```
Bluetooth headphones use higher bitrate. Reconnect headphones to apply.

## 2026-01-28: Shell improvements (zshrc)
```bash
setopt HIST_VERIFY        # Show history expansion before running
WORDCHARS='${WORDCHARS//[\/]}'  # Ctrl+W stops at /
zstyle ':completion:*' use-cache on  # Faster tab completion
export LESS="-R -F -X -i -J -W"  # Better pager behavior
```
History expansion previews, smarter word deletion, cached completions, improved less.

## 2026-01-28: Auto-prune typos from history
```bash
__prune_typos() {
    [[ $? -eq 127 ]] || return
    fc -W
    head -n -1 "$HISTFILE" > "$HISTFILE.tmp" && mv "$HISTFILE.tmp" "$HISTFILE"
    fc -R
}
precmd_functions+=(__prune_typos)
```
Commands that return "command not found" (exit 127) are automatically removed from history.

## 2026-01-28: Port-in-use helper
```bash
__port_helper() {
    [[ $? -ne 0 ]] && fc -ln -1 | grep -q "address.*in use\|EADDRINUSE" && \
        lsof -i -P | grep LISTEN
}
precmd_functions+=(__port_helper)
```
When a command fails with "address in use", automatically shows what's using the port.

## 2026-01-28: Git dirty repo reminder
```bash
__git_dirty_reminder() {
    [[ -d .git ]] && ! git diff --stat --quiet 2>/dev/null && git diff --stat 2>/dev/null | tail -1
}
chpwd_functions+=(__git_dirty_reminder)
```
Shows diff stat summary when cd'ing into a repo with uncommitted changes.

## 2026-01-28: Warn before large rm
```bash
rm() {
    if [[ "$*" =~ "-rf" ]] || [[ "$*" =~ "-r" ]]; then
        local target="${@[-1]}"
        [[ -e "$target" ]] && local size=$(du -sh "$target" 2>/dev/null | cut -f1)
        [[ -n "$size" ]] && print -P "%F{yellow}Removing $size%f"
    fi
    command rm "$@"
}
```
Shows size before rm -r operations.

## 2026-01-28: Auto-title terminal
```bash
preexec() { print -Pn "\e]0;$1\a"; }
precmd() { print -Pn "\e]0;%~\a"; }
```
Terminal title shows current command while running, current directory when idle.

## 2026-01-28: SSH key auto-add
```bash
ssh-add -l &>/dev/null || ssh-add --apple-use-keychain ~/.ssh/id_* 2>/dev/null
```
SSH keys auto-load into agent on first use, stored in macOS keychain.

## 2026-01-28: Disable Spotlight indexing for ~/repo
```bash
touch ~/repo/.metadata_never_index
```
Reduces disk I/O, Spotlight skips code folders.

## 2026-01-28: Git fsmonitor and untracked cache
```bash
git config --global core.fsmonitor true
git config --global core.untrackedcache true
```
Faster git status using macOS FSEvents.

## 2026-01-28: Increase file descriptor limit
```bash
ulimit -n 10240
```
Helps with large projects and many open files.

## 2026-01-28: Disable Siri analytics
```bash
defaults write com.apple.assistant.support "Siri Data Sharing Opt-In Status" -int 2
```

## 2026-01-28: Disable personalized ads
```bash
defaults write com.apple.AdLib allowApplePersonalizedAdvertising -bool false
```

## 2026-01-28: Git default branch main
```bash
git config --global init.defaultBranch main
```

## 2026-01-28: Faster Finder windows
```bash
defaults write com.apple.finder NSWindowResizeTime -float 0.001
```
Finder windows open instantly.
