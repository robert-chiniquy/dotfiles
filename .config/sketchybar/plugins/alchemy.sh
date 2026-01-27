#!/bin/bash
# Alchemical symbol - based on current work phase

# Read phase from shell's export (set by _detect_phase in zshrc)
PHASE="${__current_phase:-}"

case "$PHASE" in
    "exploring")
        # Mercury - exploration, communication
        SYMBOL="☿"
        COLOR="0xff5cecff"  # cyan
        ;;
    "writing")
        # Air/Quill - creation
        SYMBOL="🜁"
        COLOR="0xffffffff"  # white
        ;;
    "testing")
        # Fire - trial, transformation
        SYMBOL="🜂"
        COLOR="0xfffbb725"  # gold
        ;;
    "debugging")
        # Earth - grounding, fixing
        SYMBOL="🜃"
        COLOR="0xffaa00e8"  # purple
        ;;
    "building")
        # Crucible - forging
        SYMBOL="🝊"
        COLOR="0xffff0099"  # pink
        ;;
    *)
        # Prima materia - undifferentiated
        SYMBOL="🜔"
        COLOR="0xff444444"  # grey
        ;;
esac

sketchybar --set alchemy icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
