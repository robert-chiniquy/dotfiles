#!/bin/bash
PROCS=$(ps aux | wc -l | tr -d ' ')
sketchybar --set procs label="$PROCS"
