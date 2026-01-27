#!/bin/bash
# Hermetic Principles - 7 laws from the Kybalion
# Each principle governs an aspect of reality

PRINCIPLES=(
    "⊙:Mentalism:All is Mind"
    "⇅:Correspondence:As above so below"
    "〰:Vibration:Nothing rests"
    "☯:Polarity:All is dual"
    "⟳:Rhythm:Everything flows"
    "⚡:Cause:Every cause has effect"
    "☿:Gender:All has masculine and feminine"
)

# Map day of week to principle (7 days, 7 principles)
DOW=$(date +%u)
IDX=$((DOW - 1))

IFS=':' read -r SYMBOL NAME DESC <<< "${PRINCIPLES[$IDX]}"

# Colors based on hermetic planetary associations
COLORS=(
    "0xfffbb725"  # Sun - gold (Mentalism)
    "0xffffffff"  # Moon - white (Correspondence)
    "0xff5cecff"  # Mercury - cyan (Vibration)
    "0xffff00f8"  # Venus - magenta (Polarity)
    "0xffff0099"  # Mars - pink (Rhythm)
    "0xffaa00e8"  # Jupiter - purple (Cause)
    "0xff444444"  # Saturn - grey (Gender)
)

COLOR="${COLORS[$IDX]}"

sketchybar --set hermetic icon="$SYMBOL" label.drawing=off icon.color="$COLOR"
