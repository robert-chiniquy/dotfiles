#!/bin/sh
# agent-wt-watch.sh <worktree-name-or-path> [repo-root] [heartbeat-secs]
# Event stream for one agent worktree, designed for the Monitor tool:
# emits one line on each stage change (PROGRESS.md tail) and each new
# commit (count ahead of origin/main), and a terminal line when the
# worktree disappears. With heartbeat-secs set, also emits a HEARTBEAT
# summary line every N seconds regardless of change (a reporting
# cadence). Poll interval 60s, local-only, read-only.

ROOT="${2:-/Users/rch/repo/occult}"
HB="${3:-0}"
W="$1"
case "$W" in
  /*) ;;
  *) W="$ROOT/.claude/worktrees/$W" ;;
esac
[ -n "$1" ] || { echo "usage: agent-wt-watch.sh <worktree-name-or-path> [repo-root] [heartbeat-secs]" >&2; exit 2; }

prev_progress=""
prev_ahead=""
since_hb=0
while true; do
  if [ ! -d "$W" ]; then
    echo "WORKTREE GONE (agent finished or cleaned up): $W"
    exit 0
  fi
  progress=$(tail -1 "$W/PROGRESS.md" 2>/dev/null)
  ahead=$(git -C "$W" rev-list --count origin/main..HEAD 2>/dev/null)
  if [ -n "$prev_progress" ] && [ "$progress" != "$prev_progress" ]; then
    echo "STAGE CHANGE: $progress"
    since_hb=0
  fi
  if [ -n "$prev_ahead" ] && [ "$ahead" != "$prev_ahead" ]; then
    echo "NEW COMMIT (ahead=$ahead): $(git -C "$W" log -1 --format='%h %s' 2>/dev/null | cut -c1-90)"
    since_hb=0
  fi
  if [ "$HB" -gt 0 ] && [ "$since_hb" -ge "$HB" ]; then
    dirty=$(git -C "$W" status --short 2>/dev/null | wc -l | tr -d ' ')
    newest=$(find "$W" -name .git -prune -o -type f -print 2>/dev/null \
      | xargs stat -f '%m %N' 2>/dev/null | sort -rn | head -1)
    age="unknown"
    if [ -n "$newest" ]; then
      ts=${newest%% *}; f=${newest#* }
      age="$(( ($(date +%s) - ts) / 60 ))m ago: ${f#"$W"/}"
    fi
    procs=$(ps aux | grep -cE '[o]ccult.*\.test|[g]o (test|build|vet)')
    echo "HEARTBEAT ahead=$ahead dirty=$dirty procs=$procs newest=$age | $(echo "$progress" | cut -c1-140)"
    since_hb=0
  fi
  prev_progress="$progress"
  prev_ahead="$ahead"
  sleep 60
  since_hb=$((since_hb + 60))
done
