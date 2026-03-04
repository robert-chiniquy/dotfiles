#!/bin/bash
# Setup PaperWM tiling window manager for Hammerspoon
# Revert: git checkout .hammerspoon/init.lua && defaults write com.apple.spaces spans-displays -bool true && killall Dock

set -euo pipefail

DOTFILES="$HOME/repo/dotfiles"
SPOONS_DIR="$DOTFILES/.hammerspoon/Spoons"

echo "=== PaperWM Setup ==="

# 1. macOS: enable separate Spaces per display (required by PaperWM)
echo "Setting displays to have separate Spaces..."
defaults write com.apple.spaces spans-displays -bool false
killall Dock 2>/dev/null || true

# 2. Clone PaperWM.spoon if not present
if [ ! -d "$SPOONS_DIR/PaperWM.spoon" ]; then
    echo "Cloning PaperWM.spoon..."
    mkdir -p "$SPOONS_DIR"
    git clone https://github.com/mogenson/PaperWM.spoon "$SPOONS_DIR/PaperWM.spoon"
else
    echo "PaperWM.spoon already cloned"
fi

# 3. Copy to live Hammerspoon location
echo "Installing to ~/.hammerspoon/Spoons..."
mkdir -p "$HOME/.hammerspoon/Spoons"
cp -r "$SPOONS_DIR/PaperWM.spoon" "$HOME/.hammerspoon/Spoons/"

# 4. Copy init.lua to live location
echo "Installing init.lua..."
cp "$DOTFILES/.hammerspoon/init.lua" "$HOME/.hammerspoon/init.lua"

# 5. Stop skhd if running (dead yabai bindings, replaced by PaperWM)
if brew services list 2>/dev/null | grep -q "skhd.*started"; then
    echo "Stopping skhd..."
    brew services stop koekeishiya/formulae/skhd 2>/dev/null || true
fi

# 6. Reload Hammerspoon
echo "Reloading Hammerspoon..."
hs -c "hs.reload()" 2>/dev/null || echo "  (reload manually with cmd+ctrl+r)"

echo ""
echo "=== Done ==="
echo ""
echo "Keybindings:"
echo "  cmd+ctrl + arrows        Focus navigation"
echo "  cmd+ctrl+shift + arrows  Swap windows"
echo "  cmd+ctrl + c             Center window"
echo "  cmd+ctrl + m             Full width"
echo "  cmd+ctrl+shift + r       Cycle width"
echo "  cmd+ctrl + t             Toggle float"
echo "  cmd+ctrl + 1-5           Switch space"
echo "  cmd+ctrl+shift + 1-5     Move to space"
echo ""
echo "Preserved:"
echo "  cmd+ctrl + f             Toggle focus-follows-mouse"
echo "  cmd+ctrl + r             Reload Hammerspoon"
echo "  cmd+ctrl + w             Rotate wallpaper"
echo "  cmd+ctrl+shift + c       Caffeine toggle"
