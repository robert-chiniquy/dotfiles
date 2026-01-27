#!/bin/bash
# Veve - Vodou sacred symbols for the Loa (spirits)
# Using Unicode approximations of the geometric patterns

# Selected veve-like symbols representing different Loa
VEVES=(
    "⚚"   # Legba - crossroads
    "⚕"   # Damballa - serpent/healing
    "♆"   # Agwe - sea
    "⚶"   # Ogoun - iron/war
    "❦"   # Erzulie - love
    "☤"   # Simbi - magic
    "⚸"   # Baron Samedi - death
    "✠"   # Kalfu - dark crossroads
)

# Colors associated with each Loa
COLORS=(
    "0xffffffff"  # Legba - white/red
    "0xffffffff"  # Damballa - white
    "0xff5cecff"  # Agwe - blue/cyan
    "0xffff0099"  # Ogoun - red/pink
    "0xffff00f8"  # Erzulie - pink/magenta
    "0xffaa00e8"  # Simbi - green (purple)
    "0xff333333"  # Baron - black/purple
    "0xffff0099"  # Kalfu - red/black
)

# Seed from date
DAY=$(date +%Y%m%d)
IDX=$((DAY % 8))

SYMBOL="${VEVES[$IDX]}"
COLOR="${COLORS[$IDX]}"

sketchybar --set veve icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
