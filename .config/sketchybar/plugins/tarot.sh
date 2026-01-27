#!/bin/bash
# Daily tarot - Major Arcana, seeded by date

ARCANA=(
    "0:fool:beginnings"
    "I:magician:will"
    "II:priestess:intuition"
    "III:empress:creation"
    "IV:emperor:structure"
    "V:hierophant:tradition"
    "VI:lovers:choice"
    "VII:chariot:drive"
    "VIII:strength:courage"
    "IX:hermit:solitude"
    "X:wheel:cycles"
    "XI:justice:truth"
    "XII:hanged:surrender"
    "XIII:death:change"
    "XIV:temperance:balance"
    "XV:devil:shadow"
    "XVI:tower:upheaval"
    "XVII:star:hope"
    "XVIII:moon:illusion"
    "XIX:sun:joy"
    "XX:judgement:rebirth"
    "XXI:world:completion"
)

# Seed from date
DAY=$(date +%Y%m%d)
IDX=$((DAY % 22))

IFS=':' read -r NUM NAME MEANING <<< "${ARCANA[$IDX]}"

sketchybar --set tarot icon="$NUM" label.drawing=off icon.color="0xffff0099"
