#!/bin/bash
VOLUME=$(osascript -e 'output volume of (get volume settings)')
MUTED=$(osascript -e 'output muted of (get volume settings)')

if [ "$MUTED" = "true" ]; then
    ICON=""
elif [ "$VOLUME" -gt 60 ]; then
    ICON=""
elif [ "$VOLUME" -gt 30 ]; then
    ICON=""
elif [ "$VOLUME" -gt 0 ]; then
    ICON=""
else
    ICON=""
fi

sketchybar --set volume icon="$ICON" label="${VOLUME}%"
