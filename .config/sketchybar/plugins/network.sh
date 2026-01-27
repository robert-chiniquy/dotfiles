#!/bin/bash
INTERFACE=$(route get default 2>/dev/null | grep interface | awk '{print $2}')
if [ -n "$INTERFACE" ]; then
    BYTES_IN=$(netstat -ibn | grep -e "$INTERFACE" | head -1 | awk '{print $7}')
    sleep 1
    BYTES_IN_NEW=$(netstat -ibn | grep -e "$INTERFACE" | head -1 | awk '{print $7}')
    SPEED=$(( (BYTES_IN_NEW - BYTES_IN) / 1024 ))
    if [ $SPEED -gt 1024 ]; then
        SPEED=$(echo "scale=1; $SPEED/1024" | bc)
        sketchybar --set network label="${SPEED}M/s"
    else
        sketchybar --set network label="${SPEED}K/s"
    fi
else
    sketchybar --set network label="--"
fi
