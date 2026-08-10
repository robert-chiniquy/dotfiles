#!/bin/bash
# Block shell-level `cd` chaining, which times out due to a Claude Code bug.
# Exit 0 = allow, Exit 2 = block (message fed back to Claude).
#
# Quoted spans and heredoc bodies are stripped before matching, because a `cd`
# inside them is data, not a directory change. A regex alternation like
# grep -E "a|cd ~|b", a commit message, or an echo payload would otherwise read
# as a separator followed by cd and block a command that never changes
# directory. Stripping is whole-input (not line-by-line) so multi-line strings
# and heredoc bodies are covered.

COMMAND=$(jq -r '.tool_input.command // empty')

STRIPPED=$(printf '%s' "$COMMAND" | perl -0777 -pe '
  # Heredoc: keep the marker line, drop the body through its terminator.
  s/(<<-?\s*['"'"'"]?(\w+)['"'"'"]?[^\n]*\n).*?\n[ \t]*\2[ \t]*$/$1/gms;
  # Quoted spans, including across newlines.
  s/'"'"'[^'"'"']*'"'"'//gs;
  s/"[^"]*"//gs;
')

# `cd` as a command word: at input start, or after a separator (; & && | ||).
# The trailing boundary keeps `abcd` and `cdrom` from matching.
if printf '%s' "$STRIPPED" | grep -qE '(^|[;&|][[:space:]]*)cd([[:space:]]|$)'; then
  echo "Blocked: use directory flags (e.g., git -C /path) instead of cd. See CLAUDE.md." >&2
  exit 2
fi

exit 0
