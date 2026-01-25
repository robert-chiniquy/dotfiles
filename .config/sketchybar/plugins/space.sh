#!/bin/bash
if [ "$SELECTED" = "true" ]; then
    sketchybar --set $NAME background.color=0xffaa00e8 icon.color=0xffffffff
else
    sketchybar --set $NAME background.color=0x00000000 icon.color=0xff5cecff
fi
