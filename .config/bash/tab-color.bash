# tab-color.bash — color the current iTerm2 tab via the OSC 6 escape (bash port).
#
# Mirror of .config/zsh/tab-color.zsh, written for bash 3.2+ (no associative
# arrays, no EPOCHSECONDS) so it runs on old and new bash alike.
#
# Primitive:  tab-color <name|#hex|reset>
#   names are the vaporwave palette; a bare 6-hex also works; `reset` clears.
#
# Auto mode (opt-in): `export TAB_COLOR_AUTO=1` before sourcing. A PROMPT_COMMAND
# hook recolors the tab when the directory changes:
#   hue   = PROMPT_ACCENT (from .envrc), else a stable palette color hashed
#           from the repo name.
#   value = full for a repo committed to today, dimming to a floor over ~30
#           days since its last commit. Non-repos and unborn repos stay full.

_TAB_PALETTE_ORDER=(ff0099 5cecff ff00f8 fbb725 aa00e8)
_TAB_COLOR_FLOOR=40      # brightness floor (percent) so stale repos keep their hue
_TAB_COLOR_HORIZON=30    # days of no commits at which brightness reaches the floor

# palette name -> hex (a case, since bash 3.2 has no associative arrays)
_tab_palette_hex() {
  case "$1" in
    pink)    echo ff0099 ;;
    cyan)    echo 5cecff ;;
    magenta) echo ff00f8 ;;
    gold)    echo fbb725 ;;
    purple)  echo aa00e8 ;;
    *)       return 1 ;;
  esac
}

# Emit the iTerm2 OSC 6 tab-color escape for explicit R,G,B (0-255 each).
_tab_color_emit_rgb() {
  printf '\033]6;1;bg;red;brightness;%d\a'   "$1"
  printf '\033]6;1;bg;green;brightness;%d\a' "$2"
  printf '\033]6;1;bg;blue;brightness;%d\a'  "$3"
}

# Scale a 6-hex color by a brightness percent (0-100) and emit it.
_tab_color_emit_hex() {
  local hex="$1" pct="${2:-100}"
  local r=$((16#${hex:0:2})) g=$((16#${hex:2:2})) b=$((16#${hex:4:2}))
  _tab_color_emit_rgb $((r*pct/100)) $((g*pct/100)) $((b*pct/100))
}

# tab-color <name|#hex|reset> — manual, full brightness.
tab-color() {
  local arg="${1:-}"
  if [ -z "$arg" ] || [ "$arg" = reset ] || [ "$arg" = default ]; then
    printf '\033]6;1;bg;*;default\a'
    return
  fi
  local hex
  hex=$(_tab_palette_hex "$arg") || hex="${arg#\#}"
  case "$hex" in
    [0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F]) ;;
    *) echo "tab-color: not a palette name or 6-hex value: $arg" >&2; return 1 ;;
  esac
  _tab_color_emit_hex "$hex" 100
}

# Palette hex for the current directory: PROMPT_ACCENT if set, else a stable
# pick hashed from the git repo name (falls back to the $PWD basename).
_tab_color_for_pwd() {
  if [ -n "${PROMPT_ACCENT:-}" ]; then
    echo "${PROMPT_ACCENT#\#}"
    return
  fi
  local name
  name=$(git rev-parse --show-toplevel 2>/dev/null) || name="$PWD"
  name="${name##*/}"
  local sum
  sum=$(printf '%s' "$name" | cksum)   # "CHECKSUM BYTECOUNT"
  sum="${sum%% *}"                      # keep the checksum field only
  echo "${_TAB_PALETTE_ORDER[$((sum % 5))]}"
}

# Brightness percent from the age of the repo's last commit: 100 today,
# dropping linearly to _TAB_COLOR_FLOOR at _TAB_COLOR_HORIZON days, clamped.
# Non-repos and repos with no commits return 100 (full).
_tab_color_recency_pct() {
  local ct
  ct=$(git log -1 --format=%ct 2>/dev/null) || { echo 100; return; }
  [ -n "$ct" ] || { echo 100; return; }
  local now age_days span pct
  now=$(date +%s)
  age_days=$(( (now - ct) / 86400 ))
  [ "$age_days" -lt 0 ] && age_days=0
  span=$(( 100 - _TAB_COLOR_FLOOR ))
  pct=$(( 100 - age_days * span / _TAB_COLOR_HORIZON ))
  [ "$pct" -lt "$_TAB_COLOR_FLOOR" ] && pct=$_TAB_COLOR_FLOOR
  echo "$pct"
}

# Recolor only when the directory changed, so we don't shell out to git on
# every prompt (bash's PROMPT_COMMAND runs each prompt, not only on cd).
_tab_color_auto() {
  [ -n "${TAB_COLOR_AUTO:-}" ] || return
  if [ "$PWD" != "${_TAB_LAST_PWD:-}" ]; then
    _TAB_LAST_PWD="$PWD"
    _tab_color_emit_hex "$(_tab_color_for_pwd)" "$(_tab_color_recency_pct)"
  fi
}

# Register the prompt hook once (idempotent across re-sourcing).
case "${PROMPT_COMMAND:-}" in
  *_tab_color_auto*) ;;
  *) PROMPT_COMMAND="_tab_color_auto${PROMPT_COMMAND:+; $PROMPT_COMMAND}" ;;
esac
