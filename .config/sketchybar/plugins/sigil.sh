#!/bin/bash
# Morphing sigil - changes based on combined system state

# Gather state inputs
GIT_DIRTY=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
HOUR=$(date +%H)
LOAD=$(sysctl -n vm.loadavg | awk '{print int($2 * 10)}')
PROCS=$(ps aux | wc -l | tr -d ' ')

# Combine into single state value
STATE=$(( (GIT_DIRTY * 7 + HOUR * 13 + LOAD * 3 + PROCS) % 64 ))

# Sigil forms - various esoteric/alchemical symbols
SIGILS=(
    "⍟" "⎔" "⏣" "⏥" "⏦" "◬" "◭" "◮"
    "⟁" "⟐" "⟡" "⟢" "⟣" "⟤" "⟥" "⧫"
    "⧬" "⧭" "⧮" "⧯" "⨁" "⨂" "⨀" "⩕"
    "⩖" "⩗" "⩘" "⪮" "⫯" "⬖" "⬗" "⬘"
    "⬙" "⬚" "⭓" "⭔" "✦" "✧" "❖" "⟟"
    "⟠" "⟡" "⦿" "⧂" "⧃" "⌬" "⌭" "⍝"
    "⎈" "⎊" "⎋" "⏃" "⏄" "⏅" "⏆" "⏇"
    "⏈" "⏉" "⏊" "⏋" "⏌" "⏍" "⏎" "⏏"
)

SIGIL="${SIGILS[$STATE]}"

# Color based on git state
if (( GIT_DIRTY > 10 )); then
    COLOR="0xffff0099"
elif (( GIT_DIRTY > 0 )); then
    COLOR="0xffaa00e8"
else
    COLOR="0xff5cecff"
fi

sketchybar --set sigil icon="$SIGIL" icon.color="$COLOR" label.drawing=off
