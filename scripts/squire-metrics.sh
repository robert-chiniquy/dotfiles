#!/usr/bin/env bash
# squire-metrics.sh -- capture completion metrics for squire dispatches.
#
# Usage:
#   squire-metrics.sh record <env-id> \
#       [--started-at <ISO8601>] \
#       [--branch <branch>] \
#       [--base <base-sha>] \
#       [--env-path <path-in-env>] \
#       [--env-name <name>]
#
# If --started-at is omitted, the timestamp is pulled from
# `squire env info <env-id>` (the `Created:` field). This is the
# canonical env creation time; use it unless you want to scope the
# duration to a sub-period (e.g. you re-dispatched into an existing
# env after the first attempt stalled).
#
#   squire-metrics.sh tally
#   squire-metrics.sh tally --last N
#
# A record bundles:
#   - env_id (positional)
#   - env_name (from `squire env`, or --env-name override)
#   - started_at (caller-provided; the dispatching session knows when
#     `squire new` ran)
#   - completed_at (current UTC time at record invocation)
#   - duration_seconds (completed_at - started_at)
#   - commit_count, files_changed, lines_added, lines_removed
#     (from `git -C <env-path> diff --shortstat <base>..HEAD`)
#
# Output is appended to METRICS_FILE as JSONL (one record per line).
#
# Requires: gh, jq, squire CLI, ssh access to the env (via `squire ssh`).

set -euo pipefail

METRICS_FILE="${SQUIRE_METRICS_FILE:-$HOME/repo/dotfiles/scripts/squire-metrics.jsonl}"

usage() {
  sed -n '2,/^$/p' "$0" | sed 's/^# \{0,1\}//'
  exit 2
}

# Parse ISO 8601 to epoch seconds. Falls back to GNU `date -d` if BSD
# `date -j -f` rejects the format.
to_epoch() {
  local ts="$1"
  # BSD date (macOS): -j -f "%Y-%m-%dT%H:%M:%SZ"
  date -j -u -f "%Y-%m-%dT%H:%M:%SZ" "$ts" +%s 2>/dev/null \
    || date -d "$ts" +%s 2>/dev/null \
    || { echo "squire-metrics: cannot parse timestamp $ts" >&2; return 1; }
}

cmd_record() {
  local env_id="" started_at="" branch="" base="" env_path="/data/squire/src/c1" env_name=""
  if [[ $# -lt 1 ]]; then usage; fi
  env_id="$1"; shift
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --started-at) started_at="$2"; shift 2;;
      --branch)     branch="$2";     shift 2;;
      --base)       base="$2";       shift 2;;
      --env-path)   env_path="$2";   shift 2;;
      --env-name)   env_name="$2";   shift 2;;
      *) echo "unknown flag: $1" >&2; usage;;
    esac
  done
  if [[ -z "$started_at" ]]; then
    # Pull from `squire env info`. Strips sub-second precision because
    # to_epoch expects %Y-%m-%dT%H:%M:%SZ; the fractional seconds in
    # the Created: field would break BSD date parsing.
    # Timestamps contain ':' so we can't naively split on ':'. Match
    # the field name with sed and strip sub-second precision so BSD
    # date can parse the result.
    started_at="$(squire env info "$env_id" 2>/dev/null \
      | sed -nE 's/^Created: *([0-9TZ:.-]+)$/\1/p' \
      | sed -E 's/\.[0-9]+Z$/Z/' \
      | head -1)"
    if [[ -z "$started_at" ]]; then
      echo "squire-metrics: could not derive --started-at from 'squire env info $env_id'" >&2
      exit 2
    fi
  fi

  local completed_at
  completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  local started_epoch completed_epoch duration
  started_epoch="$(to_epoch "$started_at")"
  completed_epoch="$(to_epoch "$completed_at")"
  duration=$((completed_epoch - started_epoch))

  if [[ -z "$env_name" ]]; then
    env_name="$(squire env 2>/dev/null | awk -v id="$env_id" '$0 ~ id {print $2; exit}')"
  fi

  if [[ -z "$branch" ]]; then
    branch="$(squire ssh "$env_id" -- "git -C $env_path branch --show-current" 2>/dev/null | tr -d '\r' | tail -1)"
  fi
  if [[ -z "$base" ]]; then
    # Default base: where the env's branch forked from origin/main.
    base="$(squire ssh "$env_id" -- "git -C $env_path merge-base origin/main $branch" 2>/dev/null | tr -d '\r' | tail -1)"
  fi

  local stats commit_count files_changed lines_added lines_removed
  stats="$(squire ssh "$env_id" -- "git -C $env_path diff --shortstat ${base}..HEAD" 2>/dev/null | tr -d '\r' | tail -1)"
  commit_count="$(squire ssh "$env_id" -- "git -C $env_path rev-list --count ${base}..HEAD" 2>/dev/null | tr -d '\r' | tail -1)"
  files_changed="$(awk '{for(i=1;i<=NF;i++) if($i ~ /file/) print $(i-1)}' <<<"$stats" | head -1)"
  lines_added="$(awk '{for(i=1;i<=NF;i++) if($i ~ /insertion/) print $(i-1)}' <<<"$stats" | head -1)"
  lines_removed="$(awk '{for(i=1;i<=NF;i++) if($i ~ /deletion/) print $(i-1)}' <<<"$stats" | head -1)"
  : "${files_changed:=0}" "${lines_added:=0}" "${lines_removed:=0}" "${commit_count:=0}"

  jq -n \
    --arg env_id "$env_id" \
    --arg env_name "$env_name" \
    --arg started_at "$started_at" \
    --arg completed_at "$completed_at" \
    --argjson duration "$duration" \
    --arg branch "$branch" \
    --arg base "$base" \
    --argjson commit_count "$commit_count" \
    --argjson files_changed "$files_changed" \
    --argjson lines_added "$lines_added" \
    --argjson lines_removed "$lines_removed" \
    '{env_id: $env_id, env_name: $env_name, started_at: $started_at, completed_at: $completed_at, duration_seconds: $duration, branch: $branch, base: $base, commit_count: $commit_count, files_changed: $files_changed, lines_added: $lines_added, lines_removed: $lines_removed}' \
    >> "$METRICS_FILE"

  printf 'recorded: env=%s name=%s duration=%ds commits=%d +%d -%d files=%d\n' \
    "$env_id" "$env_name" "$duration" "$commit_count" "$lines_added" "$lines_removed" "$files_changed"
}

cmd_tally() {
  local last=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --last) last="$2"; shift 2;;
      *) echo "unknown flag: $1" >&2; usage;;
    esac
  done
  if [[ ! -s "$METRICS_FILE" ]]; then
    echo "squire-metrics: $METRICS_FILE is empty or missing" >&2
    return 1
  fi
  local input="$METRICS_FILE"
  if [[ -n "$last" ]]; then
    input="$(tail -n "$last" "$METRICS_FILE")"
  else
    input="$(cat "$METRICS_FILE")"
  fi

  jq -s '
    {
      n: length,
      duration_seconds: {
        min: (map(.duration_seconds) | min),
        median: ( (sort_by(.duration_seconds) | .[length/2 | floor].duration_seconds) ),
        mean: ( (map(.duration_seconds) | add / length) | floor ),
        max: (map(.duration_seconds) | max)
      },
      lines_added: { total: (map(.lines_added) | add), mean: ((map(.lines_added) | add) / length | floor) },
      lines_removed: { total: (map(.lines_removed) | add), mean: ((map(.lines_removed) | add) / length | floor) },
      commits_total: (map(.commit_count) | add),
      files_changed_total: (map(.files_changed) | add)
    }
  ' <<<"$input"
}

cmd="${1:-}"
shift || true
case "$cmd" in
  record) cmd_record "$@";;
  tally)  cmd_tally "$@";;
  ""|-h|--help) usage;;
  *) echo "unknown command: $cmd" >&2; usage;;
esac
