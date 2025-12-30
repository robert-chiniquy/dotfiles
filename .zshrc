. ~/.zprofile

# === Modern CLI Tools ===
# eza (better ls)
if command -v eza &>/dev/null; then
  alias ls='eza --icons --color=always --group-directories-first'
  alias ll='eza --icons --color=always --group-directories-first -l'
  alias la='eza --icons --color=always --group-directories-first -la'
  alias lt='eza --icons --color=always --tree --level=2'
  
  # Vaporwave colors for eza (using ANSI color codes)
  export EZA_COLORS="da=38;5;51:di=38;5;51:ex=38;5;201:*.md=38;5;213:*.json=38;5;221:*.yaml=38;5;51:*.go=38;5;51:*.js=38;5;221:*.ts=38;5;51"
fi

# zoxide (smart cd)
if command -v zoxide &>/dev/null; then
  eval "$(zoxide init zsh)"
fi

# bat (better cat)
if command -v bat &>/dev/null; then
  alias cat='bat --style=plain --paging=never'
  alias bcat='bat --style=full'
  export BAT_THEME="base16"
fi

# ripgrep with colors
if command -v rg &>/dev/null; then
  export RIPGREP_CONFIG_PATH="$HOME/.ripgreprc"
fi

# direnv
if command -v direnv &>/dev/null; then
  eval "$(direnv hook zsh)"
fi

# JSON/YAML viewers
alias jqc='jq -C | less -R'
alias yqc='yq -C | less -R'

# === Markdown Rendering Defaults ===
# glow wrapper: always use pager mode with vaporwave theme
glow() {
  # Set less prompt to show percentage and line numbers
  LESS='-R -M -i -j3 -P?f%f (%i/%m) ?lt Line %lt?L/%L. :byte %bB?s/%s. .?e(END):?pB %pB\%.. (press h for help)' \
  command glow -p -s ~/.config/glow/vaporwave.json "$@"
}

# Enhanced md viewer with TOC support
mdtoc() {
  local file="${1:-}"
  if [[ -z "$file" ]]; then
    echo "Usage: mdtoc FILE.md" >&2
    return 1
  fi
  
  # Extract TOC from headings
  local toc=$(grep -E '^#{1,6} ' "$file" | sed 's/^#/ /' | sed 's/^#/  /' | sed 's/^#/   /')
  
  # Show TOC first, then document
  {
    echo -e "\033[1;38;5;201m╔══════════════ TABLE OF CONTENTS ══════════════╗\033[0m"
    echo "$toc" | head -30
    echo -e "\033[1;38;5;201m╚═══════════════════════════════════════════════╝\033[0m"
    echo ""
    echo -e "\033[1;38;5;51m[Press SPACE to scroll, 'q' to quit, '/' to search]\033[0m"
    echo ""
  } | less -R
  
  glow "$file"
}

# Suffix alias: execute .md files directly to render them
alias -s md='glow'

# md command: render one or all markdown files
md() {
  if [[ $# -gt 0 ]]; then
    glow "$@"
  else
    local files=(*.md(N))
    if (( ${#files} )); then
      glow *.md
    else
      echo "No markdown files in current directory" >&2
      return 1
    fi
  fi
}

# === Markdown Browser (Esc key) ===
_md_browser_widget() {
  # Collect markdown files
  local files=(*.md(N))
  
  # If no markdown files, do nothing
  if (( ${#files} == 0 )); then
    return 0
  fi

  # fzf with wide preview (80% preview window, keyboard only)
  # Use TERM=xterm-256color to force glow to output colors
  local file
  file=$(printf '%s\n' "${files[@]}" | \
    TERM=xterm-256color fzf --no-mouse \
        --ansi \
        --preview 'TERM=xterm-256color command glow -s dark {}' \
        --preview-window=right:80%:wrap \
        --height=100% \
        --border=rounded \
        --color='fg:#ffffff,bg:#000000,hl:#ff00f8,fg+:#000000,bg+:#ff00f8,hl+:#5cecff,info:#5cecff,border:#ff00f8,prompt:#ffb1fe,pointer:#5cecff,marker:#ff00f8,spinner:#5cecff,header:#aa00e8') || {
    zle redisplay
    return 0
  }
  
  [[ -n $file ]] && glow "$file"
  zle redisplay
  return 0
}

zle -N _md_browser_widget
# Bind to double-tap Esc
bindkey '\e\e' _md_browser_widget

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

# On empty line, complete commands from PATH + files in cwd
zstyle ':completion:*' insert-tab false
zstyle ':completion:::::' completer _complete _approximate

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

# TAB completion uses default expand-or-complete (no custom binding needed)

# fzf keybindings
if command -v fzf &>/dev/null; then
  # Ctrl+T: Fuzzy file search with smart preview (bat for text, xxd for binaries)
  export FZF_CTRL_T_COMMAND='fd --type f --hidden --follow --exclude .git'
  export FZF_CTRL_T_OPTS="--preview 'if file -b --mime {} | grep -q text; then bat --color=always --style=numbers --line-range=:500 {}; else xxd -l 512 {}; fi' --preview-window=right:60%:wrap --color='fg:#ffffff,bg:#000000,hl:#ff00f8,fg+:#000000,bg+:#ff00f8,hl+:#5cecff,info:#5cecff,border:#ff00f8,prompt:#ffb1fe,pointer:#5cecff,marker:#ff00f8,spinner:#5cecff,header:#aa00e8'"
  
  # Alt+C: Fuzzy cd into subdirectories
  export FZF_ALT_C_COMMAND='fd --type d --hidden --follow --exclude .git'
  export FZF_ALT_C_OPTS="--preview 'eza --tree --level=1 --color=always {}' --color='fg:#ffffff,bg:#000000,hl:#ff00f8,fg+:#000000,bg+:#ff00f8,hl+:#5cecff,info:#5cecff,border:#ff00f8,prompt:#ffb1fe,pointer:#5cecff,marker:#ff00f8,spinner:#5cecff,header:#aa00e8'"
  
  # Load fzf keybindings
  [ -f ~/.fzf.zsh ] && source ~/.fzf.zsh
  [[ -f /opt/homebrew/opt/fzf/shell/key-bindings.zsh ]] && source /opt/homebrew/opt/fzf/shell/key-bindings.zsh
fi

export HOMEBREW_NO_ENV_HINTS="true"

# === Syntax Highlighting (MUST BE LAST) ===
for f in \
  /opt/homebrew/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh \
  /usr/local/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh \
  /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh; do
  [[ -f "$f" ]] && source "$f" && break
done

# Syntax highlighting colors (vaporwave)
if [[ -n "$ZSH_HIGHLIGHT_STYLES" ]]; then
  ZSH_HIGHLIGHT_STYLES[command]='fg=#5cecff,bold'
  ZSH_HIGHLIGHT_STYLES[alias]='fg=#ffb1fe,bold'
  ZSH_HIGHLIGHT_STYLES[builtin]='fg=#ff00f8,bold'
  ZSH_HIGHLIGHT_STYLES[function]='fg=#02c3fc,bold'
  ZSH_HIGHLIGHT_STYLES[arg0]='fg=#5cecff'
  ZSH_HIGHLIGHT_STYLES[path]='fg=#fbb725,underline'
  ZSH_HIGHLIGHT_STYLES[single-quoted-argument]='fg=#fbb725'
  ZSH_HIGHLIGHT_STYLES[double-quoted-argument]='fg=#fbb725'
fi
