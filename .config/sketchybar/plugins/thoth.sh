#!/bin/bash
# Thoth/Hermes - Egyptian-Greek wisdom tradition
# Symbols from the Hermetic tradition and Egyptian magic

# Get process count as "knowledge level"
PROCS=$(ps aux | wc -l | tr -d ' ')

# Symbols progress from simple to complex based on activity
if (( PROCS < 100 )); then
    # Minimal - Ankh (life)
    SYMBOL="☥"
    NAME="Ankh"
    COLOR="0xff5cecff"
elif (( PROCS < 200 )); then
    # Low - Eye of Horus (protection)
    SYMBOL="𓂀"
    NAME="Wadjet"
    COLOR="0xff5cecff"
elif (( PROCS < 300 )); then
    # Moderate - Caduceus (commerce/negotiation)
    SYMBOL="☤"
    NAME="Caduceus"
    COLOR="0xfffbb725"
elif (( PROCS < 400 )); then
    # Active - Djed (stability)
    SYMBOL="𓊽"
    NAME="Djed"
    COLOR="0xfffbb725"
elif (( PROCS < 500 )); then
    # High - Was scepter (power)
    SYMBOL="𓌀"
    NAME="Was"
    COLOR="0xffaa00e8"
else
    # Peak - Ibis (Thoth himself)
    SYMBOL="𓅜"
    NAME="Ibis"
    COLOR="0xffff0099"
fi

sketchybar --set thoth icon="$SYMBOL" label.drawing=off icon.color="$COLOR"
