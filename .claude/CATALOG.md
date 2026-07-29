# Skills Catalog

Shared skill index for **Claude Code, Grok Build, Codex, and other harnesses**.
Canonical skill tree: `~/.claude/skills/`. `~/.agents/skills/` is symlinked to it.
Grok also loads `~/.grok/skills/` (Grok-only harness skills) and project skills.

Count the tree with `ls ~/.claude/skills | wc -l`. Always-on skills are listed in
`~/.claude/Claude.md` and (for Grok) `~/.grok/AGENTS.md` — agents must **read the
skill bodies**, not only this index.

---

## Always On

| Skill | Description |
|-------|-------------|
| dry-engineering | Default voice: code review style, commit messages, explanations |
| healthy-interaction | Baseline interaction dispositions (no sycophancy, no therapy mode) |
| open-work-recap | Coding stopping-point recap: open PRs/tickets/issues (URLs) + Next: |
| passive-qol | Proactive environment QoL — passive, automatic, invisible |
| project-process | Project framework: artifacts, practices, proverbs (hub + references/) |

## Context-Activated

| Skill | Trigger |
|-------|---------|
| casual-slack-tone | Slack messages, DMs, PR descriptions on own repos |
| technical-writing | Blog posts, articles, long-form external content |
| technical-writing-voice | Long-form external voice (articles, talks) |
| structural-constraints | Architecture decisions, type system design |
| terraform / terraform-skill | .tf files, HCL, infrastructure pipelines |
| protogen | .proto files, gRPC, codegen |
| documentation | Writing or reviewing docs |
| subagent-prompt-review | Before Agent() / sqfan dispatch / scheduled remote agents |
| agent-worktree-status | Background agent worktree liveness before kill/restart |
| gestalt-consistency-review | Correct-but-odd-one-out APIs; convention drift |
| calendaring | Multi-month personal master schedule |
| tactical-sitrep | Named milestone + hard deadline → A/B/C readiness |
| questioning-the-user | Multiple pending decisions → one at a time |
| complete-developer-experience | Tools + docs + agents for developer-facing features |
| systematic-feature-design | 11-step feature design methodology |
| socratic-discovery | Progressive questions for consensus / assumptions |
| rigorous-critique | Complexity / fundamentals / feasibility critique |
| post-change-verification | After Go code changes: fmt/lint/build/test protocol |
| golang-code-review | Go PR / architecture / test quality review |
| pr-pass / pr-status | Open PR triage and status |
| github-pr-threads | Reply/resolve PR review threads after fixes |
| pr-deep-review | Multi-agent deep PR review |
| squire-env-management | Ephemeral remote agents and task pools |
| c1-squire-dispatch / c1-dev-stack-in-squire | c1-specific squire dispatch |
| find-delegation-pebbles | Find bounded, independent backlog tasks for remote agents |
| codebase-memory | Structural codebase graph exploration |
| large-scale-refactor | Guardrails for multi-file / long-running refactors |
| jsonl-parsing | Large JSONL / agent log processing |
| bar-chart-comparison | Narrow ASCII bar charts for metric comparisons |
| readiness-scorecard | Scorecard TUI only when explicitly requested |
| neon-grit-image-style | Personal dark countercultural image aesthetic |
| security / review personas | insecure-defaults, sharp-edges, oauth-oidc-review, authorization-model-review, key-lifecycle-review, ssrf-confused-deputy-review, custom-crypto-detection, secrets-in-llm-output, rust-unsafe-ffi-review, differential-review, security-threat-model, audit-context-building, trailmark, static-analysis-triage |
| agent orchestration | abc-agent-management, peace-agent-interview, scramble, new-rfc, open-work-recap (always), agent-verify-workflows |
| comment-discipline | Comment review: describe code, not process |
| skill-brevity | Authoring/editing skills: keep only necessary lines |
| property-based-testing | PBT across languages |
| using-vit | ATProto caps / beacons (when relevant) |

## Manual Only (`disable-model-invocation: true` where set)

| Skill | Invocation | Description |
|-------|------------|-------------|
| git-pr | `/git-pr` | Stage, check, commit, push, create PR |
| git-create-pr | `/git-create-pr` | Full PR create workflow |
| git-final-pass | `/git-final-pass` | Pre-PR final pass |
| git-reset-workspace | `/git-reset-workspace` | Workspace cleanup |
| git-cleanup | `/git-cleanup` | Branches, worktrees, stashes |
| find-work / finding-uncommitted-work | `/find-work` | Uncommitted / unpushed / unmerged work |
| incomplete-work-audit | manual | Audit incomplete work surfaces |
| humanizer | `/humanizer` | Strip AI-writing patterns |
| project-init | `/project-init [topic]` | Initialize project framework |
| project | manual | Project skill hub (if present) |
| critique | `/critique` | Four-lens design review |
| design | `/design [topic]` | Feature design methodology |
| pqthink | `/pqthink` | Six-pass pragmatic architecture judgment |
| review-code | `/review-code` | Multi-agent code review |

## Grok-only user skills (`~/.grok/skills/`)

| Skill | Description |
|-------|-------------|
| check-work | Verification subagent for diffs / builds / tests |
| create-skill | Interactive Grok skill authoring |
| help | Grok TUI/docs/config help |
| imagine | Image gen/edit tool usage for Grok Build |
| code-review | Strict maintainability review (Grok user copy) |

Bundled Grok skills (`~/.grok/bundled/skills/`) are platform-provided (docx, pdf,
pptx, execute-plan, resume-*, game-*, etc.). Prefer project/user skills when names collide.

## Layout rules (all harnesses)

1. **Canonical tree:** `~/.claude/skills/<name>/SKILL.md` (+ optional `references/`).
2. **Do not fork:** `~/.agents/skills/*` are symlinks to Claude. Do not write divergent copies there.
3. **Grok-only** workflows belong in `~/.grok/skills/`; shared engineering skills belong under Claude.
4. **Project skills** live in `<repo>/.claude/skills/` or `<repo>/.grok/skills/`.
5. After layout changes, run `grok inspect` (Grok) or confirm Claude skill list still resolves.

## Backup

Pre-unification agents tree (if needed for archaeology):
`~/.agents/skills.bak-*`
