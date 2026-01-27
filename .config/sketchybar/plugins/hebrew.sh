#!/bin/bash
# Hebrew letters - 22 letters of creation (Sefer Yetzirah)
# Each letter has mystical meaning in Kabbalah

LETTERS=(
    "א" "ב" "ג" "ד" "ה" "ו" "ז" "ח" "ט" "י" "כ"
    "ל" "מ" "נ" "ס" "ע" "פ" "צ" "ק" "ר" "ש" "ת"
)

# Three mother letters, seven doubles, twelve simples
# Colors reflect this classification
COLORS=(
    "0xffffffff"  # Aleph - mother (air)
    "0xfffbb725"  # Bet - double
    "0xfffbb725"  # Gimel - double
    "0xfffbb725"  # Dalet - double
    "0xff5cecff"  # He - simple
    "0xff5cecff"  # Vav - simple
    "0xff5cecff"  # Zayin - simple
    "0xff5cecff"  # Chet - simple
    "0xff5cecff"  # Tet - simple
    "0xff5cecff"  # Yod - simple
    "0xfffbb725"  # Kaf - double
    "0xff5cecff"  # Lamed - simple
    "0xffff0099"  # Mem - mother (water)
    "0xff5cecff"  # Nun - simple
    "0xff5cecff"  # Samekh - simple
    "0xff5cecff"  # Ayin - simple
    "0xfffbb725"  # Pe - double
    "0xff5cecff"  # Tsadi - simple
    "0xff5cecff"  # Qof - simple
    "0xfffbb725"  # Resh - double
    "0xffaa00e8"  # Shin - mother (fire)
    "0xfffbb725"  # Tav - double
)

# Cycle by hour
HOUR=$(date +%H)
IDX=$((HOUR % 22))

LETTER="${LETTERS[$IDX]}"
COLOR="${COLORS[$IDX]}"

sketchybar --set hebrew icon="$LETTER" icon.color="$COLOR" label.drawing=off
