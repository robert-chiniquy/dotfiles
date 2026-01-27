#!/bin/bash
# Gradient clock - slowly shifts color through vaporwave palette

# Vaporwave gradient: pink -> purple -> cyan -> gold -> back
# Using RGB interpolation

CACHE_FILE="/tmp/sketchybar_gradient_phase"
PHASE=$(cat "$CACHE_FILE" 2>/dev/null || echo "0")

# Increment phase (0-359 degrees around color wheel)
PHASE=$(( (PHASE + 2) % 360 ))
echo "$PHASE" > "$CACHE_FILE"

# Map phase to vaporwave colors with smooth interpolation
# 0-90: pink to purple
# 90-180: purple to cyan
# 180-270: cyan to gold
# 270-360: gold to pink

if (( PHASE < 90 )); then
    # Pink (ff0099) to Purple (aa00e8)
    t=$((PHASE * 100 / 90))
    r=$((255 - (255 - 170) * t / 100))
    g=0
    b=$((153 + (232 - 153) * t / 100))
elif (( PHASE < 180 )); then
    # Purple (aa00e8) to Cyan (5cecff)
    t=$(((PHASE - 90) * 100 / 90))
    r=$((170 - (170 - 92) * t / 100))
    g=$((0 + 236 * t / 100))
    b=$((232 + (255 - 232) * t / 100))
elif (( PHASE < 270 )); then
    # Cyan (5cecff) to Gold (fbb725)
    t=$(((PHASE - 180) * 100 / 90))
    r=$((92 + (251 - 92) * t / 100))
    g=$((236 - (236 - 183) * t / 100))
    b=$((255 - (255 - 37) * t / 100))
else
    # Gold (fbb725) to Pink (ff0099)
    t=$(((PHASE - 270) * 100 / 90))
    r=$((251 + (255 - 251) * t / 100))
    g=$((183 - 183 * t / 100))
    b=$((37 + (153 - 37) * t / 100))
fi

# Convert to hex
COLOR=$(printf "0xff%02x%02x%02x" $r $g $b)

# Update clock
TIME=$(date "+%H:%M")
sketchybar --set clock label="$TIME" icon.color="$COLOR" label.color="$COLOR"
