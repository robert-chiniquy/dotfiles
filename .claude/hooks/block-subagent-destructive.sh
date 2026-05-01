#!/bin/bash
# Block destructive git / rm commands when the Bash tool is invoked by a
# subagent. The main agent is unaffected — it sees no difference.
#
# Rationale: subagents have been observed running `git clean -fd` despite
# explicit prompt instructions not to modify files. The main agent has
# full session context and the user's consent model; subagents are
# expected to be narrow tools and should not be trusted with destructive
# shell commands.
#
# Detection: the PreToolUse payload carries `agent_id` only for calls
# issued by subagents; it is absent for main-agent calls. See
# Claude Code hook documentation.
#
# Exit 0 = allow, Exit 2 = block (stderr fed back to the calling agent).

PAYLOAD=$(cat)
COMMAND=$(printf '%s' "$PAYLOAD" | jq -r '.tool_input.command // empty')
AGENT_ID=$(printf '%s' "$PAYLOAD" | jq -r '.agent_id // empty')

# Main agent: allow everything. This hook is subagent-scoped.
if [ -z "$AGENT_ID" ]; then
  exit 0
fi

AGENT_TYPE=$(printf '%s' "$PAYLOAD" | jq -r '.agent_type // "unknown"')

# Normalize: strip `-C <path>` and `-c key=val` global git flags so
# patterns below can anchor on `git <subcommand>` uniformly. Repeated
# substitution handles `git -C a -c x=y clean`.
NORM=$(printf '%s' "$COMMAND" | \
  sed -E 's/(^|[[:space:];&|`])git[[:space:]]+-C[[:space:]]+[^[:space:]]+/\1git/g' | \
  sed -E 's/(^|[[:space:];&|`])git[[:space:]]+-c[[:space:]]+[^[:space:]]+/\1git/g' | \
  sed -E 's/(^|[[:space:];&|`])git[[:space:]]+-C[[:space:]]+[^[:space:]]+/\1git/g' | \
  sed -E 's/(^|[[:space:];&|`])git[[:space:]]+-c[[:space:]]+[^[:space:]]+/\1git/g')

# Leading boundary: start-of-string, or after a shell separator/opener
# (;, &, |, `, $(, whitespace). Prevents matches inside path fragments
# or quoted literals like `cat /tmp/git-clean-notes.txt`.
BOUND='(^|[[:space:];&|`]|\$\()'

matches() {
  printf '%s' "$NORM" | grep -qE "$1"
}

REASON=""

if   matches "${BOUND}git[[:space:]]+clean([[:space:]]|$)";                   then REASON="git clean"
elif matches "${BOUND}git[[:space:]]+reset[[:space:]]+--hard";                then REASON="git reset --hard"
elif matches "${BOUND}git[[:space:]]+reset[[:space:]]+--merge";               then REASON="git reset --merge"
elif matches "${BOUND}git[[:space:]]+checkout[[:space:]]+(--|\.([[:space:]]|$))"; then REASON="git checkout -- / ."
elif matches "${BOUND}git[[:space:]]+restore[[:space:]]+(--|\.([[:space:]]|$))";  then REASON="git restore -- / ."
elif matches "${BOUND}git[[:space:]]+branch[[:space:]]+-[Dd]([[:space:]]|$)";  then REASON="git branch -D / -d"
elif matches "${BOUND}git[[:space:]]+push[[:space:]].*(--force|-f([[:space:]]|$)|--force-with-lease)"; then REASON="git push --force"
elif matches "${BOUND}git[[:space:]]+stash[[:space:]]+(drop|clear)";          then REASON="git stash drop/clear"
elif matches "${BOUND}git[[:space:]]+rm([[:space:]]|$)";                      then REASON="git rm"
elif matches "${BOUND}git[[:space:]]+worktree[[:space:]]+remove";             then REASON="git worktree remove"
elif matches "${BOUND}rm[[:space:]]+-[^[:space:]]*[rR][^[:space:]]*([[:space:]]|$)"; then REASON="rm -r(f)"
fi

if [ -n "$REASON" ]; then
  cat >&2 <<EOF
Blocked: subagent-issued destructive command.
Agent:     $AGENT_TYPE
Reason:    $REASON
Command:   $COMMAND

Destructive git and rm operations are not available to subagents. If you
need one of these run, report the exact command back to the main agent
and let it decide (it has user context and consent).
EOF
  LOG_DIR="$HOME/.claude/logs"
  mkdir -p "$LOG_DIR" 2>/dev/null
  printf '%s\t%s\t%s\t%s\t%s\n' \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    "$AGENT_TYPE" \
    "$AGENT_ID" \
    "$REASON" \
    "$COMMAND" \
    >> "$LOG_DIR/blocked-subagent-commands.log" 2>/dev/null
  exit 2
fi

exit 0
