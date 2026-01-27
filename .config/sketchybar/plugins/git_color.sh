#!/bin/bash
# Git activity color - shifts bar border based on git state

# Check if in a git repo
if ! git rev-parse --git-dir &>/dev/null 2>&1; then
    # Not in git - neutral purple
    sketchybar --bar border_color=0xffaa00e8
    exit 0
fi

# Get git state
DIRTY=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
AHEAD=$(git rev-list --count @{u}..HEAD 2>/dev/null || echo 0)
BEHIND=$(git rev-list --count HEAD..@{u} 2>/dev/null || echo 0)
STASH=$(git stash list 2>/dev/null | wc -l | tr -d ' ')

# Priority: dirty (pink) > behind (gold warning) > ahead (cyan) > clean (purple)
if (( DIRTY > 20 )); then
    # Very dirty - intense pink
    COLOR="0xffff0066"
elif (( DIRTY > 0 )); then
    # Dirty - pink
    COLOR="0xffff0099"
elif (( BEHIND > 0 )); then
    # Behind remote - gold warning
    COLOR="0xfffbb725"
elif (( AHEAD > 0 )); then
    # Ahead - cyan (ready to push)
    COLOR="0xff5cecff"
elif (( STASH > 0 )); then
    # Has stashes - purple
    COLOR="0xffaa00e8"
else
    # Clean - dim purple
    COLOR="0xff550077"
fi

sketchybar --bar border_color="$COLOR"
