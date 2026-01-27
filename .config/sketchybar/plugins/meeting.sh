#!/bin/bash
# Meeting countdown - shows next calendar event

# Try icalBuddy first (brew install ical-buddy)
if command -v icalBuddy &>/dev/null; then
    NEXT=$(icalBuddy -n -nc -ea -li 1 -ps "| - |" -po "datetime,title" -tf "%H:%M" -df "" eventsToday+1 2>/dev/null | head -1)
else
    # Fallback: AppleScript for Calendar.app
    NEXT=$(osascript -e '
        set now to current date
        set endOfSearch to now + (24 * 60 * 60)
        tell application "Calendar"
            set nextEvent to ""
            set nextTime to endOfSearch
            repeat with c in calendars
                try
                    set evts to (every event of c whose start date >= now and start date < endOfSearch)
                    repeat with e in evts
                        if start date of e < nextTime then
                            set nextTime to start date of e
                            set h to hours of nextTime
                            set m to minutes of nextTime
                            set nextEvent to (text 1 thru 2 of ("0" & h)) & ":" & (text 1 thru 2 of ("0" & m)) & " - " & summary of e
                        end if
                    end repeat
                end try
            end repeat
            return nextEvent
        end tell
    ' 2>/dev/null)
fi

if [[ -z "$NEXT" || "$NEXT" == "missing value" ]]; then
    sketchybar --set meeting drawing=off
    exit 0
fi

# Parse time and title
TIME=$(echo "$NEXT" | cut -d'-' -f1 | tr -d ' ')
TITLE=$(echo "$NEXT" | cut -d'-' -f2- | xargs | cut -c1-20)

# Calculate minutes until meeting
NOW_MINS=$(($(date +%H) * 60 + $(date +%M)))
MTG_HOUR=$(echo "$TIME" | cut -d: -f1 | sed 's/^0//')
MTG_MIN=$(echo "$TIME" | cut -d: -f2 | sed 's/^0//')
MTG_MINS=$((MTG_HOUR * 60 + MTG_MIN))
DIFF=$((MTG_MINS - NOW_MINS))

# Color based on urgency
if (( DIFF <= 5 )); then
    color="0xffff0099"  # pink - imminent
    icon=""
elif (( DIFF <= 15 )); then
    color="0xfffbb725"  # gold - soon
    icon=""
elif (( DIFF <= 60 )); then
    color="0xff5cecff"  # cyan - upcoming
    icon=""
else
    color="0xff444444"  # grey - later
    icon=""
fi

if (( DIFF < 0 )); then
    # Meeting in progress or past
    sketchybar --set meeting drawing=off
else
    sketchybar --set meeting \
        drawing=on \
        icon="$icon" \
        icon.color="$color" \
        label="${DIFF}m $TITLE"
fi
