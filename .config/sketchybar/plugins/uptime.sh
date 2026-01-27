#!/bin/bash
UPTIME=$(uptime | sed 's/.*up //' | sed 's/,.*//' | sed 's/  / /g' | xargs)
sketchybar --set uptime label="up $UPTIME"
