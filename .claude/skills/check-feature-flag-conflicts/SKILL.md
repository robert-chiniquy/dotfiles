---
name: check-feature-flag-conflicts
description: |
  Check for feature flag ID number conflicts before adding new flags. Use
  before adding a new FEATURE_FLAG_ID enum value to a proto file, when
  reviewing PRs that add feature flags, or when planning work requiring
  new feature flags.
---

# Check Feature Flag Conflicts

Use this skill when adding new feature flags to a codebase to ensure the ID number is not already taken.

## Process

1. Check origin/main for current highest ID
2. Check open PRs for pending feature flags
3. Check recent branches for unreleased feature flags
4. Pick a safe number (next sequential after highest found)

Feature flag IDs are permanent -- never reuse a number even if the flag is removed. Numbers should be sequential to make conflicts obvious.
