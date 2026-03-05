# Skills Catalog

Authoritative index of all Claude Code skills. Grouped by category prefix.

---

## git-* : Git Workflows

| Skill | Invokable | Auto | Description |
|-------|-----------|------|-------------|
| git-final-pass | /git-final-pass | no | Pre-PR quality checklist — error handling, naming, tests, hygiene |
| git-create-pr | /git-create-pr | no | Safe dirty-tree-to-PR workflow with safety rules |
| git-reset-workspace | /git-reset-workspace | no | Multi-phase workspace cleanup — branches, worktrees, processes |
| finding-uncommitted-work | /finding-uncommitted-work | no | Scan repos for uncommitted/unpushed/unmerged work |
| pr-status | /pr-status | no | Check open PRs — comments, reviews, CI, activity |

## tone-* : Communication Styles

| Skill | Invokable | Auto | Description |
|-------|-----------|------|-------------|
| dry-witted-engineering | no | **yes** | Default tone for all engineering output |
| casual-slack-tone | no | context | Informal tone for Slack/chat/DMs |
| technical-writing-voice | no | context | Long-form voice for blog posts and deep dives |
| humanizer | /humanizer | no | Remove AI-writing patterns from text |

## design-* : Design Methodology

| Skill | Invokable | Auto | Description |
|-------|-----------|------|-------------|
| systematic-feature-design | no | context | 11-step feature design methodology |
| rigorous-critique | /rigorous-critique | no | Three-lens critique: complexity, fundamentals, feasibility |
| socratic-discovery | no | context | Progressive questions to build consensus |
| complete-developer-experience | no | context | Ensure DX includes tools + docs + agents |
| pqthink | /pqthink | no | Pragmatic CTO-level architecture judgment |

## code-* : Code Quality

| Skill | Invokable | Auto | Description |
|-------|-----------|------|-------------|
| structural-constraints | no | context | Compile-time safety over runtime checks |
| protogen | no | context | Protobuf/gRPC architecture patterns |
| check-feature-flag-conflicts | no | context | Check for feature flag ID collisions |
| terraform | no | context | Terraform modules, tests, CI/CD, security |
| incomplete-work-audit | /incomplete-work-audit | no | Find and resolve TODO/FIXME/HACK markers |

## meta-* : Project Management

| Skill | Invokable | Auto | Description |
|-------|-----------|------|-------------|
| project-process | no | **yes** | Global process framework — artifacts, practices, priorities |
| project | /project | no | Initialize directory with project framework |
| documentation | no | context | Documentation methodology and patterns |

## env-* : Environment & Setup

| Skill | Invokable | Auto | Description |
|-------|-----------|------|-------------|
| passive-qol | no | **yes** | Proactive QoL for dotfiles/shell/system config |

## util-* : Utilities

| Skill | Invokable | Auto | Description |
|-------|-----------|------|-------------|
| bar-chart-comparison | no | context | ASCII bar charts for terminal comparisons |
| jsonl-parsing | no | context | Patterns for JSONL files and event streams |

---

## Legend

- **Invokable**: can be called directly with `/skill-name`
- **Auto = yes**: always active, applied to every response
- **Auto = context**: applied when the context matches (e.g., working with proto files triggers protogen)
- **Auto = no**: only runs when explicitly invoked

## Adding Skills

When creating a new skill:
1. Choose the appropriate category prefix
2. Create directory: `~/.claude/skills/<name>/SKILL.md`
3. Add entry to this catalog
4. If auto-applied, add to CLAUDE.md Skills Application section
