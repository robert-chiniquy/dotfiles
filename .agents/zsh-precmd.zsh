#!/usr/bin/env zsh
# Keeps /tmp/agent-context populated with current pomodoro phase, git
# branch, and active beads issues, plus pi.dev / codex default model
# state. Pi and codex do not auto-consume this file (unlike goose's
# `tom` extension, now removed); it remains a useful operator-visible
# snapshot and is read by sketchybar items.
#
# Source from .zshrc:  source ~/repo/dotfiles/.agents/zsh-precmd.zsh

agent_context_update() {
    {
        # Active pi.dev defaults.
        local pi_settings="$HOME/.pi/agent/settings.json"
        if [[ -r "$pi_settings" ]] && (( $+commands[jq] )); then
            local pi_line
            pi_line=$(jq -r '
                [.defaultProvider // "?", .defaultModel // "?", .defaultThinkingLevel // ""]
                | "pi: " + .[0] + "/" + .[1] + (if .[2] != "" then " (" + .[2] + ")" else "" end)
            ' "$pi_settings" 2>/dev/null)
            [[ -n "$pi_line" ]] && print -- "$pi_line"
        fi

        # Active codex defaults from ~/.codex/config.toml.
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
    } >| /tmp/agent-context 2>/dev/null
} >/dev/null 2>&1

autoload -Uz add-zsh-hook
add-zsh-hook precmd agent_context_update
