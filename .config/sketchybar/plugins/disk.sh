#!/bin/bash
DISK=$(df -H / | awk 'NR==2 {print $5}')
sketchybar --set disk label="disk $DISK"
