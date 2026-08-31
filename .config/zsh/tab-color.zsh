# tab-color.zsh — color the current iTerm2 tab via the OSC 6 escape.
#
# Primitive:  tab-color <name|#hex|reset>
#   names are the vaporwave palette; a bare 6-hex also works; `reset` clears.
#
# Auto mode (opt-in): `export TAB_COLOR_AUTO=1` before sourcing. A chpwd hook
# then colors each tab by the project's accent HUE and a BRIGHTNESS that
# tracks how recently the repo was committed to:
#   hue  = PROMPT_ACCENT (exported from .envrc), else a stable palette color
#          hashed from the repo name.
#   value = full for a repo committed to today, dimming to a floor over ~30
#          days since its last commit. Non-repos and unborn repos stay full.
# Source this AFTER direnv's hook in .zshrc so PROMPT_ACCENT is current.

zmodload -F zsh/datetime b:EPOCHSECONDS 2>/dev/null

typeset -gA _TAB_PALETTE=(
  pink    ff0099
  cyan    5cecff
  magenta ff00f8
  gold    fbb725
  purple  aa00e8
)
typeset -ga _TAB_PALETTE_ORDER=(ff0099 5cecff ff00f8 fbb725 aa00e8)

# Brightness floor (percent) so a stale project keeps its identifiable hue.
typeset -g _TAB_COLOR_FLOOR=40
# Days of no commits at which brightness reaches the floor.
typeset -g _TAB_COLOR_HORIZON=30

# Emit the iTerm2 OSC 6 tab-color escape for explicit R,G,B (0-255 each).
_tab_color_emit_rgb() {
  printf '\e]6;1;bg;red;brightness;%d\a'   $1
  printf '\e]6;1;bg;green;brightness;%d\a' $2
  printf '\e]6;1;bg;blue;brightness;%d\a'  $3
}

# Scale a 6-hex color by a brightness percent (0-100) and emit it.
_tab_color_emit_hex() {
  local hex=$1 pct=${2:-100}
  local r=$(( 16#${hex[1,2]} )) g=$(( 16#${hex[3,4]} )) b=$(( 16#${hex[5,6]} ))
  _tab_color_emit_rgb $(( r * pct / 100 )) $(( g * pct / 100 )) $(( b * pct / 100 ))
}

# tab-color <name|#hex|reset> — manual, full brightness.
tab-color() {
  local arg=${1:-}
  if [[ -z $arg || $arg == reset || $arg == default ]]; then
    printf '\e]6;1;bg;*;default\a'
    return
  fi
  local hex=${_TAB_PALETTE[$arg]:-${arg#\#}}
  if [[ ${#hex} -ne 6 || $hex != [0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F] ]]; then
    print -u2 "tab-color: not a palette name or 6-hex value: $arg"
    return 1
  fi
  _tab_color_emit_hex $hex 100
}

# Palette hex for the current directory: PROMPT_ACCENT if set, else a stable
# pick hashed from the git repo name (falls back to the $PWD basename).
_tab_color_for_pwd() {
  if [[ -n ${PROMPT_ACCENT:-} ]]; then
    print -r -- ${PROMPT_ACCENT#\#}
    return
  fi
  local name
  name=$(git rev-parse --show-toplevel 2>/dev/null) || name=$PWD
  name=${name:t}
  local sum=${$(print -rn -- "$name" | cksum)%% *}
  print -r -- ${_TAB_PALETTE_ORDER[$(( sum % 5 + 1 ))]}
}

# Brightness percent from the age of the repo's last commit: 100 today,
# dropping linearly to _TAB_COLOR_FLOOR at _TAB_COLOR_HORIZON days, clamped.
# Non-repos and repos with no commits return 100 (full).
_tab_color_recency_pct() {
  local ct
  ct=$(git log -1 --format=%ct 2>/dev/null) || { print 100; return; }
  [[ -n $ct ]] || { print 100; return; }
  local age_days=$(( (EPOCHSECONDS - ct) / 86400 ))
  (( age_days < 0 )) && age_days=0
  local span=$(( 100 - _TAB_COLOR_FLOOR ))
  local pct=$(( 100 - age_days * span / _TAB_COLOR_HORIZON ))
  (( pct < _TAB_COLOR_FLOOR )) && pct=$_TAB_COLOR_FLOOR
  print $pct
}

_tab_color_auto() {
  [[ -n ${TAB_COLOR_AUTO:-} ]] || return
  _tab_color_emit_hex "$(_tab_color_for_pwd)" "$(_tab_color_recency_pct)"
}

autoload -Uz add-zsh-hook
add-zsh-hook chpwd _tab_color_auto
_tab_color_auto   # color the current tab at load
