#!/bin/bash
# Coding streak - count consecutive days with git commits

REPOS_DIR="$HOME/repo"
TODAY=$(date +%Y-%m-%d)
streak=0
check_date="$TODAY"

# Check each day going backwards
for i in {0..365}; do
    check_date=$(date -v-${i}d +%Y-%m-%d 2>/dev/null)
    found_commit=0

    # Search for commits on this day in any repo
    for repo in "$REPOS_DIR"/*/.git "$REPOS_DIR"/*/*/.git; do
        if [[ -d "$repo" ]]; then
            repo_dir="${repo%/.git}"
            # Check for commits on this date
            if git -C "$repo_dir" log --oneline --since="$check_date 00:00" --until="$check_date 23:59" --author="$(git config user.email)" 2>/dev/null | grep -q .; then
                found_commit=1
                break
            fi
        fi
    done

    if (( found_commit )); then
        streak=$((streak + 1))
    else
        # Allow skipping today if no commits yet
        if (( i > 0 )); then
            break
        fi
    fi
done

# Display
if (( streak >= 7 )); then
    icon=""  # fire
    color="0xffff0099"  # pink - hot streak
elif (( streak >= 3 )); then
    icon=""
    color="0xfffbb725"  # gold - warming up
else
    icon=""
    color="0xff5cecff"  # cyan - starting
fi

sketchybar --set streak icon="$icon" label="hacking ${streak}d" icon.color="$color"
