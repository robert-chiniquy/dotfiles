#!/bin/bash
# Claude Code statusline command
# Outputs single-line status info

# Function to find agent name from assignment files
get_agent_name() {
    local dir="$PWD"

    # Walk up looking for INBOX with assignment files
    while [[ "$dir" != "/" ]]; do
        if [[ -d "$dir/INBOX" ]]; then
            # Look for assignment_NAME_*.md files (in INBOX or INBOX/processed)
            local assignment=$(ls "$dir/INBOX/assignment_"*".md" "$dir/INBOX/processed/assignment_"*".md" 2>/dev/null | head -1)
            if [[ -n "$assignment" ]]; then
                # Extract name from filename: assignment_NAME_date.md
                local name=$(basename "$assignment" | sed 's/assignment_\([^_]*\)_.*/\1/')
                if [[ -n "$name" && "$name" != "AGENT" ]]; then
                    echo "$name"
                    return
                fi
            fi
        fi
        dir=$(dirname "$dir")
    done
}

# Get agent name if assigned
agent=$(get_agent_name)

# Get git branch if in repo
if git rev-parse --git-dir > /dev/null 2>&1; then
    branch=$(git branch --show-current 2>/dev/null || echo "detached")
    dirty=$(git diff --quiet 2>/dev/null || echo "*")
    if [[ -n "$agent" ]]; then
        echo "[$agent] ${branch}${dirty}"
    else
        echo "${branch}${dirty}"
    fi
else
    if [[ -n "$agent" ]]; then
        echo "[$agent] $(basename "$PWD")"
    else
        basename "$PWD"
    fi
fi
