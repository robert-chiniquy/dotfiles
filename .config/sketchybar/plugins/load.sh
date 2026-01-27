#!/bin/bash
LOAD=$(sysctl -n vm.loadavg | awk '{print $2}')
sketchybar --set load label="$LOAD"
