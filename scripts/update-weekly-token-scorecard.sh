#!/bin/sh
set -eu

TOKEN_REFRESH_SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
TOKEN_REFRESH_PYTHON=${TOKEN_REFRESH_PYTHON:-/usr/bin/python3}
TOKEN_REFRESH_COLLECTOR=${TOKEN_REFRESH_COLLECTOR:-$TOKEN_REFRESH_SCRIPT_DIR/weekly_repo_tokens.py}
TOKEN_REFRESH_TOP=${TOKEN_REFRESH_TOP:-5}
TOKEN_REFRESH_STATUS=${SCORECARD_TOKEN_FILE:-$HOME/.config/scorecard/weekly-agent-tokens.md}
TOKEN_REFRESH_ACTIVE=${SCORECARD_ACTIVE_FILE:-$HOME/.config/scorecard/status.md}
TOKEN_REFRESH_REPORT=${TOKEN_REPORT_FILE:-$HOME/.local/share/scorecard/weekly-agent-tokens.json}

case "$TOKEN_REFRESH_TOP" in
  ''|*[!0-9]*)
    echo "TOKEN_REFRESH_TOP must be a positive integer" >&2
    exit 64
    ;;
esac
if [ "$TOKEN_REFRESH_TOP" -lt 1 ]; then
  echo "TOKEN_REFRESH_TOP must be a positive integer" >&2
  exit 64
fi

TOKEN_REFRESH_STATUS_DIR=$(dirname -- "$TOKEN_REFRESH_STATUS")
TOKEN_REFRESH_REPORT_DIR=$(dirname -- "$TOKEN_REFRESH_REPORT")
/bin/mkdir -p "$TOKEN_REFRESH_STATUS_DIR" "$TOKEN_REFRESH_REPORT_DIR"

TOKEN_REFRESH_STATUS_TMP=$(/usr/bin/mktemp "$TOKEN_REFRESH_STATUS_DIR/.weekly-agent-tokens.XXXXXX")
TOKEN_REFRESH_REPORT_TMP=$(/usr/bin/mktemp "$TOKEN_REFRESH_REPORT_DIR/.weekly-agent-tokens.XXXXXX")
TOKEN_REFRESH_CHART_TMP=$(
  /usr/bin/mktemp "$TOKEN_REFRESH_STATUS_DIR/.weekly-agent-token-chart.XXXXXX"
)
TOKEN_REFRESH_ACTIVE_TMP=
cleanup() {
  /bin/rm -f -- \
    "$TOKEN_REFRESH_STATUS_TMP" \
    "$TOKEN_REFRESH_REPORT_TMP" \
    "$TOKEN_REFRESH_CHART_TMP" \
    "$TOKEN_REFRESH_ACTIVE_TMP"
}
trap cleanup EXIT HUP INT TERM

"$TOKEN_REFRESH_PYTHON" "$TOKEN_REFRESH_COLLECTOR" \
  --format scorecard \
  --top "$TOKEN_REFRESH_TOP" \
  --json-output "$TOKEN_REFRESH_REPORT_TMP" \
  > "$TOKEN_REFRESH_STATUS_TMP"

test -s "$TOKEN_REFRESH_STATUS_TMP"
test -s "$TOKEN_REFRESH_REPORT_TMP"
/usr/bin/grep -q '^# Weekly agent-token footprint$' "$TOKEN_REFRESH_STATUS_TMP"
/usr/bin/python3 -m json.tool "$TOKEN_REFRESH_REPORT_TMP" >/dev/null

/usr/bin/awk '
  BEGIN { title = "## Chart: Tokens consumed per repository (millions)" }
  $0 == title { in_chart = 1 }
  in_chart && /^## / && $0 != title { exit }
  in_chart { lines[++count] = $0 }
  END {
    while (count > 0 && lines[count] == "") {
      count--
    }
    for (line = 1; line <= count; line++) {
      print lines[line]
    }
  }
' "$TOKEN_REFRESH_STATUS_TMP" > "$TOKEN_REFRESH_CHART_TMP"

test -s "$TOKEN_REFRESH_CHART_TMP"
/usr/bin/grep -q '^type: histogram$' "$TOKEN_REFRESH_CHART_TMP"

TOKEN_REFRESH_UPDATE_ACTIVE=0
if [ ! -L "$TOKEN_REFRESH_ACTIVE" ] && [ -f "$TOKEN_REFRESH_ACTIVE" ] && \
  /usr/bin/awk '
    $0 == "<!-- weekly-agent-tokens:begin -->" { begin_count++; begin_line = NR }
    $0 == "<!-- weekly-agent-tokens:end -->" { end_count++; end_line = NR }
    END {
      valid = begin_count == 1 && end_count == 1 && begin_line < end_line
      exit !valid
    }
  ' "$TOKEN_REFRESH_ACTIVE"; then
  TOKEN_REFRESH_UPDATE_ACTIVE=1
fi

if [ "$TOKEN_REFRESH_UPDATE_ACTIVE" -eq 1 ]; then
  TOKEN_REFRESH_ACTIVE_DIR=$(dirname -- "$TOKEN_REFRESH_ACTIVE")
  /bin/mkdir -p "$TOKEN_REFRESH_ACTIVE_DIR"
  TOKEN_REFRESH_ACTIVE_TMP=$(
    /usr/bin/mktemp "$TOKEN_REFRESH_ACTIVE_DIR/.weekly-agent-tokens-active.XXXXXX"
  )
  /usr/bin/awk -v chart="$TOKEN_REFRESH_CHART_TMP" '
    $0 == "<!-- weekly-agent-tokens:begin -->" {
      print
      while ((getline line < chart) > 0) {
        print line
      }
      close(chart)
      replacing = 1
      next
    }
    $0 == "<!-- weekly-agent-tokens:end -->" {
      replacing = 0
      print
      next
    }
    !replacing { print }
  ' "$TOKEN_REFRESH_ACTIVE" > "$TOKEN_REFRESH_ACTIVE_TMP"
  test -s "$TOKEN_REFRESH_ACTIVE_TMP"
  /usr/bin/grep -q '^<!-- weekly-agent-tokens:begin -->$' "$TOKEN_REFRESH_ACTIVE_TMP"
  /usr/bin/grep -q '^<!-- weekly-agent-tokens:end -->$' "$TOKEN_REFRESH_ACTIVE_TMP"
  /bin/chmod 0644 "$TOKEN_REFRESH_ACTIVE_TMP"
fi

/bin/chmod 0644 "$TOKEN_REFRESH_STATUS_TMP" "$TOKEN_REFRESH_REPORT_TMP"
/bin/mv -f "$TOKEN_REFRESH_STATUS_TMP" "$TOKEN_REFRESH_STATUS"
/bin/mv -f "$TOKEN_REFRESH_REPORT_TMP" "$TOKEN_REFRESH_REPORT"
if [ "$TOKEN_REFRESH_UPDATE_ACTIVE" -eq 1 ]; then
  /bin/mv -f "$TOKEN_REFRESH_ACTIVE_TMP" "$TOKEN_REFRESH_ACTIVE"
fi
