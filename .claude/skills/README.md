# Claude Skills

Skill files for Claude Code. Load selectively based on task.

## When to Load

| User says | Load |
|-----------|------|
| "project" | `meta/project-index.md` (references all project-* files) |
| "proto", "protobuf", "grpc" | `design/proto-overview.md` + relevant proto-* |
| "design", "feature" | `design/systematic_feature_design.md` |
| "critique", "review design" | `design/rigorous_critique.md` |
| "stakeholder", "consensus" | `design/socratic_discovery.md` |
| "dx", "developer experience" | `design/complete_developer_experience.md` |
| "docs", "documentation" | `documentation/doc-overview.md` + relevant doc-* |
| "write docs" | `documentation/doc-process.md`, `doc-content.md` |
| "rap", "agent docs" | `documentation/rap_documentation.md` |
| "merge docs" | `documentation/documentation_merging.md` |
| "marketing", "announce" | `documentation/marketing_lens.md` |
| "tone matrix" | `documentation/tone_matrixing.md` |
| "architecture", "constraints" | `engineering/structural_constraints.md` |
| "audit", "incomplete" | `codebase/incomplete_work_audit.md` |
| "jsonl" | `utility/jsonl_parsing.md` |
| "bar chart" | `default/bar_chart_comparison.md` |
| "humanize", "ai detection" | `humanizer/SKILL.md` |
| "terraform", "opentofu", "iac" | `terraform/SKILL.md` |

## Structure

```
skills/
├── default/
│   ├── dry_witted_engineering.md    # Default tone (always loaded)
│   └── bar_chart_comparison.md
├── design/
│   ├── systematic_feature_design.md
│   ├── socratic_discovery.md
│   ├── rigorous_critique.md
│   ├── complete_developer_experience.md
│   ├── proto-overview.md            # Entry point
│   ├── proto-schema.md
│   ├── proto-architecture.md
│   ├── proto-database.md
│   ├── proto-patterns.md
│   ├── proto-project.md
│   ├── proto-testing.md
│   ├── proto-frontend.md
│   └── proto-pitfalls.md
├── documentation/
│   ├── doc-overview.md              # Entry point
│   ├── doc-process.md
│   ├── doc-content.md
│   ├── doc-templates.md
│   ├── doc-verify.md
│   ├── doc-learnings.md
│   ├── doc-organization.md
│   ├── rap_documentation.md
│   ├── layered_documentation.md
│   ├── documentation_merging.md
│   ├── tone_matrixing.md
│   └── marketing_lens.md
├── engineering/
│   ├── structural_constraints.md
│   └── check_feature_flag_conflicts.md
├── meta/
│   ├── project-index.md             # Entry point (references all project-* files)
│   ├── project-artifacts.md
│   ├── project-practices.md
│   ├── project-organization.md
│   ├── project-priorities.md
│   ├── project-multisubproject.md
│   └── PROVERBS.md                  # Guiding principles (MUST read)
├── codebase/
│   └── incomplete_work_audit.md
├── utility/
│   └── jsonl_parsing.md
├── humanizer/
│   ├── SKILL.md
│   ├── WARP.md
│   └── README.md
├── terraform/
│   ├── SKILL.md
│   └── references/
│       ├── ci-cd-workflows.md
│       ├── code-patterns.md
│       ├── module-patterns.md
│       ├── quick-reference.md
│       ├── security-compliance.md
│       └── testing-frameworks.md
└── old/                             # Archived originals
```

## Skill Groups

### Protogen (load as needed)

Start with `proto-overview.md`, add others based on task:

| File | Content |
|------|---------|
| `proto-overview.md` | What, when, philosophy |
| `proto-schema.md` | Proto organization, codegen |
| `proto-architecture.md` | Three layers, WithPassport |
| `proto-database.md` | DynamoDB, Postgres, SQLite |
| `proto-patterns.md` | GetOrCreate, Mutate, Wire |
| `proto-project.md` | Directory structure, Makefile |
| `proto-testing.md` | Unit and integration tests |
| `proto-frontend.md` | Transport, WebSocket, Proto-UI |
| `proto-pitfalls.md` | Common mistakes |

### Documentation (load as needed)

Start with `doc-overview.md`, add others based on task:

| File | Content |
|------|---------|
| `doc-overview.md` | Philosophy, phases, levels |
| `doc-process.md` | 8-step process |
| `doc-content.md` | 7-step pattern, writing guidelines |
| `doc-templates.md` | Section templates |
| `doc-verify.md` | Verification, audits |
| `doc-learnings.md` | Hard-won lessons |
| `doc-organization.md` | File structure |

### Project (load as needed)

Start with `project-index.md`, which references all project files:

| File | Content |
|------|---------|
| `project-index.md` | Index referencing all project-* files |
| `project-artifacts.md` | DATA_SOURCES, LEARNINGS, etc. |
| `project-practices.md` | Versioning, commits, catalogs |
| `project-organization.md` | Directory structure, Makefiles |
| `project-priorities.md` | Design priority, momentum |
| `project-multisubproject.md` | Build discipline, isolation |
| `PROVERBS.md` | Guiding principles (always loaded) |

### Terraform / OpenTofu (load as needed)

Start with `terraform/SKILL.md`, references loaded on demand:

| File | Content |
|------|---------|
| `SKILL.md` | Core principles, testing strategy, module dev |
| `references/testing-frameworks.md` | Static analysis, native tests, Terratest |
| `references/module-patterns.md` | Variable/output best practices |
| `references/code-patterns.md` | Block ordering, count vs for_each |
| `references/ci-cd-workflows.md` | GitHub Actions, GitLab CI templates |
| `references/security-compliance.md` | Trivy/Checkov, secrets management |
| `references/quick-reference.md` | Command cheat sheets, troubleshooting |

## Always Loaded

- `default/dry_witted_engineering.md` - Default communication tone
- `meta/project-index.md` - Project process (references all project-* files)
- `meta/PROVERBS.md` - Guiding principles

## Usage

Reference in CLAUDE.md:

```markdown
# Requirements
- Claude MUST apply skills/default/dry_witted_engineering.md
- For projects, Claude MUST apply skills/meta/project-artifacts.md
```
