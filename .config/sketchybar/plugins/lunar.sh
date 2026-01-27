#!/bin/bash
# Lunar phase - actual current moon phase
# This is astronomical, not astrological

# Calculate moon phase (simplified algorithm)
# Based on known new moon date and 29.53 day cycle

# Reference new moon: Jan 11, 2024
REF_NEW_MOON=1704931200  # Unix timestamp
SYNODIC_MONTH=2551443    # 29.53 days in seconds

NOW=$(date +%s)
DIFF=$((NOW - REF_NEW_MOON))
PHASE_SECONDS=$((DIFF % SYNODIC_MONTH))
PHASE_DAYS=$((PHASE_SECONDS / 86400))

# Map to 8 phases
if (( PHASE_DAYS < 2 )); then
    SYMBOL="🌑"
    COLOR="0xff333333"
elif (( PHASE_DAYS < 6 )); then
    SYMBOL="🌒"
    COLOR="0xff444444"
elif (( PHASE_DAYS < 9 )); then
    SYMBOL="🌓"
    COLOR="0xff666666"
elif (( PHASE_DAYS < 13 )); then
    SYMBOL="🌔"
    COLOR="0xffaaaaaa"
elif (( PHASE_DAYS < 16 )); then
    SYMBOL="🌕"
    COLOR="0xffffffff"
elif (( PHASE_DAYS < 20 )); then
    SYMBOL="🌖"
    COLOR="0xffaaaaaa"
elif (( PHASE_DAYS < 24 )); then
    SYMBOL="🌗"
    COLOR="0xff666666"
else
    SYMBOL="🌘"
    COLOR="0xff444444"
fi

sketchybar --set lunar icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
