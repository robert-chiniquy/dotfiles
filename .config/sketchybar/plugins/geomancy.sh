#!/bin/bash
# Geomancy - 16 figures of Western geomantic divination
# Each figure is 4 rows of 1 or 2 dots

# The 16 geomantic figures (using braille patterns as approximation)
FIGURES=(
    "⁘:Via:Way"
    "⁙:Populus:People"
    "⁛:Fortuna Major:Great Fortune"
    "⁜:Fortuna Minor:Lesser Fortune"
    "⁖:Acquisitio:Gain"
    "⁗:Amissio:Loss"
    "⁚:Laetitia:Joy"
    "⁝:Tristitia:Sorrow"
    "⁞:Carcer:Prison"
    "⁏:Conjunctio:Union"
    "⁎:Puella:Girl"
    "⁑:Puer:Boy"
    "⁂:Rubeus:Red"
    "⁃:Albus:White"
    "⁐:Caput:Head"
    "⁒:Cauda:Tail"
)

# Seed from date
DAY=$(date +%Y%m%d)
IDX=$((DAY % 16))

IFS=':' read -r SYMBOL NAME MEANING <<< "${FIGURES[$IDX]}"

# Color based on traditional elemental associations
case $((IDX % 4)) in
    0) COLOR="0xfffbb725" ;;  # Fire - gold
    1) COLOR="0xff5cecff" ;;  # Air - cyan
    2) COLOR="0xffaa00e8" ;;  # Water - purple
    3) COLOR="0xffff0099" ;;  # Earth - pink
esac

sketchybar --set geomancy icon="$SYMBOL" label.drawing=off icon.color="$COLOR"
