#!/bin/bash
# Goose state — current GOOSE_PROVIDER / GOOSE_MODEL / reasoning effort.
# Reads ~/.config/goose/config.yaml directly so it reflects manual edits
# and post-`goose configure` writes on the next refresh.

CONFIG="$HOME/.config/goose/config.yaml"
if [[ ! -r "$CONFIG" ]]; then
    sketchybar --set goose label="off" icon="?" icon.color=0xffaa00e8
    exit 0
fi

PROVIDER=$(awk '$1=="GOOSE_PROVIDER:"{print $2}' "$CONFIG")
MODEL=$(awk '$1=="GOOSE_MODEL:"{print $2}' "$CONFIG")

case "$PROVIDER" in
    chatgpt_codex)
        ICON="◆"
        COLOR="0xff5cecff"   # cyan
        EFFORT=$(awk '$1=="CHATGPT_CODEX_REASONING_EFFORT:"{print $2}' "$CONFIG")
        ;;
    anthropic)
        ICON="✦"
        COLOR="0xfffbb725"   # gold
        TT=$(awk '$1=="CLAUDE_THINKING_TYPE:"{print $2}' "$CONFIG")
        [[ "$TT" == "disabled" ]] && EFFORT="" || EFFORT="$TT"
        ;;
    *)
        ICON="?"
        COLOR="0xffaa00e8"   # purple
        EFFORT=""
        ;;
esac

if [[ -n "$EFFORT" ]]; then
    LABEL="$MODEL · $EFFORT"
else
    LABEL="$MODEL"
fi

sketchybar --set goose \
    icon="$ICON" \
    icon.color="$COLOR" \
    label="$LABEL" \
    label.color="$COLOR"
