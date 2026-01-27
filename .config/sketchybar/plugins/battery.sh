#!/bin/bash
PERCENTAGE=$(pmset -g batt | grep -Eo "\d+%" | cut -d% -f1)
CHARGING=$(pmset -g batt | grep 'AC Power')

if [ "$CHARGING" ]; then
    ICON=""
elif [ "$PERCENTAGE" -gt 80 ]; then
    ICON=""
elif [ "$PERCENTAGE" -gt 60 ]; then
    ICON=""
elif [ "$PERCENTAGE" -gt 40 ]; then
    ICON=""
elif [ "$PERCENTAGE" -gt 20 ]; then
    ICON=""
else
    ICON=""
fi

sketchybar --set battery icon="$ICON" label="bat ${PERCENTAGE}%"
