#!/bin/bash
# Theban Alphabet - "Witch's alphabet" / Honorian script
# Used in modern witchcraft for writing spells

# Theban doesn't have Unicode, use visually similar runes/symbols
# These approximate the angular Theban letterforms
LETTERS=(
    "ᚫ" "ᛒ" "ᚳ" "ᛞ" "ᛖ" "ᚠ" "ᚷ" "ᚻ" "ᛁ" "ᛄ" "ᛣ" "ᛚ" "ᛗ"
    "ᚾ" "ᚩ" "ᛈ" "ᛢ" "ᚱ" "ᛋ" "ᛏ" "ᚢ" "ᚥ" "ᚹ" "ᛉ" "ᛦ" "ᛎ"
)

# Seed from minute for faster cycling
MIN=$(date +%M)
IDX=$((MIN % 26))

LETTER="${LETTERS[$IDX]}"

# Color based on position in alphabet
if (( IDX < 9 )); then
    COLOR="0xff5cecff"
elif (( IDX < 18 )); then
    COLOR="0xffaa00e8"
else
    COLOR="0xffff0099"
fi

sketchybar --set theban icon="$LETTER" icon.color="$COLOR" label.drawing=off
