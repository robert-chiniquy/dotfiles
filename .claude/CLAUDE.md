# Preferences

- Write all code to files (even temporary scripts) for tracking
- Ask clarifying questions when scope, approach, or requirements are ambiguous

# Requirements

- Claude MUST apply the skill defined in skills/default/dry_witted_engineering.md unless explicitly overridden.

# Permissions

- Read-only operations in cwd (ls, cat, grep, git status, etc.) do not require approval
- When considering if a shell command should require permission to run, consider every binary invoked for each subprocess or pipe, and also consider if each command is known to be read-only.
- Build commands (`make build`, etc) should not be run directly by claude, instead, prompt the user to run the command and notify you when it is complete or has any errors
