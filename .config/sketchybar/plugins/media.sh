#!/bin/bash
STATE=$(osascript -e 'tell application "Spotify" to player state as string' 2>/dev/null)
if [ "$STATE" = "playing" ]; then
    ARTIST=$(osascript -e 'tell application "Spotify" to artist of current track as string' 2>/dev/null)
    TRACK=$(osascript -e 'tell application "Spotify" to name of current track as string' 2>/dev/null)
    sketchybar --set media label="$ARTIST - $TRACK" drawing=on
else
    # Try Apple Music
    STATE=$(osascript -e 'tell application "Music" to player state as string' 2>/dev/null)
    if [ "$STATE" = "playing" ]; then
        ARTIST=$(osascript -e 'tell application "Music" to artist of current track as string' 2>/dev/null)
        TRACK=$(osascript -e 'tell application "Music" to name of current track as string' 2>/dev/null)
        sketchybar --set media label="$ARTIST - $TRACK" drawing=on
    else
        sketchybar --set media label="" drawing=off
    fi
fi
