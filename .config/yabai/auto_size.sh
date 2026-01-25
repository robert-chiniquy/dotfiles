#!/usr/bin/env bash

# Get the window that triggered the event
WINDOW_ID="${YABAI_WINDOW_ID:-$1}"

if [ -z "$WINDOW_ID" ]; then
    exit 0
fi

# Get window info
WINDOW_INFO=$(yabai -m query --windows --window "$WINDOW_ID" 2>/dev/null)
if [ -z "$WINDOW_INFO" ]; then
    exit 0
fi

APP_NAME=$(echo "$WINDOW_INFO" | jq -r '.app')
SPACE_ID=$(echo "$WINDOW_INFO" | jq -r '.space')

# Count windows from this app on this space
APP_WINDOW_COUNT=$(yabai -m query --windows --space "$SPACE_ID" | jq "[.[] | select(.app == \"$APP_NAME\")] | length")

# Get display dimensions
DISPLAY_INFO=$(yabai -m query --displays --display)
DISPLAY_W=$(echo "$DISPLAY_INFO" | jq -r '.frame.w')
DISPLAY_H=$(echo "$DISPLAY_INFO" | jq -r '.frame.h')
DISPLAY_X=$(echo "$DISPLAY_INFO" | jq -r '.frame.x')
DISPLAY_Y=$(echo "$DISPLAY_INFO" | jq -r '.frame.y')

if [ "$APP_WINDOW_COUNT" -eq 1 ]; then
    # Solo app window: float at 80% centered
    NEW_W=$(echo "$DISPLAY_W * 0.8" | bc | cut -d. -f1)
    NEW_H=$(echo "$DISPLAY_H * 0.8" | bc | cut -d. -f1)
    NEW_X=$(echo "$DISPLAY_X + ($DISPLAY_W - $NEW_W) / 2" | bc | cut -d. -f1)
    NEW_Y=$(echo "$DISPLAY_Y + ($DISPLAY_H - $NEW_H) / 2" | bc | cut -d. -f1)

    yabai -m window "$WINDOW_ID" --toggle float 2>/dev/null
    yabai -m window "$WINDOW_ID" --move abs:"$NEW_X":"$NEW_Y" 2>/dev/null
    yabai -m window "$WINDOW_ID" --resize abs:"$NEW_W":"$NEW_H" 2>/dev/null
else
    # Multiple windows from same app: ensure tiled
    IS_FLOATING=$(echo "$WINDOW_INFO" | jq -r '."is-floating"')
    if [ "$IS_FLOATING" = "true" ]; then
        yabai -m window "$WINDOW_ID" --toggle float 2>/dev/null
    fi

    # Re-tile all windows from this app
    yabai -m query --windows --space "$SPACE_ID" | jq -r ".[] | select(.app == \"$APP_NAME\") | .id" | while read WID; do
        IS_FLOAT=$(yabai -m query --windows --window "$WID" | jq -r '."is-floating"')
        if [ "$IS_FLOAT" = "true" ]; then
            yabai -m window "$WID" --toggle float 2>/dev/null
        fi
    done
fi
