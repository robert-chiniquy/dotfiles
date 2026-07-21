# Passive QoL Improvements

macOS defaults and system tweaks applied for quality of life. All changes are passive (no new keystrokes or workflows required).

**Important:** Before suggesting shell or config changes, always check existing setup (~/.zshrc, aliases, functions) to avoid conflicts and to avoid suggesting things already implemented.

**Principles:** Passive only, no new keystrokes, no lifestyle changes, low CPU/IO.

# Rejects

- Terminal cursor color unification (2026-06-26) — iTerm rewrites its plist from memory on quit, so live edits get clobbered; safe path would require quitting iTerm first. Vaporwave dynamic profile write went sideways under zsh noclobber and left an empty file. Not worth the risk for one color.

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

## 2026-03-04: PaperWM tiling window manager
Replaced skhd/yabai with PaperWM.spoon (Hammerspoon). Scrollable horizontal columns.
`defaults write com.apple.spaces spans-displays -bool false` for separate Spaces per display.
`PaperWM.external_bar = {top = 80}` for Sketchybar clearance.

## 2026-03-04: FFM disabled by default
Focus-follows-mouse fights PaperWM's tiling model. Disabled on startup, toggle with cmd+ctrl+f.

## 2026-03-04: FFM polling interval 500ms
Reduced from 150ms. Same perceived responsiveness, less CPU.

## 2026-03-04: Starship custom modules removed
Removed custom.git_diff_line_count (2 git commands per prompt) and custom.tokei_rust (runs tokei per prompt). Built-in git_status already shows modified count.

## 2026-03-04: Consolidated chpwd git functions
Replaced 5 separate chpwd functions (each calling git rev-parse) with one `_git_chpwd` that calls git once and prints a single terse line.

## 2026-03-04: Hammerspoon alerts auto-dismiss 1.5s
Fade in 0.1s, out 0.3s, dismiss at 1.5s instead of default. Overridden globally.

## 2026-03-04: CPU meter moved to Sketchybar
Removed Hammerspoon canvas widget (10s polling + shell exec). CPU and load now in Sketchybar right side. CPU plugin uses `ps -A -o %cpu` instead of expensive `top -l 1`.

## 2026-03-04: Meeting countdown in Sketchybar
Right side, updates 60s, color-coded by urgency (pink/gold/cyan/grey).

## 2026-03-04: HazeOver installed
System-wide dim for unfocused windows. Replaces non-functional hs.window.setAlpha approach.

## 2026-03-04: Sketchybar translucent
Background alpha 30%, blur 10, pink border, shadow on. Reads through HazeOver dimming.

## 2026-03-04: Wallpaper auto-rotate every 10 minutes
Hammerspoon timer, plus manual cmd+ctrl+w still works. pcall guard for missing directory.

## 2026-03-04: VaporwaveOverlay per-window mode
Changed from --fullscreen to per-window overlay on unfocused windows only. Focused window stays clean. Uses Accessibility API for focus detection. Auto-managed by battery state (plugged in + >50%).

## 2026-03-04: 4-finger trackpad swipe for PaperWM
`defaults write com.apple.AppleMultitouchTrackpad TrackpadFourFingerHorizSwipeGesture -int 0` to free gesture from macOS Spaces. PaperWM.swipe_fingers = 4.

## 2026-03-04: direnv log format restored
Changed from silent to grey text showing load/unload and exported variable names.

## 2026-06-26: Surface direnv state in starship prompt
```toml
# ~/repo/dotfiles/.config/starship.toml
[direnv]
disabled = false
```
Starship 1.26 added the direnv module; off by default. Invisible outside direnv dirs; in a direnv dir shows "loaded allowed" / "not loaded denied" / etc. Catches the silent case where `.envrc` failed to load (stale, denied, missing) — so PROMPT_ACCENT and project credentials don't go missing without a signal.

## 2026-06-26: Faster keyboard repeat (system-wide)
```bash
defaults write -g KeyRepeat -int 2          # ~33ms between repeats
defaults write -g InitialKeyRepeat -int 12  # ~180ms before first repeat
```
Defaults were KeyRepeat=2 (50ms) / InitialKeyRepeat=15 (225ms). Lower values produce snappier text editing in every app. Requires logout/login to take effect everywhere.

## 2026-06-26: Stop desktop-click from hiding all windows
```bash
defaults write com.apple.WindowManager EnableStandardClickToShowDesktop -bool false
```
Sequoia's default — click-on-desktop shoves all windows aside — surprises and destroys arrangements. Off restores pre-Sequoia behavior.

## 2026-06-26: Stop new docs from defaulting to iCloud
```bash
defaults write -g NSDocumentSaveNewDocumentsToCloud -bool false
```
TextEdit/Preview/Pages/Numbers/Keynote default their save dialog to local instead of iCloud Drive. iCloud remains available, just not the default.

## 2026-06-26: Remove dock's auto-appended recent apps
```bash
defaults write com.apple.dock show-recents -bool false && killall Dock
```
Dock no longer auto-adds the last-used apps section — pinned set stays stable.

## 2026-06-26: Vaporwave text-selection + accent color in native macOS apps
```bash
defaults write -g AppleHighlightColor -string "1.0 0.0 0.6 Pink"
defaults write -g AppleAccentColor -int 6
```
Text selection highlight becomes hot-pink (#ff0099-ish) and Accent (buttons, checkboxes, menu selections) goes Pink family. Threads the vaporwave palette from CLI into every native Cocoa app. Per-app on next launch.

## 2026-06-26: Reveal ~/Library in Finder permanently
```bash
chflags nohidden ~/Library
```
No more option-click Go > Library dance to reach Application Support, LaunchAgents, plists, iTerm DynamicProfiles, Hammerspoon config, mail rules.

## 2026-06-26: Finder path bar + status bar
```bash
defaults write com.apple.finder ShowPathbar -bool true
defaults write com.apple.finder ShowStatusBar -bool true
killall Finder
```
Path bar (breadcrumb, click any segment to jump) + status bar (file count + free space) at the bottom of every Finder window.

## 2026-06-26: Hide desktop icons entirely
```bash
defaults write com.apple.finder CreateDesktop -bool false && killall Finder
```
Files in ~/Desktop remain accessible (Finder sidebar, Stacks, Cmd-Shift-D) but no longer render as icons on the wallpaper — keeps the neon-grit rotation unobscured.

## 2026-06-26: Silence crashed-app modal dialogs
```bash
defaults write com.apple.CrashReporter DialogType -string "none"
```
No more "MyApp quit unexpectedly" popup — logs still get written to ~/Library/Logs, apple's diagnostic collection unchanged.

## 2026-06-26: Finder Get Info opens with all panes expanded
```bash
defaults write com.apple.finder FXInfoPanesExpanded -dict \
    General -bool true OpenWith -bool true \
    Privileges -bool true MetaData -bool true
killall Finder
```
Cmd-I on a file no longer starts with MetaData/OpenWith/Privileges collapsed.

## 2026-06-26: Cmd-Tab follows the app to its space
```bash
defaults write -g AppleSpacesSwitchOnActivate -bool true
```
Cmd-Tab now jumps to the space containing the target app instead of yanking that app's window into the current space. Pairs with the pinned-spaces setup (mru-spaces=false).

## 2026-06-26: Modern git perf defaults (large-repo self-optimization)
```bash
git config --global feature.manyFiles true
git config --global core.commitGraph true
git config --global gc.writeCommitGraph true
git config --global fetch.writeCommitGraph true
git config --global gc.auto 6700
```
`feature.manyFiles` enables index.version=4 + index.skipHash for faster status/diff on repos with many files. `writeCommitGraph` on gc + fetch builds a graph that makes log / graph / blame / path-history operations dramatically faster on future git ops. `gc.auto=6700` reduces GC frequency (was 256). Effective incrementally; no manual gc needed. Applies uniformly across every context git runs in — CLI, agents, hooks — consistent with the whole-computer-consistency axiom.


## 2026-07-01: Promote universal env vars to .zshenv (subprocess/launchd/cron consistency)
```bash
# added to ~/.zshenv (removed from ~/.zshrc):
export HOMEBREW_NO_AUTO_UPDATE=1
export HOMEBREW_NO_INSTALL_CLEANUP=1
export HOMEBREW_NO_ANALYTICS=1
export HOMEBREW_NO_ENV_HINTS=true
export RIPGREP_CONFIG_PATH="$HOME/.ripgreprc"
```
These now fire in every zsh context — launchd, cron, git hooks, non-interactive subshells, ssh-non-login — not just interactive shells. Consistent behavior across the whole computer per the QoL axiom.

## 2026-07-01: Proper zsh three-file separation (.zshenv is source of truth)
```
~/.zshenv     — universal env vars, PATH, brew shellenv (reaches every context)
~/.zprofile   — login-only (aliases for eza fallback)
~/.zshrc      — interactive UX (prompt, completion, key bindings, colors)
```
Fixed pre-existing bug where brew shellenv was wrapped in `(...)` subshell — HOMEBREW_PREFIX/MANPATH/INFOPATH weren't taking effect anywhere. Now launchd jobs, cron, ssh remote 'command', and app-launched processes get the same env as interactive shells. Consistent behavior across every context per the QoL axiom.


## 2026-07-01: Restore PROMPT_ACCENT color in live starship prompt
```toml
# ~/repo/dotfiles/starship.toml   (live; ~/.config/starship.toml symlinks here)
[custom.accent]
command = "$HOME/repo/dotfiles/bin/prompt-char"
when = "true"
format = ' $output'
```
The [custom.accent] block previously had `command = "echo $__prompt_accent_cache"`, referencing a zsh shell variable that starship's subshell couldn't see (not exported). Switched to invoking `prompt-char` which reads the exported `PROMPT_ACCENT` env var from direnv directly.

Gotchas I hit and burned on:
- Live config is `~/repo/dotfiles/starship.toml`, NOT `~/repo/dotfiles/.config/starship.toml`. Verify with `readlink ~/.config/starship.toml` before editing.
- Starship's `shell = ["sh", "-c"]` field silently discards custom-module output. Use the default shell.
- Custom modules require `when` set to a shell command that exits 0 (like `"true"`); no `when` means the module doesn't run.

## 2026-07-01: Explicit UTF-8 locale in .zshenv
```bash
# added to ~/.zshenv
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
```
Every zsh context now gets the same locale — no more "works in my terminal, breaks in the script" from tools falling back to C/POSIX when launchd/cron/ssh strip the locale env.

## 2026-07-01: Auto-update dependent branch refs during rebase
```bash
git config --global rebase.updateRefs true
```
For stacked-branch workflows: rebase now retargets dependent refs automatically instead of leaving them pointing at the old base.

## 2026-07-01: Silence zsh startup errors in agent-invoked (non-TTY) shells
```zsh
# ~/repo/dotfiles/.zshrc line 987 — added -t 0 guard
[[ -t 0 && -f /opt/homebrew/opt/fzf/shell/key-bindings.zsh ]] && source /opt/homebrew/opt/fzf/shell/key-bindings.zsh
```
And line 1004 — sysctl absolute path with fallback:
```zsh
export MAKEFLAGS="-j$(/usr/sbin/sysctl -n hw.ncpu 2>/dev/null || echo 4)"
```
Root causes when agents run `zsh -i -c "..."` (technically interactive but no TTY): fzf's key-bindings.zsh line 195 evals `zle on` which fails without a TTY; and sysctl at `/usr/sbin/sysctl` isn't on the stripped agent PATH. Both errors were leaking into every agent transcript.

## 2026-07-01: Multi-column output for git list commands
```bash
git config --global column.ui auto
```
`git branch`, `git tag`, `git stash list`, `git config --list` now use terminal-width columns when entries are short — auto-disables when piped or too long.

## 2026-07-01: ISO-local dates in git log
```bash
git config --global log.date iso-local
```
`git log` dates now show as `2026-07-01 12:34:56 -0700` — greppable, sortable, consistent with everything else.

## 2026-07-02: Cap clanker-lint at half cores via personal shim
```sh
# ~/bin/clanker-lint (shadows ~/go/bin/clanker-lint on PATH)
#!/bin/sh
REAL=/Users/rch/go/bin/clanker-lint
if [ "$1" = "run" ]; then
    shift
    case " $* " in
        *" -j "*|*" --concurrency "*|*" --concurrency="*)
            exec "$REAL" run "$@" ;;
    esac
    exec "$REAL" run --concurrency=6 "$@"
fi
exec "$REAL" "$@"
```
c1's `.claude/` invocations (`verifier`, `pr-opener`, `/lint-go`, `/test-changes`, `dev-cycle-learnings`) all shell out to `make lint` / `./ci/lint_diff.sh`, which end up at `clanker-lint run` — default 0 = all 12 cores → laptop crushed. Shim caps at 6 (leaves headroom for the editor + agents while lint runs). Override per-call with `-j <N>`. Does NOT touch the shared `.golangci.yml` — team config unchanged.

## 2026-07-07: netnudge helper — fast recovery after network switch
```sh
# ~/bin/netnudge
netnudge          # cycles en0 (Wi-Fi)
netnudge en5      # or a specific interface
netnudge --list   # show wireless interfaces
```
Root cause of the symptom "network works but agents time out": HTTP keep-alive sockets are bound to the pre-switch interface's IP. New connections use the new default route (browser + ping work) but old sockets keep routing out the now-dead interface, hanging until TCP retry timeout (~2 min). Cycling Wi-Fi forces the OS to tear down those bound sockets so HTTP clients reconnect immediately on the new interface. Requires sudo prompt on invocation.

## 2026-07-07: NOPASSWD sudoers entry for netnudge
```
# /etc/sudoers.d/netnudge (mode 0440, root:wheel)
rch ALL=(ALL) NOPASSWD: /usr/sbin/networksetup -setairportpower * on
rch ALL=(ALL) NOPASSWD: /usr/sbin/networksetup -setairportpower * off
```
Scoped tightly: only setairportpower on/off with any single-argument interface name. Cannot escalate to other networksetup subcommands, cannot chain arguments. `netnudge` now runs with no prompt.

## 2026-07-13: netnudge — before/after lsof harness sockets
```sh
netnudge --dry     # print current harness sockets without cycling
netnudge           # cycle en0, print before/after harness sockets
```
Enhanced netnudge to list ESTABLISHED TCP sockets held by known harnesses (claude/opencode/codex/pi/sqfan/node/cursor/ghostty/python) before and after the cycle. `NETNUDGE_HARNESSES="a,b,c"` env var overrides the list. Verified in the wild: dry-run caught a codex socket bound to a stale Wi-Fi IP (10.103.3.28) while the current default route was hotspot (172.20.10.9) — the exact black-hole pattern.

## 2026-07-13: Claude Code statusline — active interface indicator
```
~/bin/claude-statusline    # wraps claude-hud, appends [iface SSID] or [HOTSPOT iface ip]
```
`~/.claude/settings.json` statusLine.command now points at the wrapper. Wi-Fi appears in cyan, Apple personal hotspot subnet (172.20.10.*) in hot pink. Any network switch is visible at a glance in the prompt gutter.

## 2026-07-13: NOPASSWD sudoers for netnudge (deferred install)
Install script at `~/bin/install-netnudge-sudoers`. Must be run in a live TTY (iTerm), not via Claude Code's Bash tool (no TTY for sudo prompt). Once installed, `netnudge` runs with zero prompts.

## 2026-07-13: peptalk-hotspot MTU workaround + LaunchAgent
```
~/bin/peptalk-mtu-watch                       # daemon (--once, --status, --stop)
~/Library/LaunchAgents/dev.rch.peptalk-mtu-watch.plist
~/bin/install-network-sudoers                 # one-shot installer (needs TTY sudo)
~/bin/uninstall-network-sudoers               # revert everything
```
Polls Wi-Fi SSID every 15s. On SSID="peptalk" sets en0 MTU=1400 (works around iOS-hotspot MSS-clamping issue). Any other SSID → restores MTU=1500. Consolidated sudoers file `/etc/sudoers.d/rch-network` covers both `networksetup -setairportpower * on|off` (netnudge) and `/sbin/ifconfig * mtu *` (this watcher). Log at `~/Library/Logs/peptalk-mtu-watch.log`.

## 2026-07-13: codex-tab wrapper — session identity in iTerm tab title + badge
```sh
~/bin/codex-tab <name> [codex-args...]
```
Codex TUI doesn't expose session name in its status line and `[tui]` config has no user-customizable status fields. This wrapper sets the tab title (OSC 0), iTerm badge (OSC 1337 SetBadgeFormat), and a `codexname` user variable (SetUserVar) before exec'ing codex. Works around the "which tab is which codex session" identification problem.

## 2026-07-14: Shared CARGO_TARGET_DIR across all Rust checkouts
```bash
# ~/.zshenv
export CARGO_TARGET_DIR="$HOME/.cache/cargo-target"
```
Documented in `~/repo/latchkey-project/README.md` under Build And Artifacts. Every Rust build across every checkout and worktree now writes to one shared cache directory. Cargo deduplicates identical `(crate, version, features, rustc, target-triple)` tuples across checkouts. Prevents the disk-crisis pattern of N worktrees × ~2 GB target/ each. Concurrent builds share a lock briefly; in practice imperceptible. Override for one build via `CARGO_TARGET_DIR=./target cargo build`.

Also identified but not yet acted on: `~/repo/c1/.gocache` (4 GB, actively growing) — c1's Makefile / CI scripts explicitly set `GOCACHE=./.gocache` locally, duplicating Go's default `~/Library/Caches/go-build`. Changing that likely affects c1 CI parity; requires understanding why the local cache exists before modifying.

## 2026-07-17: Auto-run disk-emergency on new interactive shell if <4 GB free
```sh
~/bin/disk-emergency                            # idempotent, threshold via DISK_EMERGENCY_THRESHOLD_GB
~/Library/Logs/disk-emergency.log               # rolling log
```
`.zshrc` hook fires it on interactive shell start when <4 GB free. Reclaim order (safest first): empty ~/.Trash, remove /private/tmp/c1-*/sqfan-*/lk-*-agent scratch dirs, clear Chrome cache, plaid-lint + clanker-lint caches, stale WirelessDiagnostics_*.tar.gz, gzip Claude transcripts >30d. Never touches repos, Colima, cargo-target, or active caches. macOS notification banner shows before/after GB and warns if still below threshold after reclaim. First smoke test on this session: 33 → 41 GB (+8 GB, 1537 old transcripts gzipped).

## 2026-07-20: c1 worktree GOCACHE hygiene
```zsh
# ~/.zshenv
export GOCACHE="$HOME/Library/Caches/go-build"

# ~/repo/dotfiles/.zshrc (chpwd hook)
_c1_worktree_envrc_link() {
    [[ -f .envrc && ! -e .envrc.local && -f .git ]] || return 0
    [[ -f ~/repo/c1/.envrc.local ]] || return 0
    /usr/bin/grep -q '\.envrc\.local' .envrc || return 0
    ln -s ~/repo/c1/.envrc.local .envrc.local
    direnv allow . >/dev/null 2>&1
}
chpwd_functions+=(_c1_worktree_envrc_link)
```
`.envrc.local` at ~/repo/c1/ pins GOCACHE to the shared cache. chpwd hook symlinks it into new c1 worktrees automatically on first cd. .zshenv sets it globally too as a fallback. Result: cache no longer scatters into per-worktree build/go-build-cache dirs. Existing worktrees (c1-21070, c1-iga-3173, c1-iga-3176, c1-tt26) already symlinked.

## 2026-07-21: Cap plaid-lint at 6 CPUs via personal shim
```sh
# ~/bin/plaid-lint (shadows ~/go/bin/plaid-lint on PATH)
# identical pattern to the clanker-lint shim: injects --concurrency=6
# into `plaid-lint run` unless -j/--concurrency given explicitly.
```
Context: agents opt into plaid-lint per-invocation (USE_PLAID=1); on a cold cache at default concurrency (0 = all 12 cores) it saturates a 24GB/12-core machine. Sizing guidance from the team thread: ~4GB RAM per vCPU → 6 for this laptop. Both lint engines (clanker, plaid) now capped identically. Override per-call with -j <N>.

## 2026-07-21: Pin LINT_CONCURRENCY=4 in c1/.envrc.local
c1's `ci/lint_diff.sh` falls back to `nproc || echo 4` on machines without cgroups. nproc isn't installed today (so the fallback is 4), but installing coreutils would silently flip it to 12 on this 12-core box. Pinned explicitly; propagates to all c1 worktrees via the .envrc.local symlinks. Companion to the plaid-lint/clanker-lint -j6 shims (which only apply when no explicit --concurrency is passed; lint_diff.sh passes one).
