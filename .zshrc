. ~/.zprofile

eval "$(atuin init zsh)"
export HOMEBREW_NO_ENV_HINTS="true"
export ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE="fg=#190319,bg=#ffb1fe,bold"
export ZSH_AUTOSUGGEST_STRATEGY=(history completion)
source $(brew --prefix)/share/zsh-autosuggestions/zsh-autosuggestions.zsh
# bindkey '\t' complete-word
# bindkey '\t' expand-or-complete
# bindkey '\r' autosuggest-accept
bindkey '\t' autosuggest-accept
zstyle ':completion:*' completer _expand _complete _ignored _files
FPATH="$HOME/.zfunc:${FPATH}"
autoload -Uz compinit
compinit
