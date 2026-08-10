#!/bin/sh
# agent-wt-status.sh [repo-root]
# One-shot, read-only status of every agent worktree under <repo>/.claude/worktrees.
# Reports per worktree: branch, commits ahead of origin/main, dirty files,
# newest non-.git file (the liveness signal), and the last PROGRESS.md line.
# Also lists running go test/build processes. No arguments needed for occult.

ROOT="${1:-/Users/rch/repo/occult}"
WT_DIR="$ROOT/.claude/worktrees"

echo "== agent worktrees under $WT_DIR =="
if [ ! -d "$WT_DIR" ] || [ -z "$(ls -A "$WT_DIR" 2>/dev/null)" ]; then
  echo "(none)"
else
  for w in "$WT_DIR"/*/; do
    name=$(basename "$w")
    branch=$(git -C "$w" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "?")
    ahead=$(git -C "$w" rev-list --count origin/main..HEAD 2>/dev/null || echo "?")
    dirty=$(git -C "$w" status --short 2>/dev/null | wc -l | tr -d ' ')
    last=$(git -C "$w" log -1 --format='%h %s' 2>/dev/null | cut -c1-70)
    newest=$(find "$w" -path "$w.git" -prune -o -type f -print 2>/dev/null \
      | xargs stat -f '%m %N' 2>/dev/null | sort -rn | head -1)
    newest_age=""
    if [ -n "$newest" ]; then
      ts=${newest%% *}
      f=${newest#* }
      now=$(date +%s)
      mins=$(( (now - ts) / 60 ))
      newest_age="${mins}m ago: ${f#"$w"}"
    fi
    echo "-- $name"
    echo "   branch=$branch ahead=$ahead dirty=$dirty"
    echo "   last-commit: $last"
    echo "   newest-file: ${newest_age:-none}"
    if [ -f "$w/PROGRESS.md" ]; then
      echo "   progress: $(tail -1 "$w/PROGRESS.md")"
    fi
  done
fi

echo "== running go test/build processes =="
ps aux | grep -E '[o]ccult.*\.test|[g]o (test|build|vet)' | awk '{printf "   pid=%s cpu=%s %s %s %s\n", $2, $10, $11, $12, $13}' | head -8
echo "== done $(date '+%H:%M:%S') =="
