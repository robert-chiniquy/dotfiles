#!/bin/bash
# Ogham - Celtic tree alphabet
# 20 letters (feda), each associated with a tree

# Ogham letters with Unicode characters
OGHAM=(
    "ᚁ:Beith:Birch"
    "ᚂ:Luis:Rowan"
    "ᚃ:Fearn:Alder"
    "ᚄ:Sail:Willow"
    "ᚅ:Nion:Ash"
    "ᚆ:Uath:Hawthorn"
    "ᚇ:Dair:Oak"
    "ᚈ:Tinne:Holly"
    "ᚉ:Coll:Hazel"
    "ᚊ:Quert:Apple"
    "ᚋ:Muin:Vine"
    "ᚌ:Gort:Ivy"
    "ᚍ:nGetal:Reed"
    "ᚎ:Straif:Blackthorn"
    "ᚏ:Ruis:Elder"
    "ᚐ:Ailm:Pine"
    "ᚑ:Onn:Gorse"
    "ᚒ:Ur:Heather"
    "ᚓ:Edad:Aspen"
    "ᚔ:Idad:Yew"
)

# Seed from date for daily letter
DAY=$(date +%Y%m%d)
IDX=$((DAY % 20))

IFS=':' read -r SYMBOL NAME TREE <<< "${OGHAM[$IDX]}"

# Color based on seasonal association of the tree
# Spring trees = cyan, Summer = gold, Autumn = pink, Winter = purple
if (( IDX < 5 )); then
    COLOR="0xff5cecff"   # Spring
elif (( IDX < 10 )); then
    COLOR="0xfffbb725"   # Summer
elif (( IDX < 15 )); then
    COLOR="0xffff0099"   # Autumn
else
    COLOR="0xffaa00e8"   # Winter
fi

sketchybar --set ogham icon="$SYMBOL" label.drawing=off icon.color="$COLOR"
