#!/bin/bash
CPU=$(top -l 1 | grep "CPU usage" | awk '{print int($3)}')
sketchybar --set cpu label="${CPU}%"
