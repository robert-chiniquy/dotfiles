
which -s brew 1>/dev/null 2>/dev/null && (
  eval "$(/opt/homebrew/bin/brew shellenv)"
)

eval "$(starship init zsh)"

export CLICOLOR=1
alias ls="ls -G"
alias ll="ls -lG"
alias lh="ls -lhG"
