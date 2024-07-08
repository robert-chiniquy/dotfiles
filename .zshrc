. ~/.zprofile

eval "$(atuin init zsh)"
export ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE="fg=#190319,bg=#ffb1fe,bold"
export ZSH_AUTOSUGGEST_STRATEGY=(history completion)

which -s brew && (
  export HOMEBREW_NO_ENV_HINTS="true"
  source $(brew --prefix)/share/zsh-autosuggestions/zsh-autosuggestions.zsh
) || (
  source /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh
)


# bindkey '\t' complete-word
# bindkey '\t' expand-or-complete
# bindkey '\r' autosuggest-accept
bindkey '\t' autosuggest-accept
zstyle ':completion:*' completer _expand _complete _ignored _files
FPATH="$HOME/.zfunc:${FPATH}"
autoload -Uz compinit
compinit
