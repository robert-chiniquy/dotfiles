# Guard against double-sourcing (login shells run .zprofile then .zshrc sources it again)
[[ -n "$__ZPROFILE_SOURCED" ]] && return 0
__ZPROFILE_SOURCED=1

which -s brew 1>/dev/null 2>/dev/null && (
  eval "$(/opt/homebrew/bin/brew shellenv)"
)

export PATH="$HOME/bin:$PATH"
export PATH="$HOME/go/bin:$PATH"
export PATH="$HOME/.opencode/bin:$PATH"

# Rust/Cargo
[[ -f "$HOME/.cargo/env" ]] && . "$HOME/.cargo/env"

export STARSHIP_LOG=error
# Note: starship init moved to .zshrc (needs zle which isn't available in .zprofile)

export CLICOLOR=1

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
