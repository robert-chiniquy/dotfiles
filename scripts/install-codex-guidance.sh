#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
policy_file="${CODEX_POLICY_FILE:-$repo_root/codex/AGENTS.policy.md}"
codex_dir="${CODEX_HOME:-$HOME/.codex}"
target="$codex_dir/AGENTS.md"
start_marker='<!-- USER:OMX:POLICY:START -->'
end_marker='<!-- USER:OMX:POLICY:END -->'

marker_count() {
  awk -v marker="$2" '$0 == marker { count++ } END { print count + 0 }' "$1"
}

if [ ! -f "$policy_file" ]; then
  echo "Codex policy not found: $policy_file" >&2
  exit 1
fi

if [ "$(marker_count "$policy_file" "$start_marker")" -ne 1 ] ||
   [ "$(marker_count "$policy_file" "$end_marker")" -ne 1 ]; then
  echo "Codex policy must contain exactly one complete OMX user-policy block" >&2
  exit 1
fi

mkdir -p "$codex_dir"

if [ -L "$target" ]; then
  echo "Refusing to replace symlinked Codex guidance: $target" >&2
  exit 1
fi

tmp="$(mktemp "$codex_dir/.AGENTS.md.tmp.XXXXXX")"
trap 'rm -f "$tmp"' EXIT

if [ -f "$target" ]; then
  awk -v start="$start_marker" -v end="$end_marker" '
    function flush_blanks() {
      for (i = 0; i < pending_blanks; i++) {
        print ""
      }
      pending_blanks = 0
    }

    $0 == start {
      if (skipping) {
        exit 42
      }
      skipping = 1
      next
    }

    $0 == end {
      if (!skipping) {
        exit 42
      }
      skipping = 0
      next
    }

    skipping {
      next
    }

    /^[[:space:]]*$/ {
      pending_blanks++
      next
    }

    {
      flush_blanks()
      print
    }

    END {
      if (skipping) {
        exit 42
      }
    }
  ' "$target" > "$tmp"
fi

if [ -s "$tmp" ]; then
  printf '\n\n' >> "$tmp"
fi
cat "$policy_file" >> "$tmp"
printf '\n' >> "$tmp"
chmod 0644 "$tmp"

if [ "$(marker_count "$tmp" "$start_marker")" -ne 1 ] ||
   [ "$(marker_count "$tmp" "$end_marker")" -ne 1 ]; then
  echo "Generated Codex guidance failed policy-marker validation" >&2
  exit 1
fi

if [ -f "$target" ] &&
   grep -Fq '<!-- omx:generated:agents-md -->' "$target" &&
   ! grep -Fq '<!-- omx:generated:agents-md -->' "$tmp"; then
  echo "Generated Codex guidance lost the OMX contract marker" >&2
  exit 1
fi

if [ -f "$target" ]; then
  old_size="$(wc -c < "$target" | tr -d ' ')"
  new_size="$(wc -c < "$tmp" | tr -d ' ')"
  if [ "$old_size" -gt 0 ] && [ $((new_size * 2)) -lt "$old_size" ]; then
    echo "Generated Codex guidance failed size-floor validation" >&2
    exit 1
  fi

  if cmp -s "$target" "$tmp"; then
    echo "Codex guidance already current"
    exit 0
  fi

  backup="$target.bak.pre-codex-guidance-$(date +%Y%m%dT%H%M%S)"
  cp -p "$target" "$backup"
fi

mv "$tmp" "$target"
trap - EXIT
echo "Installed Codex guidance: $target"
