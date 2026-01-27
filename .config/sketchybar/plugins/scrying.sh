#!/bin/bash
# Scrying mirror - Dee used an obsidian mirror for visions
# This indicator reflects system "visions" - recent activity patterns

# Get a hash of recent shell history or git activity
if [ -f ~/.zsh_history ]; then
    # Last 10 commands hashed
    HASH=$(tail -10 ~/.zsh_history 2>/dev/null | md5 | cut -c1-4)
else
    HASH=$(date +%H%M)
fi

# Convert hash to number
NUM=$((16#${HASH}))

# Mirror symbols - reflective/vision related
MIRRORS=(
    "◉" "◎" "●" "○" "◐" "◑" "◒" "◓"
    "◔" "◕" "◖" "◗" "◌" "◍" "◈" "◇"
)

IDX=$((NUM % 16))
SYMBOL="${MIRRORS[$IDX]}"

# Color based on "clarity" (lower hash = clearer vision)
if (( NUM % 4 == 0 )); then
    COLOR="0xffffffff"  # Clear
elif (( NUM % 4 == 1 )); then
    COLOR="0xff5cecff"  # Misty
elif (( NUM % 4 == 2 )); then
    COLOR="0xffaa00e8"  # Clouded
else
    COLOR="0xff333333"  # Dark
fi

sketchybar --set scrying icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
