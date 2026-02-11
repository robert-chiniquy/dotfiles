# Check Feature Flag Conflicts

Use this skill when adding new feature flags to a codebase to ensure the ID number is not already taken.

## When to Use

- Before adding a new `FEATURE_FLAG_ID_*` enum value to a proto file
- When reviewing PRs that add feature flags
- When planning work that will require a new feature flag

## Process

1. **Check origin/main for current highest ID:**
   ```bash
   git fetch origin main
   git show origin/main:path/to/features.proto | grep "FEATURE_FLAG_ID_" | grep -oE '[0-9]+' | sort -n | tail -5
   ```

2. **Check open PRs for pending feature flags:**
   ```bash
   gh pr list --state open --json number,headRefName,title | jq -r '.[] | "\(.number) \(.headRefName) \(.title)"'
   ```
   Then for each relevant PR, check its feature flag number.

3. **Check recent branches that might have unreleased feature flags:**
   ```bash
   # List branches with recent commits
   git branch -r --sort=-committerdate | head -20

   # For suspicious branches, check their feature flag proto
   git show origin/branch-name:path/to/features.proto | grep "FEATURE_FLAG_ID_" | tail -5
   ```

4. **Pick a safe number:**
   - Use the next sequential number after the highest found
   - If there are gaps (e.g., 456 then 489), investigate what 489 is for before using numbers in between
   - Document your choice in the commit message

## Example Output

```
Checking feature flag conflicts...
origin/main highest: 455 (SECURITY_INSIGHTS)
Branch foo/feature-bar: 456 (SOME_FEATURE)
Safe to use: 457+
```

## Notes

- Feature flag IDs are permanent - never reuse a number even if the flag is removed
- Numbers should be sequential to make conflicts obvious
- Large gaps (like jumping to 489) usually indicate parallel development - investigate before using intervening numbers
