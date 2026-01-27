#!/bin/bash
# Opus Magnus - Four stages of the alchemical Great Work
# Based on git repository state (transformation of code)

# Check git state
if git rev-parse --git-dir > /dev/null 2>&1; then
    DIRTY=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
    STAGED=$(git diff --cached --name-only 2>/dev/null | wc -l | tr -d ' ')
    COMMITS_AHEAD=$(git rev-list --count @{u}..HEAD 2>/dev/null || echo "0")
else
    DIRTY=0
    STAGED=0
    COMMITS_AHEAD=0
fi

if (( DIRTY > 0 && STAGED == 0 )); then
    # Nigredo - blackening, putrefaction (dirty uncommitted work)
    SYMBOL="☠"
    COLOR="0xff333333"
elif (( STAGED > 0 )); then
    # Albedo - whitening, purification (staged for commit)
    SYMBOL="☽"
    COLOR="0xffffffff"
elif (( COMMITS_AHEAD > 0 )); then
    # Citrinitas - yellowing, dawn (committed, not pushed)
    SYMBOL="☀"
    COLOR="0xfffbb725"
else
    # Rubedo - reddening, completion (clean, synced)
    SYMBOL="♁"
    COLOR="0xffff0099"
fi

sketchybar --set opus icon="$SYMBOL" icon.color="$COLOR" label.drawing=off
