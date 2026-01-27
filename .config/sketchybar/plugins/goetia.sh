#!/bin/bash
# Goetia - 72 spirits from the Lesser Key of Solomon
# Each spirit has a seal, rank, and domain

# Selected notable spirits (using mystical Unicode as seal approximations)
SPIRITS=(
    "1:Bael:King:⛧"
    "2:Agares:Duke:⍟"
    "9:Paimon:King:⎔"
    "13:Beleth:King:⏣"
    "17:Botis:President:⟐"
    "20:Purson:King:⧫"
    "21:Marax:Earl:⟁"
    "32:Asmoday:King:⩕"
    "45:Vine:King:⬖"
    "51:Balam:King:⭔"
    "55:Orobas:Prince:✦"
    "58:Amy:President:⦿"
    "61:Zagan:King:⌬"
    "68:Belial:King:⎊"
    "70:Seere:Prince:⏃"
    "72:Andromalius:Earl:⏏"
)

# Seed from date
DAY=$(date +%Y%m%d)
IDX=$((DAY % 16))

IFS=':' read -r NUM NAME RANK SEAL <<< "${SPIRITS[$IDX]}"

# Color by rank
case "$RANK" in
    "King") COLOR="0xfffbb725" ;;      # Gold for kings
    "Prince") COLOR="0xffaa00e8" ;;    # Purple for princes
    "Duke") COLOR="0xff5cecff" ;;      # Cyan for dukes
    "Earl") COLOR="0xffff0099" ;;      # Pink for earls
    *) COLOR="0xffffffff" ;;           # White for others
esac

sketchybar --set goetia icon="$SEAL" label.drawing=off icon.color="$COLOR"
