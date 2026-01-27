#!/bin/bash
# Liminal time indicator - marks threshold moments

HOUR=$(date +%H)
MIN=$(date +%M)
NOW=$((HOUR * 60 + MIN))

# Threshold times (approximate)
DAWN_START=360   # 6:00
DAWN_END=420     # 7:00
NOON_START=715   # 11:55
NOON_END=725     # 12:05
DUSK_START=1050  # 17:30
DUSK_END=1140    # 19:00
MIDNIGHT_START=1425  # 23:45
MIDNIGHT_END=15      # 00:15
WITCHING_START=180   # 3:00
WITCHING_END=240     # 4:00

# Check liminal states
if (( NOW >= DAWN_START && NOW <= DAWN_END )); then
    LABEL="dawn"
    COLOR="0xfffbb725"  # gold
elif (( NOW >= NOON_START && NOW <= NOON_END )); then
    LABEL="zenith"
    COLOR="0xffffffff"  # white
elif (( NOW >= DUSK_START && NOW <= DUSK_END )); then
    LABEL="dusk"
    COLOR="0xffaa00e8"  # purple
elif (( NOW >= MIDNIGHT_START || NOW <= MIDNIGHT_END )); then
    LABEL="midnight"
    COLOR="0xffff0099"  # pink
elif (( NOW >= WITCHING_START && NOW <= WITCHING_END )); then
    LABEL="witching"
    COLOR="0xffff00f8"  # magenta
else
    # Not liminal - hide
    sketchybar --set liminal drawing=off
    exit 0
fi

sketchybar --set liminal drawing=on icon="◈" label="$LABEL" icon.color="$COLOR" label.color="$COLOR"
