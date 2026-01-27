#!/bin/bash
# Greek letters - Pythagorean/Mystery tradition symbolism
# Each letter has numeric and mystical significance

LETTERS=(
    "Α" "Β" "Γ" "Δ" "Ε" "Ζ" "Η" "Θ" "Ι" "Κ" "Λ" "Μ"
    "Ν" "Ξ" "Ο" "Π" "Ρ" "Σ" "Τ" "Υ" "Φ" "Χ" "Ψ" "Ω"
)

# Numeric values (isopsephy)
# Special colors for sacred numbers
HOUR=$(date +%H)
IDX=$((HOUR % 24))

LETTER="${LETTERS[$IDX]}"

# Highlight Pythagorean sacred letters
case $IDX in
    0|9)  # Alpha/Iota = 1/10 (monad)
        COLOR="0xffffffff"
        ;;
    6)    # Eta = 7 (virgin number)
        COLOR="0xffaa00e8"
        ;;
    3)    # Delta = 4 (tetractys base)
        COLOR="0xfffbb725"
        ;;
    9)    # Iota = 10 (perfect number)
        COLOR="0xfffbb725"
        ;;
    23)   # Omega = end/completion
        COLOR="0xffff0099"
        ;;
    *)
        COLOR="0xff5cecff"
        ;;
esac

sketchybar --set greek icon="$LETTER" icon.color="$COLOR" label.drawing=off
