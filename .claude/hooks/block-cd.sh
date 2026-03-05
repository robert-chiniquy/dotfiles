#!/bin/bash
# Block any Bash command containing "cd " to prevent cd-chaining,
# which times out due to a Claude Code bug.
# Exit 0 = allow, Exit 2 = block (message fed back to Claude).

COMMAND=$(jq -r '.tool_input.command // empty')

if echo "$COMMAND" | grep -qE '(^|[;&|] *)cd '; then
  echo "Blocked: use directory flags (e.g., git -C /path) instead of cd. See CLAUDE.md." >&2
  exit 2
fi

exit 0
