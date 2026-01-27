#!/bin/bash
# Kabbalah - Tree of Life Sephiroth
# 10 emanations from Ein Sof, mapped to hour of day

HOUR=$(date +%H)

# Map hours to Sephiroth (roughly follows the lightning flash descent)
# Using modulo to cycle through the 10
IDX=$((HOUR % 10))

# Sephiroth with Hebrew letters and meanings
SEPHIROTH=(
    "כתר:Keter:Crown"
    "חכמה:Chokhmah:Wisdom"
    "בינה:Binah:Understanding"
    "חסד:Chesed:Mercy"
    "גבורה:Gevurah:Strength"
    "תפארת:Tiferet:Beauty"
    "נצח:Netzach:Victory"
    "הוד:Hod:Splendor"
    "יסוד:Yesod:Foundation"
    "מלכות:Malkuth:Kingdom"
)

# Colors follow the traditional associations
COLORS=(
    "0xffffffff"  # Keter - brilliant white
    "0xff5cecff"  # Chokhmah - grey/cyan
    "0xff000000"  # Binah - black (use dark)
    "0xff5cecff"  # Chesed - blue/cyan
    "0xffff0099"  # Gevurah - red/pink
    "0xfffbb725"  # Tiferet - yellow/gold
    "0xffff00f8"  # Netzach - green (use magenta)
    "0xffaa00e8"  # Hod - orange (use purple)
    "0xffaa00e8"  # Yesod - violet/purple
    "0xff5cecff"  # Malkuth - earth tones (use cyan)
)

IFS=':' read -r HEBREW NAME MEANING <<< "${SEPHIROTH[$IDX]}"
COLOR="${COLORS[$IDX]}"

# Show Hebrew letter as icon, transliteration as label
sketchybar --set sephirah icon="$HEBREW" label.drawing=off icon.color="$COLOR"
