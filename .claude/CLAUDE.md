# Preferences

- Write all code to files (even temporary scripts) for tracking
- Ask clarifying questions when scope, approach, or requirements are ambiguous
- Never add project-specific or repo-specific notes to this global config; those belong in each project's `.claude/CLAUDE.md`

# Requirements

- Claude MUST apply the skill defined in skills/default/dry_witted_engineering.md unless explicitly overridden.
- For architectural decisions, apply skills/engineering/structural_constraints.md (prefer compile-time safety over runtime checks).
- For design and planning work, apply skills in skills/design/:
  - systematic_feature_design.md: Use 10-step methodology and Level Framework for new features
  - socratic_discovery.md: Use questions to build consensus when stakeholders are skeptical
  - rigorous_critique.md: Apply three-lens critique before implementation (expect 30-50% cuts)
  - complete_developer_experience.md: Ensure Tools + Documentation + Agents (all three legs)

# Permissions

- Read-only operations in cwd (ls, cat, grep, git status, etc.) do not require approval
- When considering if a shell command should require permission to run, consider every binary invoked for each subprocess or pipe, and also consider if each command is known to be read-only.
- Build commands (`make build`, etc) should not be run directly by claude, instead, prompt the user to run the command and notify you when it is complete or has any errors
- NEVER delete files without explicit user permission. Deletion is lossy and irreversible. Always ask before removing files, even if they appear redundant or superseded.
