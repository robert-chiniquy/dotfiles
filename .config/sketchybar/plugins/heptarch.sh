#!/bin/bash
# Heptarchic Angels - John Dee's 7 planetary angels
# Each rules a day of the week in Dee's system
# Names from the Heptarchia Mystica

# Day of week (1=Monday, 7=Sunday)
DOW=$(date +%u)

case $DOW in
    1)  # Monday - Luna
        ANGEL="Blumaza"
        SYMBOL="☽"
        COLOR="0xffffffff"  # Silver/white
        ;;
    2)  # Tuesday - Mars
        ANGEL="Befafes"
        SYMBOL="♂"
        COLOR="0xffff0099"  # Red/pink
        ;;
    3)  # Wednesday - Mercury
        ANGEL="Bnaspol"
        SYMBOL="☿"
        COLOR="0xff5cecff"  # Cyan
        ;;
    4)  # Thursday - Jupiter
        ANGEL="Bynepor"
        SYMBOL="♃"
        COLOR="0xffaa00e8"  # Purple
        ;;
    5)  # Friday - Venus
        ANGEL="Babalel"
        SYMBOL="♀"
        COLOR="0xffff00f8"  # Magenta
        ;;
    6)  # Saturday - Saturn
        ANGEL="Bnapsen"
        SYMBOL="♄"
        COLOR="0xff444444"  # Dark grey
        ;;
    7)  # Sunday - Sol
        ANGEL="Bobogel"
        SYMBOL="☉"
        COLOR="0xfffbb725"  # Gold
        ;;
esac

sketchybar --set heptarch icon="$SYMBOL" label.drawing=off icon.color="$COLOR"
