#!/bin/bash
# VHS tracking line - animates across the bar

# Colors cycle through vaporwave palette
COLORS=("0xffff0099" "0xff5cecff" "0xffff00f8" "0xfffbb725" "0xffaa00e8")

# Get current position from cache or start at 0
CACHE_FILE="/tmp/sketchybar_vhs_pos"
POS=$(cat "$CACHE_FILE" 2>/dev/null || echo "0")
COLOR_IDX=$(cat "/tmp/sketchybar_vhs_color" 2>/dev/null || echo "0")

# Bar width approximation
BAR_WIDTH=1400
LINE_WIDTH=60

# Move position
POS=$((POS + 15))
if (( POS > BAR_WIDTH )); then
    POS=0
    # Cycle color on wrap
    COLOR_IDX=$(( (COLOR_IDX + 1) % ${#COLORS[@]} ))
    echo "$COLOR_IDX" > /tmp/sketchybar_vhs_color
fi
echo "$POS" > "$CACHE_FILE"

# Update the line item
sketchybar --set vhs_line \
    icon.padding_left=$POS \
    icon.color=${COLORS[$COLOR_IDX]}
