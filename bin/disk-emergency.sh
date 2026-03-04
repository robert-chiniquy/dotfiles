#!/bin/bash
set -e

echo "=== DISK EMERGENCY ==="
echo "Before: $(df -h / | tail -1 | awk '{print $4}') free"
echo ""

# 1. Go build cache (~56GB) - rebuilds on demand, zero risk
echo "--- go build cache ---"
du -sh ~/Library/Caches/go-build 2>/dev/null || true
go clean -cache
echo "DONE"

# 2. Go module cache (~12GB) - re-downloads on demand
echo "--- go module cache ---"
du -sh ~/go/pkg 2>/dev/null || true
go clean -modcache
echo "DONE"

# 3. gopls + goimports caches (~1.6GB) - regenerate automatically
echo "--- gopls/goimports caches ---"
du -sh ~/Library/Caches/gopls ~/Library/Caches/goimports 2>/dev/null || true
rm -rf ~/Library/Caches/gopls ~/Library/Caches/goimports
echo "DONE"

# 4. Homebrew cache (~757MB) - old downloads
echo "--- homebrew cache ---"
du -sh ~/Library/Caches/Homebrew 2>/dev/null || true
brew cleanup --prune=0 2>/dev/null || true
echo "DONE"

# 5. App updater caches (~1.5GB combined) - stale update downloads
echo "--- app updater caches ---"
rm -rf ~/Library/Caches/com.workflowy.desktop.ShipIt
rm -rf ~/Library/Caches/workflowy-updater
rm -rf ~/Library/Caches/com.microsoft.VSCode.ShipIt
rm -rf ~/Library/Caches/mem-updater
echo "DONE"

echo ""
echo "After: $(df -h / | tail -1 | awk '{print $4}') free"
