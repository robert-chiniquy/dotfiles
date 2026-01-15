. ~/.zprofile

# Source secrets (tokens, API keys - not in git)
[[ -f ~/.secrets ]] && source ~/.secrets

# === Starship prompt (must be in .zshrc, not .zprofile - needs zle) ===
eval "$(starship init zsh)"

# === Persist last working directory (for new tabs/sessions) ===
# New interactive shells start in the last directory you used.
__LAST_DIR_FILE="$HOME/.zsh_last_dir"
if [[ -o interactive && "$PWD" == "$HOME" && -r "$__LAST_DIR_FILE" ]]; then
  __last_dir="$(<"$__LAST_DIR_FILE")"
  [[ -d "$__last_dir" ]] && builtin cd -- "$__last_dir"
  unset __last_dir
fi

# Force horizontal banner output on macOS
if command -v gbanner >/dev/null 2>&1; then
  alias banner='gbanner'
fi

# === Shell Options ===
# Better directory navigation
setopt auto_cd              # cd by typing just directory name
setopt auto_pushd           # Push old directory onto stack
setopt pushd_ignore_dups    # Don't push duplicates
setopt pushd_silent         # Don't print stack after pushd
alias d='dirs -v'           # Show directory stack

# Enhanced globbing
setopt extended_glob        # Enable powerful globbing (^, ~, #)
setopt glob_dots            # Include hidden files in globs
setopt null_glob            # Don't error on no matches

# Better history
HISTFILE=~/.zsh_history
HISTSIZE=50000
SAVEHIST=50000
setopt extended_history     # Save timestamp and duration
setopt hist_ignore_dups     # Don't save duplicates
setopt hist_ignore_space    # Ignore commands starting with space
setopt hist_reduce_blanks   # Clean up whitespace
setopt share_history        # Share between terminals instantly
# Note: share_history = intermixed across terminals. Alternative: per-session with
# HISTFILE=~/.zsh_history_$$ but that fragments history and breaks cross-session recall.
# Intermixed is better for "what command did I run yesterday" use cases.

# === Vaporwave FZF Colors ===
export FZF_COLORS='fg:#ffffff,bg:#000000,hl:#ff00f8,fg+:#000000,bg+:#ff00f8,hl+:#5cecff,info:#5cecff,border:#ff00f8,prompt:#ffb1fe,pointer:#5cecff,marker:#ff00f8,spinner:#5cecff,header:#aa00e8'

# Job control
setopt no_notify            # Don't report background job status immediately
setopt no_bg_nice           # Don't nice background jobs

# === Modern CLI Tools ===
# eza (better ls)
if command -v eza &>/dev/null; then
  alias ls='eza --icons --color=always --group-directories-first'
  alias ll='eza --icons --color=always --group-directories-first -l'
  alias la='eza --icons --color=always --group-directories-first -la'
  alias lt='eza --icons --color=always --tree --level=2'
  alias lsg='eza --icons --color=always --group-directories-first --git -l'
  
  # Vaporwave colors for eza - retina-searing electric colors
  export EZA_COLORS="da=38;5;51:di=38;5;51:ex=38;5;201:*.md=38;5;171:*.json=38;5;221:*.yaml=38;5;51:*.go=38;5;51:*.js=38;5;221:*.ts=38;5;51"
fi

# zoxide (smart cd)
if command -v zoxide &>/dev/null; then
  eval "$(zoxide init zsh)"
fi

# bat (better cat)
if command -v bat &>/dev/null; then
  alias cat='bat --style=plain --paging=never'
  alias bcat='bat --style=full'
  # Theme set in ~/.config/bat/config
fi

# ripgrep with colors
if command -v rg &>/dev/null; then
  export RIPGREP_CONFIG_PATH="$HOME/.ripgreprc"
fi

# direnv
if command -v direnv &>/dev/null; then
  eval "$(direnv hook zsh)"
fi

# git-delta configured in .gitconfig [core] pager = delta

# === Colorized Environment ===
# Colorize grep output
export GREP_COLORS='ms=01;38;5;201:mc=01;38;5;51:sl=:cx=:fn=38;5;221:ln=38;5;51:bn=38;5;51:se=38;5;201'

# Vaporwave man pages and less output
export LESS_TERMCAP_mb=$'\e[1;38;5;201m'      # begin bold (hot pink)
export LESS_TERMCAP_md=$'\e[1;38;5;51m'       # begin blink (cyan)
export LESS_TERMCAP_me=$'\e[0m'               # reset bold/blink
export LESS_TERMCAP_so=$'\e[1;38;5;15;48;5;201m'  # begin reverse video (white on hot pink)
export LESS_TERMCAP_se=$'\e[0m'               # reset reverse video
export LESS_TERMCAP_us=$'\e[1;38;5;221m'      # begin underline (gold)
export LESS_TERMCAP_ue=$'\e[0m'               # reset underline

# JSON/YAML viewers
alias jqc='jq -C | less -R'
alias yqc='yq -C | less -R'

# === Markdown Rendering Defaults ===
# glow wrapper: pager mode + vaporwave theme + TAB to follow local .md links
# Sets _GLOW_VIEWED_FILES array with fully qualified paths of all viewed files
glow() {
  local file="$1"
  local current_dir
  
  # Reset viewed files tracking
  typeset -ga _GLOW_VIEWED_FILES=()
  
  # If no file or multiple files, just run glow normally
  if [[ -z "$file" || $# -gt 1 || ! -f "$file" ]]; then
    command glow -p -s ~/.config/glow/vaporwave.json "$@"
    return
  fi
  
  # Track the base directory for resolving relative links
  current_dir="$(dirname "$(realpath "$file")")"
  
  while true; do
    # Track this file as viewed
    _GLOW_VIEWED_FILES+=("$(realpath "$file")")
    
    # Show the file
    command glow -p -s ~/.config/glow/vaporwave.json "$file"
    
    # Extract local .md links from the file
    # Matches: [text](path.md) or [text](path.md#anchor) or [text](./path.md)
    local -a links=()
    local link target
    while IFS= read -r link; do
      # Remove anchor fragments
      target="${link%%#*}"
      # Resolve relative to current file's directory
      if [[ "$target" == /* ]]; then
        # Absolute path
        [[ -f "$target" ]] && links+=("$target")
      else
        # Relative path
        [[ -f "$current_dir/$target" ]] && links+=("$current_dir/$target")
      fi
    done < <(grep -oE '\[[^]]*\]\([^)]+\.md[^)]*\)' "$file" 2>/dev/null | \
             sed -E 's/.*\]\(([^)]+)\)/\1/' | sort -u)
    
    # No links found, exit
    (( ${#links} == 0 )) && break
    
    # Deduplicate
    links=(${(u)links})
    
    # Show fzf picker with TAB hint
    local selected
    selected=$(printf '%s\n' "${links[@]}" | \
      fzf --no-mouse \
          --prompt='TAB found links > ' \
          --header='Select linked file (ESC to exit)' \
          --preview 'bat --color=always --style=plain --theme=vaporwave-custom --language=md {}' \
          --preview-window=right:65%:wrap \
          --height=100% \
          --border=rounded \
          --color="$FZF_COLORS") || break
    
    # If user selected a file, loop to show it
    [[ -n "$selected" ]] && file="$selected" && current_dir="$(dirname "$(realpath "$file")")" || break
  done
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
  local -a all_files=()
  local -a viewed_files=()
  local min_files=20
  local max_files=60
  
  # Directories to always skip (dependencies, build artifacts, caches)
  local -a skip_dirs=(node_modules vendor .cache __pycache__ dist build .git 
                      .venv venv env .tox .pytest_cache .mypy_cache coverage
                      .next .nuxt .output target pkg mod cache plugin plugins)
  local skip_pattern="(${(j:|:)skip_dirs})"
  
  # 1. Start with current directory markdown files (D = include dotfiles)
  all_files=(*.md(ND))

  # If no markdown files anywhere, do nothing
  if (( ${#all_files} == 0 )); then
    # Try one level of subdirs before giving up (excluding skip dirs)
    # Include hidden directories with explicit .*/
    local -a subdir_files=()
    for f in */**.md(ND) .*/**.md(ND); do
      [[ "$f" == */${~skip_pattern}/* ]] && continue
      subdir_files+=("$f")
    done
    if (( ${#subdir_files} == 0 )); then
      return 0
    fi
  fi

  # 2. If under threshold, cascade into subdirectories
  if (( ${#all_files} < min_files )); then
    local -a priority_dirs=() other_dirs=()
    local dir

    # Categorize subdirectories (prioritize docs-like names)
    # Include hidden directories with explicit .*/ pattern
    for dir in */(/ND) .*/(/ND); do
      dir="${dir%/}"
      # Skip dependency/build directories
      [[ "${dir:l}" == ${~skip_pattern} ]] && continue
      case "${dir:l}" in
        doc|docs|documentation|readme|readmes|guide|guides|manual|manuals|wiki)
          priority_dirs+=("$dir")
          ;;
        *)
          other_dirs+=("$dir")
          ;;
      esac
    done
    
    # Add files from priority directories first
    local subfiles
    for dir in "${priority_dirs[@]}" "${other_dirs[@]}"; do
      (( ${#all_files} >= max_files )) && break
      subfiles=("$dir"/*.md(ND))
      if (( ${#subfiles} > 0 )); then
        local remaining=$(( max_files - ${#all_files} ))
        all_files+=("${subfiles[@]:0:$remaining}")
      fi
    done

    # If still under threshold, go deeper (2 levels), still filtering
    if (( ${#all_files} < min_files )); then
      for dir in "${priority_dirs[@]}" "${other_dirs[@]}"; do
        (( ${#all_files} >= max_files )) && break
        for f in "$dir"/**/*.md(ND); do
          (( ${#all_files} >= max_files )) && break
          # Skip dependency directories
          [[ "$f" == */${~skip_pattern}/* ]] && continue
          # Skip if already added
          [[ " ${all_files[*]} " == *" $f "* ]] && continue
          all_files+=("$f")
        done
      done
    fi
  fi
  
  # If still no files, give up
  (( ${#all_files} == 0 )) && return 0

  # fzf with wide preview (80% preview window, keyboard only)
  # Use TERM=xterm-256color to force glow to output colors
  local file
  file=$(printf '%s\n' "${all_files[@]}" | \
    TERM=xterm-256color fzf --no-mouse \
        --ansi \
        --preview 'bat --color=always --style=plain --theme=vaporwave-custom --language=md {}' \
        --preview-window=right:75%:wrap \
        --height=100% \
        --border=rounded \
        --color="$FZF_COLORS") || {
    zle redisplay
    return 0
  }
  
  # View the selected file (glow tracks viewed files in _GLOW_VIEWED_FILES)
  if [[ -n $file ]]; then
    # Add the initially selected file
    viewed_files+=("$(realpath "$file")")
    glow "$file"
    # Add any files viewed via link-following
    viewed_files+=("${_GLOW_VIEWED_FILES[@]}")
  fi
  
  # Deduplicate and print viewed files
  if (( ${#viewed_files} > 0 )); then
    viewed_files=(${(u)viewed_files})
    # Clear line and print files (ensures visibility after glow's terminal manipulation)
    print ""
    print -l "${viewed_files[@]}"
    print ""
  fi
  zle reset-prompt
  return 0
}

# Force emacs keybindings (prevent EDITOR=vim from triggering vi mode)
bindkey -e

zle -N _md_browser_widget
# Bind to double-tap Esc - KEYTIMEOUT=10 (0.1s) requires fast double-tap
KEYTIMEOUT=10
bindkey '\e\e' _md_browser_widget
bindkey -M emacs '\e\e' _md_browser_widget
bindkey -M viins '\e\e' _md_browser_widget
bindkey -M vicmd '\e\e' _md_browser_widget

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

# Better completion caching with 1-hour expiry
zstyle ':completion:*' use-cache on
zstyle ':completion:*' cache-path ~/.zsh/cache

# Cache policy: invalidate after 1 hour
_cache_policy_1h() {
  [[ -z "$1" ]] && return 0
  [[ ! -f "$1" ]] && return 0
  local -a stat
  stat=("${(@f)$(stat -f '%m' "$1" 2>/dev/null)}")
  (( $(date +%s) - stat[1] > 3600 ))
}
zstyle ':completion:*' cache-policy _cache_policy_1h

# Force file completions to verify existence
zstyle ':completion:*' file-patterns '%p:globbed-files' '*(-/):directories'

# Colorful completion descriptions
zstyle ':completion:*:descriptions' format '%B%F{201}%d%f%b'
zstyle ':completion:*:warnings' format '%F{red}No matches found%f'
zstyle ':completion:*' group-name ''

# === Autosuggestions (AFTER compinit) ===
export ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE="fg=#190319,bg=#ffb1fe,bold"
export ZSH_AUTOSUGGEST_STRATEGY=(history)  # Only history, not completion (avoids TAB flip-flop)

for f in \
  /opt/homebrew/share/zsh-autosuggestions/zsh-autosuggestions.zsh \
  /usr/local/share/zsh-autosuggestions/zsh-autosuggestions.zsh \
  /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh; do
  [[ -f "$f" ]] && source "$f" && break
done

# === Key Bindings ===
# Smart TAB: accept autosuggestion OR trigger completion
# - If autosuggestion visible: TAB accepts it
# - If completion menu open: TAB accepts selection (via menuselect keymap)
# - Otherwise: TAB triggers completion
zmodload zsh/complist

_smart_tab() {
  if [[ -n "$POSTDISPLAY" ]]; then
    zle autosuggest-accept
  else
    zle expand-or-complete
  fi
}
zle -N _smart_tab
bindkey '^I' _smart_tab

# Right arrow: move cursor, but accept suggestion if at end of line
_smart_right() {
  if [[ $CURSOR -eq ${#BUFFER} && -n "$POSTDISPLAY" ]]; then
    zle autosuggest-accept
  else
    zle forward-char
  fi
}
zle -N _smart_right
bindkey '^[[C' _smart_right
bindkey '^[OC' _smart_right    # SS3 variant (Ghostty sends this)

# Down arrow: open completion menu, else history
_smart_down() {
  zle autosuggest-clear 2>/dev/null
  zle menu-complete
}
zle -N _smart_down
bindkey '^[[B' _smart_down
bindkey '^[OB' _smart_down     # SS3 variant (Ghostty sends this)

# Up arrow: history search
bindkey '^[[A' up-line-or-history
bindkey '^[OA' up-line-or-history  # SS3 variant (Ghostty sends this)

# Left arrow: move cursor (explicit binding for both sequence types)
bindkey '^[[D' backward-char
bindkey '^[OD' backward-char   # SS3 variant (Ghostty sends this)

# In menu: TAB accepts, arrows navigate, ESC exits
bindkey -M menuselect '^I' accept-line
bindkey -M menuselect '^[[A' up-line-or-history
bindkey -M menuselect '^[OA' up-line-or-history   # SS3 variant
bindkey -M menuselect '^[[B' down-line-or-history
bindkey -M menuselect '^[OB' down-line-or-history # SS3 variant
bindkey -M menuselect '^[[D' backward-char
bindkey -M menuselect '^[OD' backward-char        # SS3 variant
bindkey -M menuselect '^[[C' forward-char
bindkey -M menuselect '^[OC' forward-char         # SS3 variant
bindkey -M menuselect '\e' send-break
bindkey -M menuselect '^C' send-break

# Ctrl+P: Fuzzy history search (complements atuin)
_fuzzy_history_widget() {
  local selected
  selected=$(fc -rl 1 | fzf --height=40% --prompt="History: " --tac --no-sort \
    --color="$FZF_COLORS" | \
    sed 's/^[ ]*[0-9]*[ ]*//')
  
  if [[ -n "$selected" ]]; then
    BUFFER="$selected"
    CURSOR=$#BUFFER
  fi
  zle redisplay
}
zle -N _fuzzy_history_widget
bindkey '^P' _fuzzy_history_widget

# fzf keybindings
if command -v fzf &>/dev/null; then
  # Ctrl+T: Fuzzy file search with smart preview (bat for text, xxd for binaries)
  export FZF_CTRL_T_COMMAND='fd --type f --hidden --follow --exclude .git --exclude cache --exclude plugin --exclude plugins'
  export FZF_CTRL_T_OPTS="--preview 'if file -b --mime {} | grep -q text; then bat --color=always --style=numbers --line-range=:500 {}; else xxd -l 512 {}; fi' --preview-window=right:60%:wrap --color=$FZF_COLORS"
  
  # Alt+C: Fuzzy cd into subdirectories
  export FZF_ALT_C_COMMAND='fd --type d --hidden --follow --exclude .git --exclude cache --exclude plugin --exclude plugins'
  export FZF_ALT_C_OPTS="--preview 'eza --tree --level=1 --color=always {}' --color=$FZF_COLORS"
  
  # Load fzf keybindings
  [ -f ~/.fzf.zsh ] && source ~/.fzf.zsh
  [[ -f /opt/homebrew/opt/fzf/shell/key-bindings.zsh ]] && source /opt/homebrew/opt/fzf/shell/key-bindings.zsh
fi

export HOMEBREW_NO_ENV_HINTS="true"

# === Atuin (after keybindings so it gets final say on up-arrow) ===
command -v atuin &>/dev/null && eval "$(atuin init zsh)"

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

# === Auto-ls after cd ===
# Automatically show tree after changing directory
chpwd() {
  emulate -L zsh

  # Persist last directory for new tabs/sessions
  [[ -n "$__LAST_DIR_FILE" ]] && print -r -- "$PWD" >| "$__LAST_DIR_FILE" 2>/dev/null
  
  # Show directory contents as tree if small enough
  local item_count=$(ls -1A 2>/dev/null | wc -l | tr -d ' ')
  if [[ $item_count -le 30 ]]; then
    eza --tree --level=1 --icons --git --color=always --group-directories-first 2>/dev/null || ls
  fi
}

# === Command Duration Display ===
# Show duration when complete, update iTerm2 badge
preexec() {
  _command_start_time=$SECONDS
  __last_cmd="$1"  # Track command for history cleanup
}

# Track failed commands for similarity-based history cleanup
typeset -ga __failed_cmds=()

# Similarity check: same command with minor arg differences
__cmds_similar() {
  local succ="$1" fail="$2"
  
  # Exact match (retry worked)
  [[ "$succ" == "$fail" ]] && return 0
  
  # Same base command (first word)
  local succ_cmd="${succ%% *}" fail_cmd="${fail%% *}"
  [[ "$succ_cmd" != "$fail_cmd" ]] && return 1
  
  # Length difference > 30% = not similar
  local -i len_s=${#succ} len_f=${#fail}
  local -i max_len=$(( len_s > len_f ? len_s : len_f ))
  local -i min_len=$(( len_s < len_f ? len_s : len_f ))
  (( (max_len - min_len) * 100 / max_len > 30 )) && return 1
  
  # Compare char by char (simple diff count)
  local -i diffs=0 i
  for (( i=0; i<min_len; i++ )); do
    [[ "${succ:$i:1}" != "${fail:$i:1}" ]] && (( diffs++ ))
  done
  (( diffs += max_len - min_len ))
  
  # Similar if <20% different
  (( diffs * 100 / max_len < 20 ))
}

precmd() {
  local __last_exit=$?
  
  # History cleanup: remove similar failed commands when a command succeeds
  if [[ -n "$__last_cmd" ]]; then
    if (( __last_exit != 0 )); then
      # Command failed - remember it
      __failed_cmds+=("$__last_cmd")
      # Keep only last 10 failed commands
      (( ${#__failed_cmds} > 10 )) && __failed_cmds=("${__failed_cmds[@]: -10}")
    elif (( ${#__failed_cmds} > 0 )); then
      # Command succeeded - check for similar failed commands to remove
      local failed
      for failed in "${__failed_cmds[@]}"; do
        if __cmds_similar "$__last_cmd" "$failed"; then
          # Remove the failed command from history file
          local escaped="${failed//\//\\/}"
          escaped="${escaped//\[/\\[}"
          escaped="${escaped//\]/\\]}"
          sed -i '' "/;${escaped}$/d" "$HISTFILE" 2>/dev/null
        fi
      done
      __failed_cmds=()  # clear after success
    fi
  fi
  unset __last_cmd

  # Show command duration
  if [[ -n $_command_start_time ]]; then
    local elapsed=$(( SECONDS - _command_start_time ))
    if (( elapsed >= 3 )); then
      echo -e "\033[38;5;221m⏱  ${elapsed}s\033[0m"
    fi
  fi
  unset _command_start_time
  
  # Update iTerm2 badge with git info (shows all changed files up to 10)
  if git rev-parse --git-dir &>/dev/null 2>&1; then
    local branch=$(git branch --show-current 2>/dev/null)
    local count=$(git status -s 2>/dev/null | wc -l | tr -d ' ')
    
    if [[ -n "$branch" ]]; then
      local badge_text="⎇ $branch"
      
      # Check if there are changes
      if [[ "$count" -gt 0 ]]; then
        local files=$(git status -s 2>/dev/null | head -10)
        badge_text="$badge_text\n"
        badge_text="$badge_text$(echo "$files" | awk '{print "  " $2 " " $1}')"
        
        # Show "and N more" if more than 10
        if (( count > 10 )); then
          badge_text="$badge_text\n  ... and $((count - 10)) more"
        fi
      else
        # No changes - show skull and crossbones
        badge_text="$badge_text\n\n☠☠☠☠☠\n☠☠☠☠☠\n☠☠☠☠☠"
      fi
      
      printf "\e]1337;SetBadgeFormat=%s\a" $(echo -n "$badge_text" | base64)
    fi
  else
    # Not in a git repo - show skulls
    printf "\e]1337;SetBadgeFormat=%s\a" $(echo -n "☠☠☠☠☠\n☠☠☠☠☠\n☠☠☠☠☠" | base64)
  fi
}

# opencode
export PATH=/Users/rch/.opencode/bin:$PATH
