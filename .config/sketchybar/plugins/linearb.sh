#!/bin/bash
# Linear B - Mycenaean Greek syllabic script (c. 1450-1200 BCE)
# Deciphered by Michael Ventris in 1952
# Used for palace administration records at Knossos, Pylos, etc.

# Linear B syllables (Unicode U+10000-U+1007F)
# Selected recognizable syllables
SYLLABLES=(
    "𐀀"   # a
    "𐀁"   # e
    "𐀂"   # i
    "𐀃"   # o
    "𐀄"   # u
    "𐀅"   # da
    "𐀆"   # de
    "𐀇"   # di
    "𐀈"   # do
    "𐀉"   # du
    "𐀊"   # ja
    "𐀋"   # je
    "𐀍"   # jo
    "𐀏"   # ka
    "𐀐"   # ke
    "𐀑"   # ki
    "𐀒"   # ko
    "𐀓"   # ku
    "𐀔"   # ma
    "𐀕"   # me
    "𐀖"   # mi
    "𐀗"   # mo
    "𐀘"   # mu
    "𐀙"   # na
    "𐀚"   # ne
    "𐀛"   # ni
    "𐀜"   # no
    "𐀝"   # nu
    "𐀞"   # pa
    "𐀟"   # pe
    "𐀠"   # pi
    "𐀡"   # po
    "𐀢"   # pu
    "𐀣"   # qa
    "𐀤"   # qe
    "𐀥"   # qi
    "𐀦"   # qo
    "𐀨"   # ra
    "𐀩"   # re
    "𐀪"   # ri
    "𐀫"   # ro
    "𐀬"   # ru
)

# Seed from hour for slow cycling
HOUR=$(date +%H)
MIN=$(date +%M)
IDX=$(( (HOUR * 60 + MIN) % 42 ))

SYMBOL="${SYLLABLES[$IDX]}"

# Color based on position - bronze age palette
if (( IDX < 14 )); then
    COLOR="0xfffbb725"   # Gold/bronze
elif (( IDX < 28 )); then
    COLOR="0xff5cecff"   # Cyan (Aegean sea)
else
    COLOR="0xffff0099"   # Pink (Minoan frescoes)
fi

sketchybar --set linearb icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
