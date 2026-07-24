# ~/.zshenv — sourced by every zsh (interactive, non-interactive, login,
# subprocess, launchd, cron, ssh non-login). Anything here reaches every
# context the shell runs in. Env vars only — no aliases, no interactive UX.

# Homebrew (sets HOMEBREW_PREFIX, HOMEBREW_CELLAR, MANPATH, INFOPATH, PATH)
[[ -x /opt/homebrew/bin/brew ]] && eval "$(/opt/homebrew/bin/brew shellenv)"

# Cargo (prepends ~/.cargo/bin to PATH)
[[ -f "$HOME/.cargo/env" ]] && . "$HOME/.cargo/env"

# Auto-dedupe PATH so subsequent prepends (from /etc/zprofile's path_helper
# or user's .zprofile) merge cleanly instead of accumulating duplicates.
typeset -U path PATH fpath FPATH

# User-installed binaries (last-prepended is highest priority)
export PATH="$HOME/go/bin:$PATH"
export PATH="$HOME/bin:$PATH"
export PATH="$HOME/.opencode/bin:$PATH"
export PATH="$HOME/Library/Python/3.9/bin:$PATH"
export PATH="$HOME/.local/bin:$PATH"

# Locale (default to UTF-8 everywhere; launchd/cron may not set these)
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

# Universal env vars — same behavior in every context
export EDITOR="vim"
export VISUAL="vim"
export CLICOLOR=1
export STARSHIP_LOG=error
export CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1
export PI_TELEMETRY=0

# Homebrew flags
export HOMEBREW_NO_AUTO_UPDATE=1
export HOMEBREW_NO_INSTALL_CLEANUP=1
export HOMEBREW_NO_ANALYTICS=1
export HOMEBREW_NO_ENV_HINTS=true
export RIPGREP_CONFIG_PATH="$HOME/.ripgreprc"

# Shared cargo target — every Rust build across every checkout/worktree
# writes to one place so cargo can dedup by dependency hash. Reclaims
# ~10-30 GB vs the default per-checkout target/ dirs.
export CARGO_TARGET_DIR="$HOME/.cache/cargo-target"

# Go build cache — pin to the shared macOS default so no rogue tool
# (sqfan agent, worktree with stale env) points it at a per-batch dir.
export GOCACHE="$HOME/Library/Caches/go-build"

# Cap Go runtime threads machine-wide (12-core/24GB box): leaves headroom
# when multiple agents compile/lint simultaneously. Per-invocation override:
# GOMAXPROCS=12 <cmd>.
export GOMAXPROCS=8
