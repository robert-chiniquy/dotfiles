#!/bin/bash
# Vaporwave gradient cycle with breathing fade
COLORS=(
    "0xff0a0a12"  # near black
    "0xff1a0a2e"  # dark purple
    "0xff2d1b4e"  # deep purple
    "0xff4a1a6b"  # purple
    "0xff6b1a8e"  # bright purple
    "0xff8a1aaa"  # purple-magenta
    "0xffaa00e8"  # magenta
    "0xffcc00dd"  # magenta-pink
    "0xffff00f8"  # hot pink
    "0xffff0099"  # pink
    "0xffee1177"  # pink-magenta
    "0xffcc2288"  # deep pink
    "0xff9933aa"  # purple-pink
    "0xff6644cc"  # blue-purple
    "0xff4455dd"  # blue
    "0xff2277ee"  # bright blue
    "0xff00aaff"  # cyan-blue
    "0xff00ddff"  # light cyan
    "0xff5cecff"  # cyan
    "0xff33ccee"  # cyan-teal
    "0xff2299cc"  # teal
    "0xff1166aa"  # dark teal
    "0xff0a4488"  # deep blue
    "0xff0a2266"  # navy
    "0xff0a1144"  # dark navy
    "0xff0a0a22"  # near black blue
    "0xff0a0a12"  # near black
)

# Alpha values for breathing effect (hex)
ALPHAS=("ff" "f0" "e0" "d0" "c0" "b0" "a0" "b0" "c0" "d0" "e0" "f0")

INDEX_FILE="/tmp/sketchybar_border_index"
ALPHA_FILE="/tmp/sketchybar_alpha_index"

if [ -f "$INDEX_FILE" ]; then
    INDEX=$(cat "$INDEX_FILE")
else
    INDEX=0
fi

if [ -f "$ALPHA_FILE" ]; then
    ALPHA_INDEX=$(cat "$ALPHA_FILE")
else
    ALPHA_INDEX=0
fi

COLOR="${COLORS[$INDEX]}"
ALPHA="${ALPHAS[$ALPHA_INDEX]}"

# Apply alpha to border color
BORDER_COLOR="0x${ALPHA}${COLOR:4}"

# Background with breathing alpha
BG_ALPHA=$((16#$ALPHA * 85 / 255))
BG_HEX=$(printf '%02x' $BG_ALPHA)
BG_COLOR="0x${BG_HEX}0a0a12"

NEXT_ALPHA=$(( (ALPHA_INDEX + 1) % ${#ALPHAS[@]} ))
echo $NEXT_ALPHA > "$ALPHA_FILE"

# Only advance color every full alpha cycle
if [ $NEXT_ALPHA -eq 0 ]; then
    NEXT_INDEX=$(( (INDEX + 1) % ${#COLORS[@]} ))
    echo $NEXT_INDEX > "$INDEX_FILE"
fi

sketchybar --bar border_color="$BORDER_COLOR" color="$BG_COLOR"
