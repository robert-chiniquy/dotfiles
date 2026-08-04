#!/bin/sh
set -eu

TOKEN_TEST_ROOT=$(/usr/bin/mktemp -d /private/tmp/token-refresh-test.XXXXXX)
cleanup() {
  /bin/rm -rf -- "$TOKEN_TEST_ROOT"
}
trap cleanup EXIT HUP INT TERM

TOKEN_TEST_SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
TOKEN_TEST_REFRESH="$TOKEN_TEST_SCRIPT_DIR/update-weekly-token-scorecard.sh"
TOKEN_TEST_COLLECTOR="$TOKEN_TEST_ROOT/fake_collector.py"
TOKEN_TEST_STATUS="$TOKEN_TEST_ROOT/status/weekly-agent-tokens.md"
TOKEN_TEST_ACTIVE="$TOKEN_TEST_ROOT/status/status.md"
TOKEN_TEST_REPORT="$TOKEN_TEST_ROOT/data/weekly-agent-tokens.json"

/bin/mkdir -p "$(dirname -- "$TOKEN_TEST_ACTIVE")"

cat > "$TOKEN_TEST_COLLECTOR" <<'PY'
import argparse
import json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--format")
parser.add_argument("--top")
parser.add_argument("--json-output", type=Path, required=True)
args = parser.parse_args()
args.json_output.write_text(json.dumps({"ok": True}) + "\n", encoding="utf-8")
print("# Weekly agent-token footprint")
print("sub: fixture")
print()
print("## Chart: Tokens consumed per repository (millions)")
print("type: histogram")
print("| repository | tokens (M) |")
print("| --- | ---: |")
print("| occult | 42.0 |")
print()
print("## Callouts")
print("| TOP | fixture |")
PY

cat > "$TOKEN_TEST_ACTIVE" <<'MD'
# Ticket scorecard
sub: keep this

<!-- weekly-agent-tokens:begin -->
## Chart: Tokens consumed per repository (millions)
type: histogram
| repository | tokens (M) |
| --- | ---: |
| stale-repo | 1.0 |
<!-- weekly-agent-tokens:end -->

## Tickets
| id | state | score | criterion | note |
| T-1 | risk | 3 | Keep ticket | untouched | grp:tickets |
MD

cat > "$TOKEN_TEST_ROOT/expected-active.md" <<'MD'
# Ticket scorecard
sub: keep this

<!-- weekly-agent-tokens:begin -->
## Chart: Tokens consumed per repository (millions)
type: histogram
| repository | tokens (M) |
| --- | ---: |
| occult | 42.0 |
<!-- weekly-agent-tokens:end -->

## Tickets
| id | state | score | criterion | note |
| T-1 | risk | 3 | Keep ticket | untouched | grp:tickets |
MD

TOKEN_TEST_STDOUT="$TOKEN_TEST_ROOT/stdout"
TOKEN_TEST_STDERR="$TOKEN_TEST_ROOT/stderr"
SCORECARD_TOKEN_FILE="$TOKEN_TEST_STATUS" \
SCORECARD_ACTIVE_FILE="$TOKEN_TEST_ACTIVE" \
TOKEN_REPORT_FILE="$TOKEN_TEST_REPORT" \
TOKEN_REFRESH_COLLECTOR="$TOKEN_TEST_COLLECTOR" \
  "$TOKEN_TEST_REFRESH" > "$TOKEN_TEST_STDOUT" 2> "$TOKEN_TEST_STDERR"

test ! -s "$TOKEN_TEST_STDOUT"
test ! -s "$TOKEN_TEST_STDERR"
/usr/bin/grep -q '^# Weekly agent-token footprint$' "$TOKEN_TEST_STATUS"
/usr/bin/grep -q '"ok": true' "$TOKEN_TEST_REPORT"
/usr/bin/cmp -s "$TOKEN_TEST_ROOT/expected-active.md" "$TOKEN_TEST_ACTIVE"

echo '# Other scorecard' > "$TOKEN_TEST_ACTIVE"
SCORECARD_TOKEN_FILE="$TOKEN_TEST_STATUS" \
SCORECARD_ACTIVE_FILE="$TOKEN_TEST_ACTIVE" \
TOKEN_REPORT_FILE="$TOKEN_TEST_REPORT" \
TOKEN_REFRESH_COLLECTOR="$TOKEN_TEST_COLLECTOR" \
  "$TOKEN_TEST_REFRESH" >/dev/null 2>&1
/usr/bin/grep -qx '# Other scorecard' "$TOKEN_TEST_ACTIVE"

cat > "$TOKEN_TEST_ACTIVE" <<'MD'
# Malformed managed block
<!-- weekly-agent-tokens:begin -->
old chart without an end marker
MD
/bin/cp -f "$TOKEN_TEST_ACTIVE" "$TOKEN_TEST_ROOT/malformed-active.md"
SCORECARD_TOKEN_FILE="$TOKEN_TEST_STATUS" \
SCORECARD_ACTIVE_FILE="$TOKEN_TEST_ACTIVE" \
TOKEN_REPORT_FILE="$TOKEN_TEST_REPORT" \
TOKEN_REFRESH_COLLECTOR="$TOKEN_TEST_COLLECTOR" \
  "$TOKEN_TEST_REFRESH" >/dev/null 2>&1
/usr/bin/cmp -s "$TOKEN_TEST_ROOT/malformed-active.md" "$TOKEN_TEST_ACTIVE"

/bin/rm -f -- "$TOKEN_TEST_ACTIVE"
echo '# Symlink target' > "$TOKEN_TEST_ROOT/symlink-target.md"
/bin/ln -s "$TOKEN_TEST_ROOT/symlink-target.md" "$TOKEN_TEST_ACTIVE"
SCORECARD_TOKEN_FILE="$TOKEN_TEST_STATUS" \
SCORECARD_ACTIVE_FILE="$TOKEN_TEST_ACTIVE" \
TOKEN_REPORT_FILE="$TOKEN_TEST_REPORT" \
TOKEN_REFRESH_COLLECTOR="$TOKEN_TEST_COLLECTOR" \
  "$TOKEN_TEST_REFRESH" >/dev/null 2>&1
/usr/bin/grep -qx '# Symlink target' "$TOKEN_TEST_ROOT/symlink-target.md"

/bin/rm -f -- "$TOKEN_TEST_ACTIVE"
SCORECARD_TOKEN_FILE="$TOKEN_TEST_STATUS" \
SCORECARD_ACTIVE_FILE="$TOKEN_TEST_ACTIVE" \
TOKEN_REPORT_FILE="$TOKEN_TEST_REPORT" \
TOKEN_REFRESH_COLLECTOR="$TOKEN_TEST_COLLECTOR" \
  "$TOKEN_TEST_REFRESH" >/dev/null 2>&1
test ! -e "$TOKEN_TEST_ACTIVE"

echo 'old-status' > "$TOKEN_TEST_STATUS"
echo 'old-report' > "$TOKEN_TEST_REPORT"
/bin/cp -f "$TOKEN_TEST_ROOT/expected-active.md" "$TOKEN_TEST_ACTIVE"
/bin/cp -f "$TOKEN_TEST_ACTIVE" "$TOKEN_TEST_ROOT/old-active.md"
cat > "$TOKEN_TEST_COLLECTOR" <<'PY'
raise SystemExit("fixture failure")
PY

if SCORECARD_TOKEN_FILE="$TOKEN_TEST_STATUS" \
  SCORECARD_ACTIVE_FILE="$TOKEN_TEST_ACTIVE" \
  TOKEN_REPORT_FILE="$TOKEN_TEST_REPORT" \
  TOKEN_REFRESH_COLLECTOR="$TOKEN_TEST_COLLECTOR" \
    "$TOKEN_TEST_REFRESH" >/dev/null 2>&1; then
  echo "expected failed collection" >&2
  exit 1
fi

/usr/bin/grep -qx 'old-status' "$TOKEN_TEST_STATUS"
/usr/bin/grep -qx 'old-report' "$TOKEN_TEST_REPORT"
/usr/bin/cmp -s "$TOKEN_TEST_ROOT/old-active.md" "$TOKEN_TEST_ACTIVE"
