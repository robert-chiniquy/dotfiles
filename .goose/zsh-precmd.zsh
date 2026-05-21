#!/usr/bin/env zsh
# Keeps /tmp/goose-context populated with current pomodoro phase, git
# branch, and active beads issues. Goose's `tom` (Top Of Mind) extension
# reads this file every turn via $GOOSE_MOIM_MESSAGE_FILE so each prompt
# starts with up-to-date situational context.
#
# Source from .zshrc:  source ~/repo/dotfiles/.goose/zsh-precmd.zsh

goose_context_update() {
    {
        # Active goose flavor (single awk pass over config.yaml top-level keys).
        local config="$HOME/.config/goose/config.yaml"
        if [[ -r "$config" ]]; then
            local fields
            fields=$(awk '
                $1=="GOOSE_PROVIDER:"                  {p=$2}
                $1=="GOOSE_MODEL:"                     {m=$2}
                $1=="CHATGPT_CODEX_REASONING_EFFORT:"  {c=$2}
                $1=="CLAUDE_THINKING_TYPE:"            {t=$2}
                END {print p"\t"m"\t"c"\t"t}
            ' "$config")
            local provider model codex_effort think_type effort
            IFS=$'\t' read -r provider model codex_effort think_type <<< "$fields"
            case "$provider" in
                chatgpt_codex) effort="$codex_effort" ;;
                anthropic)     [[ "$think_type" != "disabled" ]] && effort="$think_type" ;;
            esac
            if [[ -n "$effort" ]]; then
                print -- "goose: $provider/$model ($effort)"
            elif [[ -n "$model" ]]; then
                print -- "goose: $provider/$model"
            fi
        fi

        # Active pi.dev defaults (jq parse of ~/.pi/agent/settings.json).
        local pi_settings="$HOME/.pi/agent/settings.json"
        if [[ -r "$pi_settings" ]] && (( $+commands[jq] )); then
            local pi_line
            pi_line=$(jq -r '
                [.defaultProvider // "?", .defaultModel // "?", .defaultThinkingLevel // ""]
                | "pi: " + .[0] + "/" + .[1] + (if .[2] != "" then " (" + .[2] + ")" else "" end)
            ' "$pi_settings" 2>/dev/null)
            [[ -n "$pi_line" ]] && print -- "$pi_line"
        fi

        # Active codex defaults (top-level keys in ~/.codex/config.toml).
        # Variable names prefixed cdx_ to avoid colliding with codex_effort
        # already in scope from the goose block above.
        local cdx_config="$HOME/.codex/config.toml"
        if [[ -r "$cdx_config" ]]; then
            local cdx_fields cdx_model cdx_effort
            cdx_fields=$(awk -F' *= *' '
                /^\[/ {section=$0; next}
                section=="" && $1=="model"                  {gsub(/"/,"",$2); m=$2}
                section=="" && $1=="model_reasoning_effort" {gsub(/"/,"",$2); e=$2}
                END {print m"\t"e}
            ' "$cdx_config")
            IFS=$'\t' read -r cdx_model cdx_effort <<< "$cdx_fields"
            if [[ -n "$cdx_effort" ]]; then
                print -- "codex: $cdx_model ($cdx_effort)"
            elif [[ -n "$cdx_model" ]]; then
                print -- "codex: $cdx_model"
            fi
        fi

        if [[ -r /tmp/pomodoro-state ]]; then
            print -- "pomodoro: $(< /tmp/pomodoro-state)"
        fi

        local branch
        branch=$(git -C "$PWD" branch --show-current 2>/dev/null) || true
        [[ -n "$branch" ]] && print -- "branch: $branch ($(basename "$PWD"))"

        if (( $+commands[bd] )); then
            local active
            active=$(bd list --status=in_progress 2>/dev/null | head -3)
            if [[ -n "$active" ]]; then
                print -- "active bd:"
                print -- "$active"
            fi
        fi
    } >| /tmp/goose-context 2>/dev/null
} >/dev/null 2>&1
# ^ outer redirect is belt-and-suspenders: catches any output that escapes
# the inner redirect (zsh trace messages, awk warnings on a malformed config,
# `local` errors in edge cases). Inner already directs the real payload to
# /tmp/goose-context; outer ensures the user's terminal stays silent.

autoload -Uz add-zsh-hook
add-zsh-hook precmd goose_context_update
