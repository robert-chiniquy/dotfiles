#!/bin/bash
# Angelic Choirs - 9 orders from Pseudo-Dionysius
# Maps to hour of day in descending hierarchy

HOUR=$(date +%H)

# 9 choirs, cycle through day
IDX=$((HOUR % 9))

case $IDX in
    0)  # Seraphim - highest, burning ones
        SYMBOL="𖤍"
        COLOR="0xffff0099"
        ;;
    1)  # Cherubim - knowledge
        SYMBOL="𖤐"
        COLOR="0xfffbb725"
        ;;
    2)  # Thrones - divine justice
        SYMBOL="⌬"
        COLOR="0xfffbb725"
        ;;
    3)  # Dominions - leadership
        SYMBOL="♔"
        COLOR="0xffaa00e8"
        ;;
    4)  # Virtues - movement of stars
        SYMBOL="✧"
        COLOR="0xff5cecff"
        ;;
    5)  # Powers - warrior angels
        SYMBOL="⚔"
        COLOR="0xffff0099"
        ;;
    6)  # Principalities - nations
        SYMBOL="⚜"
        COLOR="0xfffbb725"
        ;;
    7)  # Archangels - messengers
        SYMBOL="𐤀"
        COLOR="0xffffffff"
        ;;
    8)  # Angels - guardians
        SYMBOL="◬"
        COLOR="0xff5cecff"
        ;;
esac

sketchybar --set choir icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
