#!/usr/bin/env bash
set -euo pipefail

home_dir="${HOME:?HOME must be set}"
claude_dir="${CLAUDE_HOME:-$home_dir/.claude}"
agents_dir="${AGENTS_HOME:-$home_dir/.agents}"
canonical="$claude_dir/skills"
target="$agents_dir/skills"
codex_dir="${CODEX_HOME:-$home_dir/.codex}"
codex_skills="$codex_dir/skills"

if [ ! -d "$canonical" ]; then
  echo "Canonical skill tree not found: $canonical" >&2
  exit 1
fi

canonical_real="$(cd -- "$canonical" && pwd -P)"
mkdir -p "$agents_dir"

if [ -L "$target" ]; then
  target_real="$(cd -- "$target" && pwd -P)"
  if [ "$target_real" != "$canonical_real" ]; then
    echo "Refusing to replace skills symlink with unexpected target: $target -> $target_real" >&2
    exit 1
  fi
  echo "Shared agent skills already current: $target -> $canonical"
else
  if [ -e "$target" ] && [ ! -d "$target" ]; then
    echo "Refusing to replace non-directory agent skills path: $target" >&2
    exit 1
  fi

  if [ -d "$target" ]; then
    shopt -s nullglob
    for entry in "$target"/*; do
      if [ ! -L "$entry" ]; then
        echo "Refusing to hide non-symlink agent skill entry: $entry" >&2
        exit 1
      fi
      if [ ! -d "$entry" ]; then
        echo "Refusing to hide broken agent skill link: $entry" >&2
        exit 1
      fi
      entry_real="$(cd -- "$entry" && pwd -P)"
      case "$entry_real" in
        "$canonical_real"/*) ;;
        *)
          echo "Refusing to hide agent skill outside canonical tree: $entry -> $entry_real" >&2
          exit 1
          ;;
      esac
    done

    backup="$agents_dir/skills.bak-pre-unify-$(date +%Y%m%dT%H%M%S)-$$"
    mv "$target" "$backup"
    echo "Backed up prior agent skill links: $backup"
  fi

  ln -s "$canonical" "$target"
fi

target_real="$(cd -- "$target" && pwd -P)"
if [ "$target_real" != "$canonical_real" ]; then
  echo "Shared agent skill verification failed: $target -> $target_real" >&2
  exit 1
fi

canonical_count="$(find "$canonical" -mindepth 2 -maxdepth 2 -name SKILL.md -type f | wc -l | tr -d ' ')"
target_count="$(find -L "$target" -mindepth 2 -maxdepth 2 -name SKILL.md -type f | wc -l | tr -d ' ')"
if [ "$target_count" -ne "$canonical_count" ]; then
  echo "Shared agent skill count mismatch: canonical=$canonical_count target=$target_count" >&2
  exit 1
fi

overlap_count=0
different_count=0
if [ -d "$codex_skills" ]; then
  shopt -s nullglob
  for skill_dir in "$canonical"/*; do
    [ -f "$skill_dir/SKILL.md" ] || continue
    skill_name="${skill_dir##*/}"
    [ -f "$codex_skills/$skill_name/SKILL.md" ] || continue
    overlap_count=$((overlap_count + 1))
    if ! cmp -s "$skill_dir/SKILL.md" "$codex_skills/$skill_name/SKILL.md"; then
      different_count=$((different_count + 1))
    fi
  done
fi

echo "Installed shared agent skills: $target -> $canonical ($target_count skills)"
if [ "$overlap_count" -gt 0 ]; then
  echo "Codex/OMX also provides $overlap_count same-name skills ($different_count differ); Codex exposes both roots."
fi
