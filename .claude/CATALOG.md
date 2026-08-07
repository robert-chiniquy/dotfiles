# Skills Catalog

Shared skill index for **Claude Code, Grok Build, Codex, and other harnesses**.
Canonical skill tree: `~/.claude/skills/`. Codex's user-skill root,
`~/.agents/skills/`, is symlinked to it. OMX separately manages
`~/.codex/skills/`; Codex may show both entries when an OMX skill and a shared
skill have the same name. Grok also loads `~/.grok/skills/` (Grok-only harness
skills) and project skills.

Count the tree with `ls ~/.claude/skills | wc -l`. Always-on skills are listed in
`~/.claude/Claude.md` and (for Grok) `~/.grok/AGENTS.md` — agents must **read the
skill bodies**, not only this index.

---

## Always On

| Skill | Description |
|-------|-------------|
| dry-engineering | Default voice: code review style, commit messages, explanations |
| healthy-interaction | Baseline interaction dispositions (no sycophancy, no therapy mode) |
| open-work-recap | Coding stopping-point recap: open PRs/tickets/issues (URLs) + Next |
| project-process | Thin hub: optional artifacts + design → plan → RFC pipeline |

## Context-Activated

| Skill | Trigger |
|-------|---------|
| engineering-guidelines | Design, debug, review, or quality judgment beyond Claude.md |
| passive-qol | Dotfiles, shell config, system QoL, friction |
| casual-slack-tone | Slack messages, DMs, PR descriptions on own repos |
| technical-writing | Blog posts, articles, long-form external content |
| technical-writing-voice | Long-form external voice (articles, talks) |
| structural-constraints | Architecture decisions, type system design |
| terraform / terraform-skill | .tf files, HCL, infrastructure pipelines |
| protogen | .proto files, gRPC, codegen |
| documentation | Writing or reviewing docs |
| design | `/design` — stage 1 of design → plan → RFC |
| systematic-feature-design | Large feature / architecture design (stage 1 depth) |
| rigorous-critique / critique | After a plan, before treating it as ready (stage 2) |
| new-rfc | Stage 3: adversarially reviewed RFC from design+plan |
| socratic-discovery | Progressive questions for consensus / assumptions |
| complete-developer-experience | Tools + docs + agents for developer-facing features |
| overcorrection-review | Needless complexity, premature exclusions, cost/value claims |
| post-change-verification | After Go code changes: fmt/lint/build/test protocol |
| golang-code-review | Go PR / architecture / test quality review |
| pr-pass / pr-status | Open PR triage and status |
| mergeability-walkthrough | One-PR-at-a-time merge decisions |
| github-pr-threads | After fixes: `Addressed in <sha>` + resolve threads |
| pr-deep-review | Multi-agent deep PR review |
| gh-fix-ci | Failing GitHub Actions checks |
| squire-env-management | Ephemeral remote agents and task pools |
| c1-squire-dispatch / c1-dev-stack-in-squire | c1-specific squire dispatch |
| find-delegation-pebbles | Bounded independent backlog tasks for remote agents |
| codebase-memory | Structural codebase graph exploration |
| large-scale-refactor | Multi-file / long-running refactors |
| jsonl-parsing | Large JSONL / agent log processing |
| bar-chart-comparison | Narrow ASCII bar charts for metric comparisons |
| readiness-scorecard | Scorecard TUI only when explicitly requested |
| neon-grit-image-style | Personal dark countercultural image aesthetic |
| refine-illustrations-iteratively | Iterative image edit sessions |
| calendaring | Multi-month personal master schedule |
| tactical-sitrep | Named milestone + hard deadline → readiness |
| questioning-the-user | Multiple pending decisions → one at a time |
| subagent-prompt-review | Before Agent() / squire dispatch / scheduled remote agents |
| agent-worktree-status | Background agent worktree liveness |
| agent-verify-workflows | Explicit web workflow verification |
| abc-agent-management | Improve subagent prompts after poor runs |
| peace-agent-interview | Elicit uncontaminated account after bad subagent run |
| scramble | Parallel local tactical push |
| comment-discipline | Comments describe code, not process |
| skill-brevity | Authoring/editing skills: keep only necessary lines |
| property-based-testing | PBT across languages |
| using-vit | ATProto caps / beacons |
| check-feature-flag-conflicts | FEATURE_FLAG_ID conflicts before adding flags |
| gestalt-consistency-review | Correct-but-odd-one-out APIs |
| insecure-defaults | Fail-open / weak default security |
| sharp-edges | Footgun APIs and dangerous config |
| oauth-oidc-review | OAuth/OIDC implementation review |
| authorization-model-review | RBAC/ABAC/ReBAC authz review |
| key-lifecycle-review | Key/secret lifecycle review |
| ssrf-confused-deputy-review | SSRF / confused deputy |
| custom-crypto-detection | Hand-rolled crypto |
| secrets-in-llm-output | Secrets leaked into agent output |
| rust-unsafe-ffi-review | Rust unsafe / FFI soundness |
| differential-review | Security-focused PR/diff review |
| security-threat-model | Explicit threat model request |
| audit-context-building | Line-by-line audit context |
| trailmark | Code graph for security analysis |
| static-analysis-triage | Novel linter output → PRs |

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
| design | `/design [topic]` | Feature design (pipeline stage 1) |
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
6. **Codex/agents mirror:** `scripts/install-shared-agent-skills.sh` makes
   `~/.agents/skills` a symlink to the canonical tree. `install.sh` runs it;
   never restore a hand-picked per-skill subset.
7. **OMX overlaps are separate:** Codex does not merge same-name skills from
   `~/.codex/skills/` and `~/.agents/skills/`. Keep both roots intact; resolve a
   real semantic collision by renaming or disabling one explicit skill path,
   not by shrinking the shared mirror.

## Work pipeline (project-process)

1. **Design** → `DESIGN_<topic>.md` (`design` / `systematic-feature-design`)
2. **Implementation plan** → `PLAN_<OBJECTIVE>.md` (critique before ready)
3. **RFC** → `new-rfc` (consumes design + plan; owner gate; no auto-impl)

Skip stages for trivial work; do not invent a later stage that ignores an earlier artifact.

## Backup

Pre-unification agents tree (if needed for archaeology):
`~/.agents/skills.bak-*`
