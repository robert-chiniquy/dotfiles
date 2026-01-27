#!/bin/bash
# Daily rune - Elder Futhark, seeded by date

RUNES=(
    "ᚠ:fehu:wealth"
    "ᚢ:uruz:strength"
    "ᚦ:thurisaz:force"
    "ᚨ:ansuz:wisdom"
    "ᚱ:raido:journey"
    "ᚲ:kenaz:torch"
    "ᚷ:gebo:gift"
    "ᚹ:wunjo:joy"
    "ᚺ:hagalaz:hail"
    "ᚾ:nauthiz:need"
    "ᛁ:isa:ice"
    "ᛃ:jera:harvest"
    "ᛇ:eihwaz:yew"
    "ᛈ:perthro:mystery"
    "ᛉ:algiz:protection"
    "ᛊ:sowilo:sun"
    "ᛏ:tiwaz:victory"
    "ᛒ:berkano:growth"
    "ᛖ:ehwaz:horse"
    "ᛗ:mannaz:self"
    "ᛚ:laguz:water"
    "ᛜ:ingwaz:seed"
    "ᛞ:dagaz:dawn"
    "ᛟ:othala:heritage"
)

# Seed from date
DAY=$(date +%Y%m%d)
IDX=$((DAY % 24))

IFS=':' read -r SYMBOL NAME MEANING <<< "${RUNES[$IDX]}"

sketchybar --set rune icon="$SYMBOL" label="$MEANING" icon.color="0xffffffff" label.color="0xff666666"
