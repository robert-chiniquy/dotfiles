#!/bin/zsh

set -euo pipefail

wrapper="/Users/rch/repo/dotfiles/bin/codex-limited"
fixed_wrapper="/Users/rch/repo/dotfiles/bin/codex-capped"
probe='require("node:v8").getHeapStatistics().heap_size_limit'

assert_eq() {
  local expected="$1"
  local actual="$2"
  local label="$3"

  if [[ "$expected" != "$actual" ]]; then
    echo "FAIL: $label" >&2
    echo "expected: $expected" >&2
    echo "actual:   $actual" >&2
    exit 1
  fi
}

expected_default="$(
  NODE_OPTIONS='--max-old-space-size=3072' \
  /opt/homebrew/bin/node -p "$probe"
)"
default_limit="$(
  CODEX_REAL_BIN=/opt/homebrew/bin/node \
  "$wrapper" -p "$probe"
)"
assert_eq "$expected_default" "$default_limit" "default heap cap should be applied"

fixed_limit="$(
  CODEX_REAL_BIN=/opt/homebrew/bin/node \
  "$fixed_wrapper" -p "$probe"
)"
assert_eq "$expected_default" "$fixed_limit" "fixed wrapper should apply the default heap cap"

expected_override="$(
  NODE_OPTIONS='--max-old-space-size=4096' \
  /opt/homebrew/bin/node -p "$probe"
)"
override_limit="$(
  CODEX_MAX_OLD_SPACE_MB=4096 \
  CODEX_REAL_BIN=/opt/homebrew/bin/node \
  "$wrapper" -p "$probe"
)"
assert_eq "$expected_override" "$override_limit" "override heap cap should be applied"

if CODEX_MAX_OLD_SPACE_MB=bad CODEX_REAL_BIN=/bin/true "$wrapper" >/dev/null 2>&1; then
  echo "FAIL: invalid CODEX_MAX_OLD_SPACE_MB should be rejected" >&2
  exit 1
fi

echo "PASS: codex-limited"
