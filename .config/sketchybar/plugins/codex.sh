#!/bin/bash
# Codex state — current model + reasoning_effort from ~/.codex/config.toml.
# Color of the label encodes effort: deeper effort -> cooler color.

CONFIG="$HOME/.codex/config.toml"
if [[ ! -r "$CONFIG" ]]; then
    sketchybar --set codex label="off" icon="?" icon.color=0xffaa00e8
    exit 0
fi

FIELDS=$(awk -F' *= *' '
    /^\[/ {section=$0; next}
    section=="" && $1=="model"                  {gsub(/"/,"",$2); m=$2}
    section=="" && $1=="model_reasoning_effort" {gsub(/"/,"",$2); e=$2}
    END {print m"\t"e}
' "$CONFIG")

MODEL=$(echo "$FIELDS" | cut -f1)
EFFORT=$(echo "$FIELDS" | cut -f2)

case "$EFFORT" in
    xhigh)        COLOR="0xff5cecff" ;;   # cyan
    high)         COLOR="0xffff0099" ;;   # pink
    medium)       COLOR="0xffff00f8" ;;   # magenta
    low)          COLOR="0xfffbb725" ;;   # gold
    minimal|off)  COLOR="0xff8a8a9c" ;;   # gray
    *)            COLOR="0xffaa00e8" ;;   # purple (unknown)
esac

if [[ -n "$EFFORT" ]]; then
    LABEL="$MODEL · $EFFORT"
else
    LABEL="$MODEL"
fi

sketchybar --set codex \
    icon="▼" \
    icon.color="$COLOR" \
    label="$LABEL" \
    label.color="$COLOR"
