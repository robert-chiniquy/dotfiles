#!/bin/bash
# Enochian alphabet - John Dee's angelic script
# 21 letters revealed through Edward Kelley's scrying
# We cycle through based on combined date/hour

# Enochian letters (using closest Unicode approximations)
# The actual Enochian script isn't in Unicode, so we use
# visually similar archaic/mystical characters
LETTERS=(
    "B"   # Un - Pa
    "C"   # Veh - Gon
    "G"   # Ged - Graph
    "D"   # Gal - Orth
    "F"   # Or - Na
    "A"   # Tal - Ur
    "E"   # Gon - Mals
    "M"   # Na - Ger
    "I"   # Gon - Drux
    "H"   # Med - Fam
    "L"   # Ur - Tal
    "P"   # Mals - Don
    "Q"   # Ger - Ceph
    "N"   # Drux - Vau
    "X"   # Pal - Graph
    "O"   # Med - Gisa
    "R"   # Don - Med
    "Z"   # Ceph - Gisg
    "U"   # Van - Na
    "S"   # Fam - Graph
    "T"   # Gisg - Med
)

# Use arcane-looking Unicode characters that evoke the angular Enochian script
GLYPHS=(
    "ᛒ" "ᚲ" "ᚷ" "ᛞ" "ᚠ" "ᚨ" "ᛖ" "ᛗ" "ᛁ" "ᚺ"
    "ᛚ" "ᛈ" "ᛩ" "ᚾ" "ᛪ" "ᛟ" "ᚱ" "ᛉ" "ᚢ" "ᛊ" "ᛏ"
)

# Seed from date and hour for slow cycling
DAY=$(date +%Y%m%d)
HOUR=$(date +%H)
IDX=$(( (DAY + HOUR) % 21 ))

GLYPH="${GLYPHS[$IDX]}"

# Color based on letter's position in the alphabet
# Earlier = cyan, later = pink (spectrum walk)
if (( IDX < 7 )); then
    COLOR="0xff5cecff"
elif (( IDX < 14 )); then
    COLOR="0xfffbb725"
else
    COLOR="0xffff0099"
fi

sketchybar --set enochian icon="$GLYPH" icon.color="$COLOR" label.drawing=off
