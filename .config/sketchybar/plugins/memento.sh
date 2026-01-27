#!/bin/bash
# Memento Mori - mortality symbols
# Intensity based on uptime (longer = more dire)

# Get uptime in hours
UPTIME_SEC=$(sysctl -n kern.boottime | awk '{print $4}' | tr -d ',')
NOW=$(date +%s)
UPTIME_HOURS=$(( (NOW - UPTIME_SEC) / 3600 ))

if (( UPTIME_HOURS < 2 )); then
    # Fresh - hourglass half
    SYMBOL="⏳"
    COLOR="0xff5cecff"
elif (( UPTIME_HOURS < 6 )); then
    # Working - hourglass
    SYMBOL="⌛"
    COLOR="0xfffbb725"
elif (( UPTIME_HOURS < 12 )); then
    # Long session - skull
    SYMBOL="☠"
    COLOR="0xffaa00e8"
elif (( UPTIME_HOURS < 24 )); then
    # Very long - coffin
    SYMBOL="⚰"
    COLOR="0xffff0099"
else
    # Excessive - death
    SYMBOL="💀"
    COLOR="0xffff0099"
fi

sketchybar --set memento icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
