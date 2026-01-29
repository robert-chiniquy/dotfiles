#!/bin/bash
# Wheel of the Year - next pagan sabbat countdown
# The 8 sabbats of the Wiccan calendar

# Get current day of year and year
DOY=$(date +%j | sed 's/^0*//')
YEAR=$(date +%Y)

# Sabbat dates (approximate day of year)
# Samhain: Oct 31 = 304 (305 leap)
# Yule: Dec 21 = 355 (356 leap)
# Imbolc: Feb 1 = 32
# Ostara: Mar 20 = 79 (80 leap)
# Beltane: May 1 = 121 (122 leap)
# Litha: Jun 21 = 172 (173 leap)
# Lughnasadh: Aug 1 = 213 (214 leap)
# Mabon: Sep 22 = 265 (266 leap)

# Check leap year
if (( YEAR % 4 == 0 && (YEAR % 100 != 0 || YEAR % 400 == 0) )); then
    LEAP=1
else
    LEAP=0
fi

# Sabbat data: day_of_year|name|symbol|theme
SABBATS=(
    "32|Imbolc|🜚|First Light"
    "$((79 + LEAP))|Ostara|🌱|Spring Equinox"
    "$((121 + LEAP))|Beltane|🜂|Fire Festival"
    "$((172 + LEAP))|Litha|☉|Summer Solstice"
    "$((213 + LEAP))|Lughnasadh|🌾|First Harvest"
    "$((265 + LEAP))|Mabon|🍂|Autumn Equinox"
    "$((304 + LEAP))|Samhain|💀|Veil Thins"
    "$((355 + LEAP))|Yule|❄|Rebirth of Sun"
)

# Find next sabbat
DAYS_IN_YEAR=$((365 + LEAP))
NEXT_NAME=""
NEXT_SYMBOL=""
NEXT_THEME=""
NEXT_DAYS=999

for sabbat in "${SABBATS[@]}"; do
    IFS='|' read -r sday name symbol theme <<< "$sabbat"

    if (( sday > DOY )); then
        days_until=$((sday - DOY))
    else
        days_until=$((DAYS_IN_YEAR - DOY + sday))
    fi

    if (( days_until < NEXT_DAYS )); then
        NEXT_DAYS=$days_until
        NEXT_NAME="$name"
        NEXT_SYMBOL="$symbol"
        NEXT_THEME="$theme"
    fi
done

# Color based on proximity
if (( NEXT_DAYS <= 7 )); then
    COLOR="0xffff0099"  # Pink - imminent
elif (( NEXT_DAYS <= 30 )); then
    COLOR="0xfffbb725"  # Gold - approaching
else
    COLOR="0xff5cecff"  # Cyan - distant
fi

# Format output
if (( NEXT_DAYS == 0 )); then
    LABEL="$NEXT_NAME - $NEXT_THEME - TODAY"
elif (( NEXT_DAYS == 1 )); then
    LABEL="$NEXT_NAME - $NEXT_THEME - tomorrow"
else
    LABEL="$NEXT_NAME - $NEXT_THEME - ${NEXT_DAYS}d"
fi

sketchybar --set sabbat icon="$NEXT_SYMBOL" label="$LABEL" icon.color="$COLOR" label.color="$COLOR"
