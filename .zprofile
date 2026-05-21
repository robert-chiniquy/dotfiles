# Guard against double-sourcing (login shells run .zprofile then .zshrc sources it again)
[[ -n "$__ZPROFILE_SOURCED" ]] && return 0
__ZPROFILE_SOURCED=1

which -s brew 1>/dev/null 2>/dev/null && (
  eval "$(/opt/homebrew/bin/brew shellenv)"
)

export PATH="$HOME/bin:$PATH"
export PATH="$HOME/go/bin:$PATH"
export PATH="$HOME/.opencode/bin:$PATH"
export PATH="$HOME/Library/Python/3.9/bin:$PATH"

# Rust/Cargo
[[ -f "$HOME/.cargo/env" ]] && . "$HOME/.cargo/env"

export STARSHIP_LOG=error
# Note: starship init moved to .zshrc (needs zle which isn't available in .zprofile)

export CLICOLOR=1

# Claude Code experimental features
export CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1

# Default editor (vim for quick edits, can override per-project)
export EDITOR="vim"
export VISUAL="vim"

# Fallback ls aliases (if eza not available)
if ! command -v eza &>/dev/null; then
  alias ls="ls -G"
  alias ll="ls -lG"
  alias lh="ls -lhG"
fi
export PATH="$HOME/.local/bin:$PATH"

# goose CLI
export GOOSE_CLI_DARK_THEME=1
export GOOSE_CLI_SHOW_COST=1
export GOOSE_TELEMETRY_OFF=1
export GOOSE_MODE=smart_approve
export GOOSE_MAX_TURNS=20
export GOOSE_RECIPE_PATH="$HOME/repo/dotfiles/.goose/recipes"
export GOOSE_STATUS_HOOK="$HOME/repo/dotfiles/bin/goose-status-hook"
export GOOSE_MOIM_MESSAGE_FILE=/tmp/goose-context

# pi.dev (pi-coding-agent)
export PI_TELEMETRY=0
