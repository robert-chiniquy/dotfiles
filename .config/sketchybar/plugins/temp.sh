#!/bin/bash
# Get CPU temp if possible
TEMP=$(sudo powermetrics --samplers smc -i1 -n1 2>/dev/null | grep "CPU die temperature" | awk '{print int($4)}')
if [ -z "$TEMP" ]; then
    sketchybar --set temp label="--"
else
    sketchybar --set temp label="${TEMP}C"
fi
