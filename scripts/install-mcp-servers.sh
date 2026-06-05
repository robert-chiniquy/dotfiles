#!/usr/bin/env bash
# install-mcp-servers.sh -- install every MCP server listed in
# mcp/manifest.yaml at the pinned version. Idempotent.
#
# Agent configs should call the resulting binaries by absolute path:
#   npm:     /opt/homebrew/bin/<bin>
#   uv_git:  ~/.local/bin/<entrypoint>  (uv tool dir; created on first run)
#
# Re-run after editing manifest.yaml. Re-run is safe — npm/uv handle
# already-installed-at-same-version as a noop.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MANIFEST="$SCRIPT_DIR/../mcp/manifest.yaml"

[[ -r "$MANIFEST" ]] || { echo "manifest not found: $MANIFEST" >&2; exit 1; }
command -v yq   >/dev/null || { echo "yq required (brew install yq)" >&2; exit 1; }
command -v npm  >/dev/null || { echo "npm required" >&2; exit 1; }
command -v uvx  >/dev/null || { echo "uv required (brew install uv)" >&2; exit 1; }

echo "=== npm packages ==="
yq -r '.npm[] | "\(.package)\t\(.version)\t\(.bin)"' "$MANIFEST" \
| while IFS=$'\t' read -r pkg ver bin; do
    printf 'pinning %-45s @%-15s ... ' "$pkg" "$ver"
    npm install -g "${pkg}@${ver}" >/dev/null 2>&1
    if command -v "$bin" >/dev/null; then
        echo "ok -> $(command -v "$bin")"
    else
        echo "WARNING: binary '$bin' not on PATH after install"
    fi
done

echo
echo "=== uv_git packages (prime cache so first agent session has no network round-trip) ==="
yq -r '.uv_git[] | "\(.name)\t\(.git)\t\(.entrypoint)"' "$MANIFEST" \
| while IFS=$'\t' read -r name git_url entry; do
    printf 'priming %-12s %-60s ... ' "$name" "$git_url"
    uvx --refresh --from "$git_url" "$entry" --help >/dev/null 2>&1 || true
    echo done
done

echo
echo "=== summary ==="
yq -r '.npm[] | "  npm:     \(.bin)   \(.package)@\(.version)"' "$MANIFEST"
yq -r '.uv_git[] | "  uv_git:  \(.entrypoint)   \(.git)"' "$MANIFEST"
