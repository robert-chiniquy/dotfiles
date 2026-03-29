# Skills Catalog

17 skills. Always-on skills load at startup. Manual skills load via `/name`.

---

## Always On

| Skill | Description |
|-------|-------------|
| dry-engineering | Default voice: code review style, commit messages, explanations |
| passive-qol | Proactive environment QoL — passive, automatic, invisible |
| project-process | Project framework: artifacts, practices, proverbs (hub + references/) |

## Context-Activated

| Skill | Trigger |
|-------|---------|
| casual-slack-tone | Slack messages, DMs, PR descriptions on own repos |
| technical-writing | Blog posts, articles, long-form external content |
| structural-constraints | Architecture decisions, type system design |
| terraform | .tf files, HCL, infrastructure pipelines (hub + references/) |
| protogen | .proto files, gRPC, codegen (hub + references/) |
| documentation | Writing or reviewing docs (hub + references/) |

## Manual Only (`disable-model-invocation: true`)

| Skill | Invocation | Description |
|-------|------------|-------------|
| git-pr | `/git-pr` | Stage, check, commit, push, create PR |
| git-cleanup | `/git-cleanup` | Workspace cleanup: branches, worktrees, stashes |
| find-work | `/find-work` | Find uncommitted/unpushed/unmerged work + code markers |
| humanizer | `/humanizer` | Strip AI-writing patterns from text |
| project-init | `/project-init [topic]` | Initialize directory with project framework |
| critique | `/critique` | Four-lens design review: complexity, fundamentals, feasibility, scope |
| design | `/design [topic]` | Feature design methodology: research through implementation |
| pqthink | `/pqthink` | Six-pass pragmatic architecture judgment |
