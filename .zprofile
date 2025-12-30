which -s brew 1>/dev/null 2>/dev/null && (
  eval "$(/opt/homebrew/bin/brew shellenv)"
)

export PATH="$HOME/go/bin:$PATH"

export STARSHIP_LOG=error
eval "$(starship init zsh)"

export CLICOLOR=1

# Fallback ls aliases (if eza not available)
if ! command -v eza &>/dev/null; then
  alias ls="ls -G"
  alias ll="ls -lG"
  alias lh="ls -lhG"
fi
