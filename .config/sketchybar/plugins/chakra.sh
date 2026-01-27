#!/bin/bash
# Chakras - 7 energy centers (Sanskrit tradition)
# Position based on time of day (root morning, crown evening)

HOUR=$(date +%H)

# Map 24 hours to 7 chakras (roughly 3.4 hours each)
if (( HOUR < 4 )); then
    SYMBOL="॰"  # Muladhara (root)
    COLOR="0xffff0099"  # Red/pink
elif (( HOUR < 7 )); then
    SYMBOL="॰॰"  # Svadhisthana (sacral)
    COLOR="0xfffbb725"  # Orange/gold
elif (( HOUR < 10 )); then
    SYMBOL="॰॰॰"  # Manipura (solar plexus)
    COLOR="0xfffbb725"  # Yellow/gold
elif (( HOUR < 14 )); then
    SYMBOL="❤"  # Anahata (heart)
    COLOR="0xff5cecff"  # Green (use cyan)
elif (( HOUR < 17 )); then
    SYMBOL="◉"  # Vishuddha (throat)
    COLOR="0xff5cecff"  # Blue/cyan
elif (( HOUR < 20 )); then
    SYMBOL="◎"  # Ajna (third eye)
    COLOR="0xffaa00e8"  # Indigo/purple
else
    SYMBOL="✴"  # Sahasrara (crown)
    COLOR="0xffff00f8"  # Violet/magenta
fi

sketchybar --set chakra icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
