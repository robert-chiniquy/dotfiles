#!/bin/bash
# Monas Hieroglyphica - John Dee's unified symbol
# The glyph represents unity of all creation
# We show it with subtle variations based on hour

HOUR=$(date +%H)

# The Monas itself (Unicode approximation using combining characters)
# Since the actual Monas isn't in Unicode, we use related symbols
# that evoke its lunar/solar/elemental nature

case $((HOUR % 4)) in
    0) SYMBOL="☉" ;;  # Solar (top of Monas)
    1) SYMBOL="☽" ;;  # Lunar (crescent of Monas)
    2) SYMBOL="♀" ;;  # Venus/Copper (cross of Monas)
    3) SYMBOL="☿" ;;  # Mercury (full Monas resembles)
esac

# Color cycles through the day
case $((HOUR / 6)) in
    0) COLOR="0xff5cecff" ;;  # Night - cyan
    1) COLOR="0xfffbb725" ;;  # Morning - gold
    2) COLOR="0xffff0099" ;;  # Afternoon - pink
    3) COLOR="0xffaa00e8" ;;  # Evening - purple
esac

sketchybar --set monas icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
