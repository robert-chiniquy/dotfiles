. ~/.zprofile

# Silence direnv (must be before hook)
export DIRENV_LOG_FORMAT=

# Source secrets (tokens, API keys - not in git)
[[ -f ~/.secrets ]] && source ~/.secrets

# === Greeting (only on first shell, not subshells) ===
if [[ -z "$_GREETED" && -o interactive ]]; then
  export _GREETED=1

  if command -v fastfetch &>/dev/null && (( ${COLUMNS:-$(tput cols)} >= 120 )) && (( ${LINES:-$(tput lines)} >= 30 )); then
    [[ -x ~/.config/fastfetch/gen-logo.sh ]] && ~/.config/fastfetch/gen-logo.sh > ~/.config/fastfetch/logo.txt 2>/dev/null
    fastfetch
  fi

  # Daily brew outdated check (runs in background, shows next prompt)
  _brew_marker=~/.cache/brew-outdated-check
  mkdir -p ~/.cache
  if [[ ! -f "$_brew_marker" ]] || [[ $(find "$_brew_marker" -mtime +1 2>/dev/null) ]]; then
    touch "$_brew_marker"
    (
      outdated=$(brew outdated --quiet 2>/dev/null | wc -l | tr -d ' ')
      if (( outdated > 0 )); then
        echo "$outdated" > "$_brew_marker"
      else
        echo "0" > "$_brew_marker"
      fi
    ) &!
  elif [[ -f "$_brew_marker" ]]; then
    _brew_count=$(cat "$_brew_marker" 2>/dev/null)
    (( _brew_count > 0 )) && print -P "%F{243}$_brew_count outdated brew packages%f"
  fi

  # GitHub PR status check (hourly, only if something to report)
  _pr_marker=~/.cache/gh-pr-status
  if command -v gh &>/dev/null && { [[ ! -f "$_pr_marker" ]] || [[ $(find "$_pr_marker" -mmin +60 2>/dev/null) ]]; }; then
    touch "$_pr_marker"
    (
      # Check for PRs with activity in last hour
      pr_data=$(gh pr list --author @me --state open --json number,title,updatedAt,reviewDecision,statusCheckRollup,comments 2>/dev/null)
      [[ -z "$pr_data" || "$pr_data" == "[]" ]] && exit 0

      # Parse with jq - find PRs updated in last hour or with failures
      cutoff=$(date -v-1H +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -d '1 hour ago' --iso-8601=seconds 2>/dev/null)

      output=""

      # PRs with CI failures
      failures=$(echo "$pr_data" | jq -r --arg cutoff "$cutoff" '
        .[] | select(.statusCheckRollup != null) |
        select([.statusCheckRollup[] | select(.conclusion == "FAILURE")] | length > 0) |
        "#\(.number) \(.title | .[0:40])"' 2>/dev/null)
      [[ -n "$failures" ]] && output="${output}CI failures:\n$failures\n"

      # PRs with changes requested
      changes=$(echo "$pr_data" | jq -r '
        .[] | select(.reviewDecision == "CHANGES_REQUESTED") |
        "#\(.number) \(.title | .[0:40])"' 2>/dev/null)
      [[ -n "$changes" ]] && output="${output}Changes requested:\n$changes\n"

      # PRs updated in last hour (new comments/reviews)
      recent=$(echo "$pr_data" | jq -r --arg cutoff "$cutoff" '
        .[] | select(.updatedAt > $cutoff) |
        select(.reviewDecision != "CHANGES_REQUESTED") |
        select([.statusCheckRollup // [] | .[] | select(.conclusion == "FAILURE")] | length == 0) |
        "#\(.number) updated"' 2>/dev/null)
      [[ -n "$recent" ]] && output="${output}Recent activity:\n$recent\n"

      # PRs ready to merge (approved + all checks pass)
      ready=$(echo "$pr_data" | jq -r '
        .[] | select(.reviewDecision == "APPROVED") |
        select([.statusCheckRollup // [] | .[] | select(.conclusion != "SUCCESS" and .conclusion != "SKIPPED" and .conclusion != null)] | length == 0) |
        "#\(.number) ready to merge"' 2>/dev/null)
      [[ -n "$ready" ]] && output="${output}Ready to merge:\n$ready\n"

      [[ -n "$output" ]] && echo "$output" > "$_pr_marker.data"
    ) &!
  fi
  if [[ -f "$_pr_marker.data" ]]; then
    _pr_status=$(cat "$_pr_marker.data" 2>/dev/null)
    if [[ -n "$_pr_status" ]]; then
      print -P "%F{201}PRs:%f"
      echo "$_pr_status" | while IFS= read -r line; do
        [[ -n "$line" ]] && print -P "  %F{243}$line%f"
      done
      rm -f "$_pr_marker.data"
    fi
  fi

  # Next calendar event (cached for 15 min)
  _cal_marker=~/.cache/next-event
  if [[ ! -f "$_cal_marker" ]] || [[ $(find "$_cal_marker" -mmin +15 2>/dev/null) ]]; then
    (
      event=$(osascript -e '
        set now to current date
        set later to now + (2 * 60 * 60)
        tell application "Calendar"
          set allEvents to {}
          repeat with c in calendars
            set allEvents to allEvents & (every event of c whose start date >= now and start date <= later)
          end repeat
          if (count of allEvents) > 0 then
            set nextEvent to item 1 of allEvents
            set minStart to start date of nextEvent
            repeat with e in allEvents
              if start date of e < minStart then
                set nextEvent to e
                set minStart to start date of e
              end if
            end repeat
            set mins to round ((minStart - now) / 60)
            return (summary of nextEvent) & " in " & mins & "m"
          end if
        end tell
      ' 2>/dev/null)
      echo "$event" > "$_cal_marker"
    ) &!
  fi
  _next_event=$(cat "$_cal_marker" 2>/dev/null)
  [[ -n "$_next_event" ]] && print -P "%F{243}$_next_event%f"
fi


# === Starship prompt (must be in .zshrc, not .zprofile - needs zle) ===
eval "$(starship init zsh)"

# === Terminal title: dir + git branch ===
_set_terminal_title() {
  local title="${PWD/#$HOME/~}"
  if git rev-parse --git-dir &>/dev/null 2>&1; then
    local branch=$(git branch --show-current 2>/dev/null)
    [[ -n "$branch" ]] && title="$title [$branch]"
  fi
  print -Pn "\e]0;$title\a"
}
precmd_functions+=(_set_terminal_title)

# === Background git fetch on repo entry ===
_auto_git_fetch() {
  if git rev-parse --git-dir &>/dev/null 2>&1; then
    local git_root=$(git rev-parse --show-toplevel 2>/dev/null)
    local fetch_marker="$git_root/.git/FETCH_HEAD"
    # Only fetch if FETCH_HEAD is older than 10 minutes
    if [[ ! -f "$fetch_marker" ]] || [[ $(find "$fetch_marker" -mmin +10 2>/dev/null) ]]; then
      (git fetch --quiet --prune &) 2>/dev/null
    fi
  fi
}
chpwd_functions+=(_auto_git_fetch)

# === Show uncommitted changes age on repo entry ===
_show_uncommitted_age() {
  git rev-parse --git-dir &>/dev/null 2>&1 || return
  local changed_files=$(git status --porcelain 2>/dev/null | head -1)
  [[ -z "$changed_files" ]] && return
  # Get oldest modified tracked file's age
  local oldest_mtime=$(git status --porcelain 2>/dev/null | awk '{print $2}' | head -20 | xargs -I{} stat -f %m {} 2>/dev/null | sort -n | head -1)
  [[ -z "$oldest_mtime" ]] && return
  local now=$(date +%s)
  local age=$((now - oldest_mtime))
  local age_str
  if (( age < 3600 )); then
    age_str="$((age / 60))m ago"
  elif (( age < 86400 )); then
    age_str="$((age / 3600))h ago"
  else
    age_str="$((age / 86400))d ago"
  fi
  print -P "%F{243}uncommitted changes from $age_str%f"
}
chpwd_functions+=(_show_uncommitted_age)

# === Show ahead/behind remote on repo entry ===
_show_ahead_behind() {
  git rev-parse --git-dir &>/dev/null 2>&1 || return
  local upstream=$(git rev-parse --abbrev-ref @{upstream} 2>/dev/null) || return
  local ahead=$(git rev-list --count @{upstream}..HEAD 2>/dev/null)
  local behind=$(git rev-list --count HEAD..@{upstream} 2>/dev/null)
  local output=""
  (( ahead > 0 )) && output="$ahead ahead"
  (( behind > 0 )) && output="${output:+$output, }$behind behind"
  [[ -n "$output" ]] && print -P "%F{243}$output $upstream%f"
}
chpwd_functions+=(_show_ahead_behind)

# === Show last commit age on repo entry ===
_show_last_commit_age() {
  git rev-parse --git-dir &>/dev/null 2>&1 || return
  local age=$(git log -1 --format=%cr 2>/dev/null)
  [[ -n "$age" ]] && print -P "%F{243}last commit $age%f"
}
chpwd_functions+=(_show_last_commit_age)

# === Show branch age on repo entry ===
_show_branch_age() {
  git rev-parse --git-dir &>/dev/null 2>&1 || return
  local branch=$(git branch --show-current 2>/dev/null)
  [[ -z "$branch" || "$branch" == "main" || "$branch" == "master" ]] && return
  # Find merge-base with main or master
  local base=$(git merge-base main HEAD 2>/dev/null || git merge-base master HEAD 2>/dev/null) || return
  local age=$(git log -1 --format=%cr "$base" 2>/dev/null)
  [[ -n "$age" ]] && print -P "%F{243}branch started $age%f"
}
chpwd_functions+=(_show_branch_age)

# === Claude project files display on directory entry ===
_show_project_files() {
  local project_files=(
    DATA_SOURCES.md LEARNINGS.md HUMAN_TODOS.md
    FAILURES.md DEMO.md REMAINING_TODOS.md PROJECT.md GLOSSARY.md
  )
  local found=()
  local f lines w=$COLUMNS

  # Check for project files
  for f in "${project_files[@]}"; do
    [[ -f "$f" ]] && found+=("$f")
  done

  # Also check for PLAN_*.md files
  for f in PLAN_*.md(N); do
    [[ -f "$f" ]] && found+=("$f")
  done

  # If no project files, skip
  (( ${#found} == 0 )) && return

  # Narrow terminal: compact single-line summary
  if (( w < 50 )); then
    print -P "%F{201}${#found} project files%f"
    return
  fi

  # Medium terminal: filename with line count (one per line)
  if (( w < 80 )); then
    for f in "${found[@]}"; do
      lines=$(wc -l < "$f" | tr -d ' ')
      print -P "%F{51}${f%.md}%f%F{243}:${lines}%f"
    done
    return
  fi

  # Wide terminal: full display
  print -P "%F{201}project:%f"
  for f in "${found[@]}"; do
    lines=$(wc -l < "$f" | tr -d ' ')
    if git rev-parse --git-dir &>/dev/null 2>&1; then
      local stat=$(git diff --shortstat "$f" 2>/dev/null | awk '{printf "+%d/-%d", $4, $6}')
      [[ -n "$stat" && "$stat" != "+0/-0" ]] && stat="%F{221}$stat%f" || stat=""
      print -P "  %F{51}$f%f %F{243}${lines}L%f $stat"
    else
      print -P "  %F{51}$f%f %F{243}${lines}L%f"
    fi
  done

  # Parse DATA_SOURCES.md for local paths in same directory
  if [[ -f "DATA_SOURCES.md" ]]; then
    local sources=()
    while IFS= read -r line; do
      # Match lines like "- `./file.md`" or "- ./file.md" or "- file.md -"
      if [[ "$line" =~ ^-[[:space:]]+\`?\.?/?([^/\`[:space:]]+\.(md|txt|go|rs|py|ts|js))\`? ]]; then
        local src="${match[1]}"
        [[ -f "$src" && ! " ${found[*]} " =~ " $src " ]] && sources+=("$src")
      fi
    done < DATA_SOURCES.md

    if (( ${#sources} > 0 )); then
      print -P "%F{201}sources:%f"
      for f in "${sources[@]:0:5}"; do  # Limit to 5
        lines=$(wc -l < "$f" | tr -d ' ')
        print -P "  %F{221}$f%f %F{243}${lines}L%f"
      done
      (( ${#sources} > 5 )) && print -P "  %F{243}... +$((${#sources} - 5)) more%f"
    fi
  fi
}
chpwd_functions+=(_show_project_files)

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
setopt correct              # Suggest corrections for typos
# Note: share_history = intermixed across terminals. Alternative: per-session with
# HISTFILE=~/.zsh_history_$$ but that fragments history and breaks cross-session recall.
# Intermixed is better for "what command did I run yesterday" use cases.

# Styled correction prompt
SPROMPT='%F{221}%R%f -> %F{51}%r%f? [%F{201}y%f/%F{201}n%f/%F{201}a%f/%F{201}e%f] '

# === Vaporwave FZF Colors ===
export FZF_COLORS='fg:#ffffff,bg:#000000,hl:#ff00f8,fg+:#000000,bg+:#ff00f8,hl+:#5cecff,info:#5cecff,border:#ff00f8,prompt:#ffb1fe,pointer:#5cecff,marker:#ff00f8,spinner:#5cecff,header:#aa00e8'

# Job control
setopt no_notify            # Don't report background job status immediately
setopt no_bg_nice           # Don't nice background jobs
setopt interactive_comments # Allow comments in interactive shell
setopt no_flow_control      # Disable Ctrl+S/Ctrl+Q flow control (frees up keys)
setopt long_list_jobs       # More verbose job notifications
setopt no_clobber           # Don't overwrite files with > (use >| to force)

# === Modern CLI Tools ===
# eza (better ls)
if command -v eza &>/dev/null; then
  alias ls='eza --icons --color=always --group-directories-first'
  alias ll='eza --icons --color=always --group-directories-first -l'
  alias la='eza --icons --color=always --group-directories-first -la'
  alias lt='eza --icons --color=always --tree --level=2'
  alias lsg='eza --icons --color=always --group-directories-first --git -l'
  
  # Vaporwave colors for eza - retina-searing electric colors
  # 201=hot pink, 51=cyan, 221=gold, 171=light purple, 129=purple
  export EZA_COLORS="\
da=38;5;51:\
di=38;5;51;1:\
ex=38;5;201;1:\
ln=38;5;171:\
*.md=38;5;171:*.txt=38;5;171:*.rst=38;5;171:*.org=38;5;171:\
*.json=38;5;221:*.yaml=38;5;221:*.yml=38;5;221:*.xml=38;5;221:*.csv=38;5;221:\
*.go=38;5;51:*.rs=38;5;201:*.py=38;5;221:*.lua=38;5;51:*.ts=38;5;51:*.tsx=38;5;51:\
*.js=38;5;221:*.jsx=38;5;221:*.html=38;5;221:*.css=38;5;51:*.scss=38;5;51:\
*.sh=38;5;51:*.bash=38;5;51:*.zsh=38;5;51:*.fish=38;5;51:\
*.sql=38;5;221:*.graphql=38;5;171:\
*.toml=38;5;129:*.ini=38;5;129:*.conf=38;5;129:*.cfg=38;5;129:*.env=38;5;129:\
*.gitignore=38;5;129:*.dockerignore=38;5;129:*.editorconfig=38;5;129:\
Makefile=38;5;201:Dockerfile=38;5;201:Justfile=38;5;201:Cargo.toml=38;5;201:Cargo.lock=38;5;129:\
*.proto=38;5;51:*.pb.go=38;5;129:\
*.test.go=38;5;171:*_test.go=38;5;171:*.spec.ts=38;5;171:*.test.ts=38;5;171"
fi

# zoxide (smart cd) - replaces cd transparently
if command -v zoxide &>/dev/null; then
  eval "$(zoxide init zsh --cmd __z)"  # Use __z internally

  # Replace cd with zoxide-powered version
  cd() {
    if [[ $# -eq 0 ]]; then
      builtin cd ~
    elif [[ $1 == "-" ]]; then
      builtin cd -
    elif [[ -d $1 ]]; then
      # Exact directory exists, use it (and teach zoxide)
      __zoxide_z "$@"
    else
      # Let zoxide find it
      __zoxide_z "$@"
    fi
  }

  alias cdi='__zoxide_zi'  # Interactive selection
fi

# fd (better find) - integrations
if command -v fd &>/dev/null; then
  alias f='fd'
  alias fh='fd --hidden'
  alias ff='fd --type f'  # Files only
  alias fdir='fd --type d'  # Dirs only
  # fd + fzf: find file and open in editor
  fe() { fd --type f "$@" | fzf --preview 'bat --color=always {}' | xargs -r $EDITOR; }
  # fd + fzf: cd to directory
  fcd() { cd "$(fd --type d "$@" | fzf --preview 'eza --tree --level=1 --icons {}')" }
fi

# tokei (code stats)
if command -v tokei &>/dev/null; then
  alias loc='tokei'
  alias locs='tokei --sort code'  # Sort by lines of code
fi

# erdtree aliases
if command -v erd &>/dev/null; then
  alias tree='erd --icons --human'
  alias t='erd --icons --human --level 2'
  alias t1='erd --icons --human --level 1'
  alias t3='erd --icons --human --level 3'
  alias tsize='erd --icons --human --sort size --dir-order last'
fi

# zellij (terminal multiplexer)
if command -v zellij &>/dev/null; then
  alias zj='zellij'
  alias zja='zellij attach'
  alias zjl='zellij list-sessions'
fi

# bat (better cat)
if command -v bat &>/dev/null; then
  alias cat='bat --style=plain --paging=never'
  alias bcat='bat --style=full'
  alias batdiff='git diff --name-only | xargs bat --diff'  # Show changed files with diff markers
  # Help viewer with bat
  alias -g -- --help='--help 2>&1 | bat --language=help --style=plain'
  # Theme set in ~/.config/bat/config
fi

# rg + fzf: interactive grep with preview
if command -v rg &>/dev/null && command -v fzf &>/dev/null; then
  rgf() {
    rg --color=always --line-number --no-heading "$@" |
      fzf --ansi --delimiter : \
          --preview 'bat --color=always --highlight-line {2} {1}' \
          --preview-window 'up,60%,+{2}-10' \
          --color="$FZF_COLORS"
  }
fi

# neovim (alias vim to nvim)
if command -v nvim &>/dev/null; then
  alias vim='nvim'
  alias vi='nvim'
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
# Side-by-side diff when terminal is wide enough
alias gds='git diff --side-by-side'
alias gdss='git diff --staged --side-by-side'

# difftastic (structural diff via git diff external)
export DFT_BACKGROUND=dark
export DFT_COLOR=always

# Verbose mkdir (shows created path)
alias mkdir='mkdir -pv'

# Ping defaults (stop after 5, show stats)
alias ping='ping -c 5'

# wget resume by default
alias wget='wget -c'

# curl progress bar and follow redirects
alias curl='curl -L --progress-bar'

# === Colorized Environment ===
# Colorize grep output
export GREP_COLORS='ms=01;38;5;201:mc=01;38;5;51:sl=:cx=:fn=38;5;221:ln=38;5;51:bn=38;5;51:se=38;5;201'

# More verbose rm/mv/cp (shows what's happening)
alias rm='rm -v'
alias mv='mv -v'
alias cp='cp -v'

# Show human-readable sizes by default
alias df='df -h'
alias du='du -h'
alias free='top -l 1 -s 0 | grep PhysMem'

# Vaporwave LS_COLORS (used by erdtree, ls, etc.)
# 51=cyan, 221=gold, 129=purple, 183=soft orchid, 175=dusty rose
# Purple for code/important, pink for config/meta
export LS_COLORS="\
di=1;38;5;51:\
ln=38;5;129:\
so=38;5;129:\
pi=38;5;221:\
ex=1;38;5;129:\
bd=38;5;175:\
cd=38;5;175:\
su=38;5;129;48;5;51:\
sg=38;5;51;48;5;175:\
tw=38;5;51;48;5;175:\
ow=38;5;51:\
*.rs=1;38;5;129:\
*.go=38;5;51:\
*.py=38;5;221:\
*.js=38;5;221:\
*.ts=38;5;51:\
*.tsx=38;5;51:\
*.md=38;5;129:\
*.json=38;5;221:\
*.yaml=38;5;221:\
*.yml=38;5;221:\
*.toml=38;5;175:\
*.sh=38;5;51:\
*.zsh=38;5;51:\
*.lua=38;5;51:\
*.sql=38;5;221:\
*.html=38;5;221:\
*.css=38;5;51:\
*.proto=38;5;51:\
*.txt=38;5;129:\
*.log=38;5;243:\
*.git=1;38;5;175:\
*.gitignore=38;5;175:\
Makefile=1;38;5;129:\
Dockerfile=1;38;5;129:\
Cargo.toml=1;38;5;129:\
*.lock=38;5;175:\
*.tar=38;5;221:\
*.gz=38;5;221:\
*.zip=38;5;221:\
*.png=38;5;51:\
*.jpg=38;5;51:\
*.svg=38;5;51:\
*.mp4=38;5;129:\
*.mp3=38;5;129"

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

# mdwatch: watch directory for markdown changes and auto-display with changed line highlight
mdwatch() {
  if ! command -v fswatch &>/dev/null; then
    echo "fswatch not installed. Install with: brew install fswatch" >&2
    return 1
  fi

  local dir="${1:-.}"
  local cache_dir="/tmp/mdwatch-$$"
  local fifo="/tmp/mdwatch-fifo-$$"
  mkdir -p "$cache_dir"
  mkfifo "$fifo"

  # Cleanup on exit - kill fswatch and remove temp files
  cleanup() {
    pkill -P $$ fswatch 2>/dev/null
    rm -rf "$cache_dir" "$fifo"
  }
  trap cleanup EXIT INT TERM

  echo -e "\033[1;38;5;51mWatching $dir for .md changes... (Ctrl+C to stop)\033[0m"

  # Start fswatch writing to FIFO in background
  fswatch -r "$dir" 2>/dev/null | grep --line-buffered '\.md$' > "$fifo" &

  while read -r f; do
    # Skip if file doesn't exist (deleted)
    [[ -f "$f" ]] || continue

    local hash=$(echo "$f" | md5)
    local cache_file="$cache_dir/$hash"
    local first_changed_line=""

    # Find first changed line by diffing with cached version
    if [[ -f "$cache_file" ]]; then
      first_changed_line=$(diff "$cache_file" "$f" 2>/dev/null | grep -E '^[0-9]' | head -1 | sed -E 's/^[0-9,]+[acd]([0-9]+).*/\1/')
    fi

    # Cache current version
    cp "$f" "$cache_file" 2>/dev/null

    clear
    echo -e "\033[1;38;5;51m[$(date +%H:%M:%S)]\033[0m \033[1;38;5;201m$f\033[0m"
    echo -e "\033[38;5;201m────────────────────────────────────────\033[0m"

    if [[ -n "$first_changed_line" && "$first_changed_line" =~ ^[0-9]+$ ]]; then
      echo -e "\033[38;5;221mChanged at line $first_changed_line\033[0m"
      local start_line=$((first_changed_line > 5 ? first_changed_line - 5 : 1))
      bat --color=always --style=numbers --theme=vaporwave-custom --language=md \
          --highlight-line="$first_changed_line" \
          --line-range="$start_line:" \
          "$f"
    else
      # Use bat for consistent rendering (glow -p hangs in watch context)
      bat --color=always --style=plain --theme=vaporwave-custom --language=md "$f"
    fi
  done < "$fifo"
}

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
  local max_files=150
  
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
  # Ctrl+R refreshes file list from filesystem
  local file
  file=$(printf '%s\n' "${all_files[@]}" | \
    TERM=xterm-256color fzf --no-mouse \
        --ansi \
        --preview 'bat --color=always --style=plain --theme=vaporwave-custom --language=md {}' \
        --preview-window=right:75%:wrap \
        --height=100% \
        --border=rounded \
        --header='Ctrl+R to refresh file list' \
        --bind "ctrl-r:reload(find . -name '*.md' -not -path '*/node_modules/*' -not -path '*/.git/*' -not -path '*/vendor/*' -not -path '*/.cache/*' 2>/dev/null | head -150)" \
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
bindkey -M emacs '\e\e' _md_browser_widget
bindkey -M viins '\e\e' _md_browser_widget
bindkey -M vicmd '\e\e' _md_browser_widget

# Alt+M: start mdwatch in current directory
_mdwatch_widget() {
  zle -I
  mdwatch .
  zle reset-prompt
}
zle -N _mdwatch_widget
bindkey '\em' _mdwatch_widget

# === Completion System (BEFORE plugins) ===
[[ -d "$HOME/.zfunc" ]] && FPATH="$HOME/.zfunc:${FPATH}"
[[ -d /opt/homebrew/share/zsh/site-functions ]] && FPATH="/opt/homebrew/share/zsh/site-functions:${FPATH}"
[[ -d /usr/local/share/zsh/site-functions ]] && FPATH="/usr/local/share/zsh/site-functions:${FPATH}"

autoload -Uz compinit
# Use cached completions if dump is fresh (<24h), rebuild if stale
if [[ -n ~/.zcompdump(#qN.mh-24) ]]; then
  compinit -C
else
  compinit
fi

zstyle ':completion:*' completer _expand _complete _ignored _approximate _files
zstyle ':completion:*' menu select
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}' 'r:|=*' 'l:|=* r:|=*'

# File completion: show all files, prefer exact matches
zstyle ':completion:*' file-sort modification
zstyle ':completion:*:*:*:*:*' file-patterns '*:all-files' '*(-/):directories'
zstyle ':completion:*' insert-tab false

# Better completion caching with 1-hour expiry
zstyle ':completion:*' use-cache on
zstyle ':completion:*' cache-path ~/.zsh/cache
[[ -d ~/.zsh/cache ]] || mkdir -p ~/.zsh/cache

# Cache policy: invalidate after 1 hour
_cache_policy_1h() {
  [[ -z "$1" ]] && return 0
  [[ ! -f "$1" ]] && return 0
  local -a stat
  stat=("${(@f)$(stat -f '%m' "$1" 2>/dev/null)}")
  (( $(date +%s) - stat[1] > 3600 ))
}
zstyle ':completion:*' cache-policy _cache_policy_1h

# Prioritize local files for ambiguous completions
zstyle ':completion:*:*:*:default' menu select
zstyle ':completion:*' accept-exact '*(N)'

# Colorful completion descriptions
zstyle ':completion:*:descriptions' format '%B%F{201}-- %d --%f%b'
zstyle ':completion:*:warnings' format '%F{221}no matches%f'
zstyle ':completion:*:messages' format '%F{51}%d%f'
zstyle ':completion:*:corrections' format '%B%F{221}%d (errors: %e)%f%b'
zstyle ':completion:*' group-name ''
zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}  # Use LS_COLORS in completion
zstyle ':completion:*:*:kill:*:processes' list-colors '=(#b) #([0-9]#)*=0=01;31'  # Color PIDs
zstyle ':completion:*:kill:*' command 'ps -u $USER -o pid,%cpu,tty,cputime,cmd'

# Completion menu visual enhancements
zstyle ':completion:*' list-separator '  --'
zstyle ':completion:*' list-prompt '%F{51}-- %l --%f'
zstyle ':completion:*' select-prompt '%F{201}-- %p --%f'
zstyle ':completion:*:default' list-colors ${(s.:.)LS_COLORS} 'ma=48;5;201;38;5;15'  # Selected item: white on hot pink

# === Autosuggestions (AFTER compinit) ===
export ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE="fg=#666666"  # Subtle gray for suggestions
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
  # Always trigger file/command completion - use right arrow for history suggestions
  zle autosuggest-clear 2>/dev/null
  zle expand-or-complete
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

# Up arrow: handled by atuin (line ~1060)

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

# === Passive tool improvements (no workflow changes) ===
export RIPGREP_CONFIG_PATH="$HOME/.ripgreprc"
export STARSHIP_LOG="error"  # Silence starship warnings

# Colored compiler output
export GCC_COLORS='error=01;38;5;201:warning=01;38;5;221:note=01;38;5;51:caret=01;38;5;51:locus=01:quote=01'

# jq colors (vaporwave: null=pink, false=pink, true=cyan, numbers=cyan, strings=gold, arrays=purple, objects=purple)
export JQ_COLORS='1;35:0;35:0;36:0;36:0;33:1;35:1;35'

# Less colors for man pages (already set via LESS_TERMCAP but this ensures colored output)
export MANPAGER="less -R --use-color"

# Make parallel builds by default (use all cores)
export MAKEFLAGS="-j$(sysctl -n hw.ncpu)"

# fd defaults (ignore common junk)
export FD_OPTIONS="--hidden --follow --exclude .git --exclude node_modules --exclude vendor"

# === Bracketed paste (security: prevents paste injection) ===
autoload -Uz bracketed-paste-magic
zle -N bracketed-paste bracketed-paste-magic

# === Word navigation with Ctrl+arrows ===
bindkey '^[[1;5C' forward-word      # Ctrl+Right
bindkey '^[[1;5D' backward-word     # Ctrl+Left
bindkey '^[[3;5~' kill-word         # Ctrl+Delete

# === Edit command in $EDITOR with Ctrl+X Ctrl+E ===
autoload -Uz edit-command-line
zle -N edit-command-line
bindkey '^X^E' edit-command-line

# === Magic space (expand history inline) ===
# Type !! then space -> expands to last command
# Type !$ then space -> expands to last argument
bindkey ' ' magic-space

# === Run-help (Esc-h shows man page for command) ===
autoload -Uz run-help
unalias run-help 2>/dev/null
bindkey '\eh' run-help

# === Vaporwave calendar ===
# Replaces cal with colored version
cal() {
  local cyan=$'\e[38;5;51m'
  local purple=$'\e[38;5;129m'
  local gold=$'\e[38;5;221m'
  local pink=$'\e[1;38;5;201m'
  local reset=$'\e[0m'
  local today=$(date +%-d)
  local n=0

  command cal "$@" | while IFS= read -r line; do
    ((n++))
    if [[ $n -eq 1 ]]; then
      echo "${cyan}${line}${reset}"
    elif [[ $n -eq 2 ]]; then
      echo "${purple}${line}${reset}"
    else
      # Highlight today with word boundary matching
      echo "${gold}$(echo "$line" | sed -E "s/(^|[^0-9])${today}([^0-9]|$)/\1${pink}${today}${reset}${gold}\2/g")${reset}"
    fi
  done
}

# === Performance tuning ===
# KEYTIMEOUT set earlier (10 = 100ms for double-tap Esc)

# Compile zcompdump for faster loading
[[ ~/.zcompdump.zwc -ot ~/.zcompdump ]] && zcompile ~/.zcompdump 2>/dev/null

# Lazy-load heavy completions
zstyle ':completion:*' use-cache true
zstyle ':completion:*' rehash true  # Auto-detect new executables

# === Enhanced output ===
# More informative time command output
TIMEFMT=$'\n%J\n  user: %U  sys: %S  total: %*E\n  cpu: %P  mem: %MKB'

# Show command timing for long commands (>5s)
REPORTTIME=5

# === Git speedups ===
# Disable git prompt for huge repos (faster prompt)
export GIT_PS1_SHOWDIRTYSTATE=
export GIT_PS1_SHOWUNTRACKEDFILES=

# === Homebrew speedups ===
export HOMEBREW_NO_AUTO_UPDATE=1
export HOMEBREW_NO_INSTALL_CLEANUP=1
export HOMEBREW_NO_ANALYTICS=1

# === Atuin (history backend) + fzf (UI) ===
if command -v atuin &>/dev/null; then
  # Use atuin for history storage but fzf for search UI
  eval "$(atuin init zsh --disable-up-arrow --disable-ctrl-r)"

  _atuin_fzf_search() {
    emulate -L zsh
    zle -I
    local selected
    selected=$(atuin search --cmd-only --limit 5000 2>/dev/null | \
      fzf --height=~15 --min-height=5 --tac --no-sort --query="$BUFFER" \
          --bind='ctrl-d:half-page-down,ctrl-u:half-page-up,tab:accept' \
          --color='fg:#5cecff,bg:-1,hl:#ff0099,fg+:#fbb725,bg+:-1,hl+:#ff0099,pointer:#ff0099,prompt:#5cecff,info:#666666')
    zle reset-prompt
    if [[ -n $selected ]]; then
      RBUFFER=""
      LBUFFER=$selected
    fi
  }
  zle -N atuin-fzf-search _atuin_fzf_search
  bindkey '^r' atuin-fzf-search
  bindkey '^[[A' atuin-fzf-search
  bindkey '^[OA' atuin-fzf-search
fi

# === Syntax Highlighting (MUST BE LAST) ===
for f in \
  /opt/homebrew/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh \
  /usr/local/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh \
  /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh; do
  [[ -f "$f" ]] && source "$f" && break
done

# Syntax highlighting colors (vaporwave)
if [[ -n "$ZSH_HIGHLIGHT_STYLES" ]]; then
  # Commands: cyan
  ZSH_HIGHLIGHT_STYLES[command]='fg=#5cecff,bold'
  ZSH_HIGHLIGHT_STYLES[arg0]='fg=#5cecff'
  ZSH_HIGHLIGHT_STYLES[function]='fg=#5cecff,bold'
  # Aliases/builtins: hot pink
  ZSH_HIGHLIGHT_STYLES[alias]='fg=#ff0099,bold'
  ZSH_HIGHLIGHT_STYLES[builtin]='fg=#ff00f8,bold'
  ZSH_HIGHLIGHT_STYLES[reserved-word]='fg=#ff00f8'
  # Strings: gold
  ZSH_HIGHLIGHT_STYLES[path]='fg=#fbb725,underline'
  ZSH_HIGHLIGHT_STYLES[single-quoted-argument]='fg=#fbb725'
  ZSH_HIGHLIGHT_STYLES[double-quoted-argument]='fg=#fbb725'
  # Unknown/errors: purple (not red)
  ZSH_HIGHLIGHT_STYLES[unknown-token]='fg=#aa00e8'
  ZSH_HIGHLIGHT_STYLES[commandseparator]='fg=#5cecff'
  ZSH_HIGHLIGHT_STYLES[redirection]='fg=#ff0099'
  ZSH_HIGHLIGHT_STYLES[globbing]='fg=#aa00e8'
  ZSH_HIGHLIGHT_STYLES[history-expansion]='fg=#aa00e8'
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

# === Dangerous Command Warning ===
_check_dangerous() {
  local cmd="$1"
  local warn=""
  case "$cmd" in
    *'rm -rf'*|*'rm -fr'*) warn="rm -rf" ;;
    *'git push -f'*|*'git push --force'*) warn="force push" ;;
    *'git reset --hard'*) warn="reset --hard" ;;
    *'git clean -f'*) warn="git clean -f" ;;
    *'chmod -R 777'*) warn="chmod 777" ;;
    *'> /'*) warn="overwrite root file" ;;
    *'dd if='*) warn="dd" ;;
  esac
  if [[ -n "$warn" ]]; then
    print -Pn "%F{203}[WARNING: $warn] Continue? (y/N) %f"
    read -k1 reply
    echo
    [[ "$reply" != [yY] ]] && return 1
  fi
  return 0
}

# === Phase detector - what mode are you in? ===
typeset -ga __recent_cmds=()
_detect_phase() {
  local cmd="${1%% *}"  # First word
  __recent_cmds+=("$cmd")
  (( ${#__recent_cmds} > 10 )) && __recent_cmds=("${__recent_cmds[@]: -10}")

  # Count patterns in recent commands
  local writing=0 testing=0 debugging=0 building=0 exploring=0
  for c in "${__recent_cmds[@]}"; do
    case "$c" in
      vim|nvim|code|nano|emacs|edit) ((writing++)) ;;
      test|pytest|jest|mocha|cargo|go) ((testing++)) ;;
      echo|print|log|gdb|lldb|debug) ((debugging++)) ;;
      make|build|compile|npm|yarn|cargo) ((building++)) ;;
      ls|cd|find|grep|cat|less|head|tail) ((exploring++)) ;;
    esac
  done

  # Determine dominant phase
  local max=$writing phase="writing"
  (( testing > max )) && max=$testing phase="testing"
  (( debugging > max )) && max=$debugging phase="debugging"
  (( building > max )) && max=$building phase="building"
  (( exploring > max )) && max=$exploring phase="exploring"

  (( max >= 3 )) && export __current_phase="$phase" || export __current_phase=""
}

# === Blast radius warning ===
_blast_radius() {
  local cmd="$1"
  local warning=""
  local count=0

  case "$cmd" in
    rm\ -rf*|rm\ -fr*)
      # Extract path and count potential victims
      local path="${cmd#rm -rf }"
      path="${path#rm -fr }"
      [[ -d "$path" ]] && count=$(find "$path" -type f 2>/dev/null | wc -l | tr -d ' ')
      (( count > 10 )) && warning="$count files"
      ;;
    git\ checkout\ .|git\ restore\ .)
      count=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
      (( count > 0 )) && warning="$count files"
      ;;
    git\ clean*)
      count=$(git clean -n -d 2>/dev/null | wc -l | tr -d ' ')
      (( count > 0 )) && warning="$count files"
      ;;
    chmod\ -R*|chown\ -R*)
      local path="${cmd##* }"
      [[ -d "$path" ]] && count=$(find "$path" -type f 2>/dev/null | wc -l | tr -d ' ')
      (( count > 20 )) && warning="$count files"
      ;;
  esac

  [[ -n "$warning" ]] && print -P "%F{221}~ $warning%f"
}

# === Welcome back (friendly idle detection) ===
typeset -g __last_cmd_time=$(date +%s)
_welcome_back() {
  local now=$(date +%s)
  local idle=$((now - __last_cmd_time))
  __last_cmd_time=$now

  if (( idle > 600 )); then
    local mins=$((idle / 60))
    if (( mins >= 60 )); then
      print -P "%F{243}welcome back%f %F{238}(${mins}m break)%f"
    else
      print -P "%F{243}welcome back%f %F{238}(${mins}m)%f"
    fi
  elif (( idle > 300 )); then
    print -P "%F{238}...%f"
  fi
}

# === Win celebration (detect test/build success) ===
_check_win() {
  local exit_code=$1
  local cmd="$2"
  (( exit_code != 0 )) && return

  case "$cmd" in
    *test*|*pytest*|*jest*|*mocha*|*cargo\ test*|*go\ test*|*npm\ test*|*make\ test*)
      print -P "%F{51}tests passed%f"
      ;;
    *build*|*make\ build*|*cargo\ build*|*go\ build*|*npm\ run\ build*)
      print -P "%F{51}build succeeded%f"
      ;;
    *compile*|*make\ all*)
      print -P "%F{51}compiled%f"
      ;;
  esac
}

# === Weekly momentum (days coded this week) ===
_weekly_momentum() {
  local momentum_file=~/.cache/weekly-momentum
  local today=$(date +%Y-%m-%d)
  local dow=$(date +%u)  # 1=Monday, 7=Sunday

  # Read existing data
  local data=""
  [[ -f "$momentum_file" ]] && data=$(cat "$momentum_file")

  # Add today if not present
  if [[ "$data" != *"$today"* ]]; then
    echo "$today" >> "$momentum_file"
  fi

  # Count days this week (keep only last 7 days)
  local week_start=$(date -v-$((dow-1))d +%Y-%m-%d 2>/dev/null || date -d "last monday" +%Y-%m-%d 2>/dev/null)
  local count=$(awk -v start="$week_start" '$1 >= start' "$momentum_file" 2>/dev/null | wc -l | tr -d ' ')

  echo "$count/7"
}

# === Focus time (uninterrupted session) ===
typeset -g __focus_start=${__focus_start:-$(date +%s)}
typeset -g __focus_breaks=0
_track_focus() {
  local now=$(date +%s)
  local idle=$((now - __last_cmd_time))

  # Break detected (>10 min idle resets focus)
  if (( idle > 600 )); then
    __focus_start=$now
    __focus_breaks=0
  fi

  local focus_mins=$(( (now - __focus_start) / 60 ))
  if (( focus_mins >= 60 )); then
    local hours=$(( focus_mins / 60 ))
    local mins=$(( focus_mins % 60 ))
    export __focus_time="${hours}h${mins}m focus"
  elif (( focus_mins >= 30 )); then
    export __focus_time="${focus_mins}m focus"
  else
    export __focus_time=""
  fi
}

# === Coding hours tracking ===
_track_coding_hours() {
  local today=$(date +%Y-%m-%d)
  local hours_file=~/.cache/coding-hours-data
  local display_file=~/.cache/coding-hours-today
  local now=$(date +%s)

  # Read last timestamp and date
  local last_ts=0 last_date=""
  [[ -f "$hours_file" ]] && read last_date last_ts total_secs < "$hours_file" 2>/dev/null

  # Reset if new day
  [[ "$last_date" != "$today" ]] && total_secs=0

  # Add time since last command (max 5 min to avoid idle time)
  if (( last_ts > 0 && now - last_ts < 300 )); then
    (( total_secs += now - last_ts ))
  fi

  # Save state
  echo "$today $now $total_secs" >| "$hours_file"

  # Update display (hours with 1 decimal)
  local hours=$(echo "scale=1; $total_secs / 3600" | bc)
  echo "${hours}h" >| "$display_file"
}

# === Momentum tracking ===
typeset -ga __cmd_times=()
_momentum_sparkline() {
  # Track last 20 command timestamps, show sparkline
  local now=$(date +%s)
  __cmd_times+=($now)
  # Keep only last 20
  (( ${#__cmd_times} > 20 )) && __cmd_times=("${__cmd_times[@]: -20}")
  # Calculate intervals and map to sparkline using block chars
  if (( ${#__cmd_times} >= 5 )); then
    local spark=""
    local chars=("▁" "▂" "▃" "▄" "▅" "▆" "▇" "█")
    for (( i=2; i<=${#__cmd_times}; i++ )); do
      local gap=$(( ${__cmd_times[$i]} - ${__cmd_times[$((i-1))]} ))
      # Map gap to char: logarithmic scale (powers of 2)
      local idx=0
      (( gap < 1 )) && idx=7
      (( gap >= 1 && gap < 2 )) && idx=6
      (( gap >= 2 && gap < 4 )) && idx=5
      (( gap >= 4 && gap < 8 )) && idx=4
      (( gap >= 8 && gap < 16 )) && idx=3
      (( gap >= 16 && gap < 32 )) && idx=2
      (( gap >= 32 && gap < 64 )) && idx=1
      (( gap >= 64 )) && idx=0
      spark+="${chars[$((idx+1))]}"
    done
    export __momentum_spark="$spark"
  fi
}

# Show momentum on demand
momentum() {
  if [[ -n "$__momentum_spark" ]]; then
    print -P "%F{243}$__momentum_spark%f"
    local high=$(echo "$__momentum_spark" | tr -cd '█▇▆' | wc -c | tr -d ' ')
    local total=${#__momentum_spark}
    if (( total > 0 )); then
      local pct=$((high * 100 / total))
      if (( pct > 70 )); then
        print -P "%F{51}in flow%f"
      elif (( pct > 40 )); then
        print -P "%F{221}steady%f"
      else
        print -P "%F{243}warming up%f"
      fi
    fi
  else
    print -P "%F{243}not enough data yet%f"
  fi
}

# === Command prediction ===
# Predict next command based on history patterns
predict() {
  local last_cmd="${1:-$(fc -ln -1 | awk '{print $1}')}"
  # Find most common command that follows last_cmd
  awk -F';' -v last="$last_cmd" '
    NR>1 && prev ~ "^"last { count[curr]++ }
    { prev=$2; curr=$2 }
    END {
      max=0; pred=""
      for (c in count) if (count[c]>max) { max=count[c]; pred=c }
      if (pred!="") print pred
    }
  ' "$HISTFILE" | head -1
}

# === Command Duration Display ===
# Show duration when complete, update iTerm2 badge
preexec() {
  _blast_radius "$1"
  _detect_phase "$1"
  _track_coding_hours
  _momentum_sparkline
  _command_start_time=$SECONDS
  __last_cmd="$1"  # Track command for history cleanup
  _last_cmd_name="${1%% *}"  # First word for notification
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
  local -a __pstat=("${pipestatus[@]}")

  # Welcome back + focus tracking
  _track_focus
  _welcome_back

  # Win celebration for tests/builds
  [[ -n "$__last_cmd" ]] && _check_win $__last_exit "$__last_cmd"

  # Show PIPESTATUS if pipe with any failures
  if (( ${#__pstat} > 1 )); then
    local has_fail=0
    for code in "${__pstat[@]}"; do (( code != 0 )) && has_fail=1 && break; done
    (( has_fail )) && print -P "%F{243}pipe: ${(j:|:)__pstat}%f"
  fi

  # Exit code translation
  if (( __last_exit != 0 )); then
    local meaning=""
    case $__last_exit in
      1)   meaning="general error" ;;
      126) meaning="not executable" ;;
      127) meaning="command not found" ;;
      130) meaning="interrupted (Ctrl+C)" ;;
      137) meaning="killed (SIGKILL)" ;;
      143) meaning="terminated (SIGTERM)" ;;
      *)   (( __last_exit > 128 )) && meaning="signal $((__last_exit - 128))" ;;
    esac
    [[ -n "$meaning" ]] && print -P "%F{243}exit $__last_exit: $meaning%f"
  fi

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
          # Remove the failed command from history file (portable: macOS + Linux)
          local escaped="${failed//\//\\/}"
          escaped="${escaped//\[/\\[}"
          escaped="${escaped//\]/\\]}"
          sed -i.bak "/;${escaped}$/d" "$HISTFILE" 2>/dev/null && rm -f "$HISTFILE.bak"
        fi
      done
      __failed_cmds=()  # clear after success
    fi
  fi

  unset __last_cmd

  # Show command duration + macOS notification for long commands
  if [[ -n $_command_start_time ]]; then
    local elapsed=$(( SECONDS - _command_start_time ))
    if (( elapsed >= 3 )); then
      echo -e "\033[38;5;221m${elapsed}s\033[0m"
      # Bell if command took >4 minutes
      if (( elapsed >= 240 )); then
        print '\a'
      fi
      # macOS notification if command took >30s (background terminal)
      if (( elapsed >= 30 )); then
        osascript -e "display notification \"Completed in ${elapsed}s\" with title \"$_last_cmd_name\"" 2>/dev/null &!
      fi
    fi
  fi
  unset _command_start_time _last_cmd_name
  
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

# yazi file manager - cd to directory on exit
y() {
  local tmp="$(mktemp -t "yazi-cwd.XXXXXX")"
  yazi "$@" --cwd-file="$tmp"
  if [[ -f "$tmp" ]]; then
    local cwd="$(cat "$tmp")"
    [[ -n "$cwd" && "$cwd" != "$PWD" ]] && cd "$cwd"
  fi
  rm -f "$tmp"
}

# === Auto-start yazi in Ghostty ===
# Ghostty is fast enough for yazi; iTerm2 is not
if [[ "$TERM_PROGRAM" == "ghostty" && -z "$YAZI_LEVEL" && -o interactive ]]; then
  # Start yazi, and cd to the directory yazi exits in
  function _yazi_startup() {
    local tmp="$(mktemp -t "yazi-cwd.XXXXXX")"
    yazi --cwd-file="$tmp"
    if [[ -f "$tmp" ]]; then
      local cwd="$(cat "$tmp")"
      [[ -n "$cwd" && "$cwd" != "$PWD" ]] && cd "$cwd"
    fi
    rm -f "$tmp"
  }
  _yazi_startup
  unfunction _yazi_startup
fi

# === Which All - show all versions in PATH ===
whichall() {
  [[ -z "$1" ]] && { echo "Usage: whichall <command>"; return 1; }
  local cmd="$1"
  local found=0
  local IFS=:
  for dir in $PATH; do
    [[ -x "$dir/$cmd" ]] && {
      local ver=$("$dir/$cmd" --version 2>/dev/null | head -1)
      print -P "%F{51}$dir/$cmd%f ${ver:+%F{243}($ver)%f}"
      found=1
    }
  done
  (( ! found )) && print -P "%F{203}$cmd not found in PATH%f"
}

# === Trash instead of rm ===
trash() {
  [[ $# -eq 0 ]] && { echo "Usage: trash <file> ..."; return 1; }
  local f
  for f in "$@"; do
    if [[ -e "$f" ]]; then
      mv "$f" ~/.Trash/
    else
      print -P "%F{203}$f: not found%f"
    fi
  done
}

# === Colorize stderr (red) ===
# Disabled: process substitution causes parse error messages
# exec 2> >(while IFS= read -r line; do print -P "%F{203}$line%f" >&2; done)

# === History Stats ===
histstats() {
  print -P "%F{221}History Statistics%f"
  local total=$(wc -l < "$HISTFILE" | tr -d ' ')
  local unique=$(awk -F';' '{print $2}' "$HISTFILE" | sort -u | wc -l | tr -d ' ')
  print -P "  Total: %F{51}$total%f commands"
  print -P "  Unique: %F{51}$unique%f commands"
  print -P "\n%F{221}Top 10 commands:%f"
  awk -F';' '{print $2}' "$HISTFILE" | awk '{print $1}' | sort | uniq -c | sort -rn | head -10 | \
    while read count cmd; do
      print -P "  %F{243}$count%f $cmd"
    done
}

# === Empty Enter = Git Diffstat + Dangerous Command Check ===
function _accept_line_or_diffstat() {
  # Empty line: show git diffstat without new prompt
  if [[ -z "$BUFFER" ]]; then
    if git rev-parse --git-dir &>/dev/null; then
      local file_count=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
      local stat=$(git diff --shortstat 2>/dev/null)
      local output=""
      (( file_count > 0 )) && output="$file_count files"
      [[ -n "$stat" ]] && output="${output:+$output, }$stat"
      if [[ -n "$output" ]]; then
        zle -I
        print -P "%F{243}$output%f"
        return
      fi
    fi
    # Empty line, no git info - just accept (normal Enter behavior)
    zle .accept-line
    return
  fi
  # Non-empty: check for dangerous commands
  if ! _check_dangerous "$BUFFER"; then
    zle redisplay
    return
  fi
  zle .accept-line
}
zle -N accept-line _accept_line_or_diffstat

export PATH="$HOME/.local/bin:$PATH"

# Dynamic wallpaper project
# ~/Pictures/dynamic-wallpaper/ - AI-generated wallpapers for time-based cycling
# Use wallpapper CLI to build HEIC from images
setopt HIST_VERIFY
WORDCHARS='${WORDCHARS//[\/]}'
zstyle ':completion:*' use-cache on
export LESS="-R -F -X -i -J -W"

# Auto-remove command-not-found typos from history
__prune_typos() {
    [[ $? -eq 127 ]] || return
    fc -W
    head -n -1 "$HISTFILE" > "$HISTFILE.tmp" && mv "$HISTFILE.tmp" "$HISTFILE"
    fc -R
}
precmd_functions+=(__prune_typos)

# Port-in-use helper - show what's using port on bind errors
__port_helper() {
    [[ $? -ne 0 ]] && fc -ln -1 | grep -q "address.*in use\|EADDRINUSE" && \
        lsof -i -P | grep LISTEN
}
precmd_functions+=(__port_helper)

# Show git diff stat when entering dirty repo
__git_dirty_reminder() {
    [[ -d .git ]] && ! git diff --stat --quiet 2>/dev/null && git diff --stat 2>/dev/null | tail -1
}
chpwd_functions+=(__git_dirty_reminder)

# Warn before large rm -rf
unalias rm 2>/dev/null
rm() {
    if [[ "$*" =~ "-rf" ]] || [[ "$*" =~ "-r" ]]; then
        local target="${@[-1]}"
        [[ -e "$target" ]] && local size=$(du -sh "$target" 2>/dev/null | cut -f1)
        [[ -n "$size" ]] && print -P "%F{yellow}Removing $size%f"
    fi
    command rm -v "$@"
}

# Auto-title terminal with current command/directory
# NOTE: Terminal title is already handled by _set_terminal_title in precmd_functions (line 138)
# and the main preexec/precmd functions. These duplicate definitions were overriding all
# the sophisticated prompt functionality. Removed.

# SSH key auto-add on first use
ssh-add -l &>/dev/null || ssh-add --apple-use-keychain ~/.ssh/id_* 2>/dev/null
ulimit -n 10240
