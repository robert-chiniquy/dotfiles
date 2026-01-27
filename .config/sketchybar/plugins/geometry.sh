#!/bin/bash
# Sacred geometry - responds to system activity

# Get CPU as activity proxy
CPU=$(top -l 1 | grep "CPU usage" | awk '{print int($3)}')

# Map activity to geometric forms (simple to complex)
if (( CPU < 10 )); then
    # Stillness - point/void
    SYMBOL="·"
    COLOR="0xff333333"
elif (( CPU < 25 )); then
    # Low - line/duality
    SYMBOL="│"
    COLOR="0xff444444"
elif (( CPU < 40 )); then
    # Moderate - triangle
    SYMBOL="△"
    COLOR="0xff5cecff"
elif (( CPU < 55 )); then
    # Active - square
    SYMBOL="□"
    COLOR="0xfffbb725"
elif (( CPU < 70 )); then
    # High - pentagon
    SYMBOL="⬠"
    COLOR="0xffaa00e8"
elif (( CPU < 85 )); then
    # Intense - hexagon (harmony)
    SYMBOL="⬡"
    COLOR="0xffff0099"
else
    # Peak - octagon (completion)
    SYMBOL="⯃"
    COLOR="0xffff00f8"
fi

sketchybar --set geometry icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
