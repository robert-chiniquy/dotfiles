# ~/.zprofile — login shell only.
# Env vars and brew shellenv live in ~/.zshenv (universal).
# PATH prepends duplicated here because /etc/zprofile runs path_helper
# BETWEEN .zshenv and .zprofile and demotes user paths. `typeset -U path`
# in .zshenv dedupes automatically, so these re-prepends promote user
# paths back to first position without introducing duplicates.

[[ -n "$__ZPROFILE_SOURCED" ]] && return 0
__ZPROFILE_SOURCED=1

# Re-prepend user paths so they win over /etc/paths.d entries.
# Order: last prepended = highest priority.
export PATH="$HOME/go/bin:$PATH"
export PATH="$HOME/bin:$PATH"
export PATH="$HOME/.opencode/bin:$PATH"
export PATH="$HOME/Library/Python/3.9/bin:$PATH"
export PATH="$HOME/.local/bin:$PATH"

# Fallback ls aliases (if eza not available)
if ! command -v eza &>/dev/null; then
  alias ls="ls -G"
  alias ll="ls -lG"
  alias lh="ls -lhG"
fi
