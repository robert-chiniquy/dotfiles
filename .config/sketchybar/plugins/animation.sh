#!/bin/bash
# Rotating vaporwave animation
FRAMES=("▁" "▂" "▃" "▄" "▅" "▆" "▇" "█" "▇" "▆" "▅" "▄" "▃" "▂")
INDEX_FILE="/tmp/sketchybar_anim_index"

if [ -f "$INDEX_FILE" ]; then
    INDEX=$(cat "$INDEX_FILE")
else
    INDEX=0
fi

FRAME="${FRAMES[$INDEX]}"
NEXT_INDEX=$(( (INDEX + 1) % ${#FRAMES[@]} ))
echo $NEXT_INDEX > "$INDEX_FILE"

sketchybar --set animation label="$FRAME$FRAME$FRAME"
