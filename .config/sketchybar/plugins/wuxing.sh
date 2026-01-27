#!/bin/bash
# Wu Xing - Chinese Five Elements/Phases
# Cycle based on creative/destructive cycle and time

HOUR=$(date +%H)

# Five phases in generative order
IDX=$((HOUR % 5))

case $IDX in
    0)  # Wood - growth, spring
        SYMBOL="木"
        COLOR="0xff5cecff"  # Green (cyan)
        ;;
    1)  # Fire - expansion, summer
        SYMBOL="火"
        COLOR="0xffff0099"  # Red (pink)
        ;;
    2)  # Earth - stability, center
        SYMBOL="土"
        COLOR="0xfffbb725"  # Yellow (gold)
        ;;
    3)  # Metal - contraction, autumn
        SYMBOL="金"
        COLOR="0xffffffff"  # White
        ;;
    4)  # Water - stillness, winter
        SYMBOL="水"
        COLOR="0xffaa00e8"  # Black (purple)
        ;;
esac

sketchybar --set wuxing icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
