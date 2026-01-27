#!/bin/bash
# Watchtower quadrant - The four Enochian tablets
# East (Air), South (Fire), West (Water), North (Earth)
# Plus the Tablet of Union in the center
# Quadrant shown based on time of day

HOUR=$(date +%H)

# Map hours to quadrants (6-hour periods)
if (( HOUR >= 6 && HOUR < 12 )); then
    # Morning - East/Air (rising)
    SYMBOL="ᚨ"  # Air-like rune
    ELEMENT="Air"
    COLOR="0xffffffff"  # White
elif (( HOUR >= 12 && HOUR < 18 )); then
    # Afternoon - South/Fire (zenith)
    SYMBOL="ᚠ"  # Fire-like rune
    ELEMENT="Fire"
    COLOR="0xfffbb725"  # Gold
elif (( HOUR >= 18 && HOUR < 24 )); then
    # Evening - West/Water (setting)
    SYMBOL="ᛚ"  # Water-like rune
    ELEMENT="Water"
    COLOR="0xff5cecff"  # Cyan
else
    # Night - North/Earth (nadir)
    SYMBOL="ᛏ"  # Earth-like rune
    ELEMENT="Earth"
    COLOR="0xffaa00e8"  # Purple
fi

# At liminal hours (0, 6, 12, 18) show Tablet of Union
MIN=$(date +%M)
if (( HOUR % 6 == 0 && MIN < 15 )); then
    SYMBOL="⊕"
    ELEMENT="Union"
    COLOR="0xffff0099"
fi

sketchybar --set watchtower icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
