. ~/.zprofile

# === Completion System (BEFORE plugins) ===
[[ -d "$HOME/.zfunc" ]] && FPATH="$HOME/.zfunc:${FPATH}"
[[ -d /opt/homebrew/share/zsh/site-functions ]] && FPATH="/opt/homebrew/share/zsh/site-functions:${FPATH}"
[[ -d /usr/local/share/zsh/site-functions ]] && FPATH="/usr/local/share/zsh/site-functions:${FPATH}"

autoload -Uz compinit
if [[ -n ~/.zcompdump(#qN.mh+24) ]]; then
  compinit
else
  compinit -C
fi

zstyle ':completion:*' completer _expand _complete _ignored _files
zstyle ':completion:*' menu select
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'

# === Autosuggestions (AFTER compinit) ===
export ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE="fg=#190319,bg=#ffb1fe,bold"
export ZSH_AUTOSUGGEST_STRATEGY=(history completion)

for f in \
  /opt/homebrew/share/zsh-autosuggestions/zsh-autosuggestions.zsh \
  /usr/local/share/zsh-autosuggestions/zsh-autosuggestions.zsh \
  /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh; do
  [[ -f "$f" ]] && source "$f" && break
done

# === Atuin ===
command -v atuin &>/dev/null && eval "$(atuin init zsh)"

# === Key Bindings ===
bindkey '^[[C' autosuggest-accept

export HOMEBREW_NO_ENV_HINTS="true"
