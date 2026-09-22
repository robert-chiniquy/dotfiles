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
| pr-watch | PR event catch-up: `pr-watch --since` after any prior look |
| sleep-report | Host sleep: `sleep-report --since` at turn start; do not print `SLEEP` to the user |

## Context-Activated

| Skill | Trigger |
|-------|---------|
| open-work-recap | End of coding/status turn: open items + Next checklist |
| project-process | Non-trivial work: design → plan → RFC hub (one path) |
| engineering-guidelines | Design, debug, review, or quality judgment beyond Claude.md |
| passive-qol | Dotfiles, shell config, system QoL, friction |
| casual-slack-tone | Slack messages, DMs, PR descriptions on own repos |
| technical-writing | Blog posts, articles, long-form external content |
| technical-writing-voice | Long-form external voice (articles, talks) |
| structural-constraints | Architecture decisions, type system design |
| subprocess-lifecycle | Spawning a child process: ownership mode, parent-death, signals, reaping |
| checkoutless-github-publish | No writable git checkout: scratch-tree verification + gh data API publish/rebase with the whole-file divergence gate |
| terraform / terraform-skill | .tf files, HCL, infrastructure pipelines |
| protogen | .proto files, gRPC, codegen |
| documentation | Writing or reviewing docs |
| design | Stage 1 default depth (`/design`); see project-process |
| systematic-feature-design | Stage 1 **only** when large/architecture (not a second pipeline) |
| designing-occult-application | Userspace Occult app: theory, middle functor, G, last hop |
| occult-engine-work | Occult engine implementer gates (residual, one path, identity); complements principled-review |
| rigorous-critique | Stage 2 **only** critique path (canonical) |
| critique | `/critique` alias for rigorous-critique job |
| new-rfc | Stage 3 **only** RFC path |
| socratic-discovery | Progressive questions for consensus / assumptions |
| complete-developer-experience | Tools + docs + agents for developer-facing features |
| overcorrection-review | Needless complexity, premature exclusions, cost/value claims |
| post-change-verification | After Go code changes: fmt/lint/build/test protocol |
| pre-push-self-review | Before push/PR: written graph-walk of your own diff (callers, teardown, enum consumers, falsified comments) |
| golang-code-review | Go PR / architecture / test quality review |
| invalid-cache-review | Caches, memos, derived indexes: hit rate, staleness, and measurement corruption |
| pr-pass | Open PR triage and status |
| mergeability-walkthrough | One-PR-at-a-time merge decisions |
| github-pr-threads | After fixes: `Addressed in <sha>` + resolve threads |
| pr-deep-review | Multi-agent deep PR review |
| gh-fix-ci | Failing GitHub Actions checks |
| squire-env-management | Ephemeral remote agents and task pools |
| c1-squire-dispatch / c1-dev-stack-in-squire | c1-specific squire dispatch |
| on-call | ConductorOne on-call: docs/channels/paging quickref + agent capability map |
| organizer | Calendar writes without API write access: emit .ics + OS `open` |
| squire-qol | Personalize squire envs (dotfiles/nix), user skills/files, env vars |
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
| rfc-surface-clusters | RFC surfaces → workflow clusters, ranked by Slack+Linear UX damage |
| questioning-the-user | Multiple pending decisions → one at a time |
| terms-pass | Series TERMS.md gloss pass: one file, one term at a time, drafts wait |
| information-flow | Nested conceptual-progress outline, then reorder the post if the outline is better |
| subagent-prompt-review | Before Agent() / squire dispatch / scheduled remote agents |
| agent-worktree-status | Background agent worktree liveness |
| agent-verify-workflows | Explicit web workflow verification |
| abc-agent-management | Improve subagent prompts after poor runs |
| peace-agent-interview | Elicit uncontaminated account after bad subagent run |
| scramble | Parallel local tactical push |
| comment-discipline | Comments describe code, not process |
| retcon | Before share: keep code, learnings, commit messages; strip author and clocks |
| retcon-review | Reviewer persona: provenance leaks and missing work-local retcon derivatives |
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
| resume-after-host-reboot | Same-pane continue after host reboot (not resume-codex) |
| occult-app-naming | Name Occult app identifiers; do not explain them |
| owner-design-lock | Do not overturn the user's design without asking |
| work-in-checkout-not-tmp | Work in the repo or a worktree, not /private/tmp |
| no-compat-surfaces | Cut over only; no compatibility shims |
| standing-production-merge | Simple production PRs merge when reviewed/approved/CI green; complex only with permission |
| occult-lsp | occult_lsp only sees the session workspace root |
| occult-factoring-discipline | Weaker-LLM Occult factoring; one concept one spine |
| expected-work-inventory | Personal expected-work list across trackers |
| occult-lint | occult-lint unroll/exists/plane rules |

## Manual Only (`disable-model-invocation: true` where set)

| Skill | Invocation | Description |
|-------|------------|-------------|
| git-create-pr | `/git-create-pr` | Full PR create workflow |
| git-final-pass | `/git-final-pass` | Pre-PR final pass |
| git-reset-workspace | `/git-reset-workspace` | Workspace cleanup |
| finding-uncommitted-work | `/finding-uncommitted-work` | Uncommitted / unpushed / unmerged work |
| incomplete-work-audit | manual | Audit incomplete work surfaces |
| humanizer | `/humanizer` | Strip AI-writing patterns |
| project | `/project [topic]` | Initialize project framework |
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

## Work pipeline (project-process) — one path only

1. **Design** → `DESIGN_<topic>.md` — depth skill: `design`, or `systematic-feature-design` if large  
2. **Plan** → `PLAN_<OBJECTIVE>.md` — critique with `rigorous-critique` before ready  
3. **RFC** → `new-rfc` only — consumes design+plan; owner gate; no auto-impl  

Skip stages for trivial work. **MUST NOT** invent parallel methodologies (pqthink/socratic are lenses, not alternate pipelines).

## Skill construction (compliance)

- Skills >~100 lines: **Common Mistakes first** (or immediately after a 1-paragraph intro).  
- High-stakes skills: end with **Before finishing** (≤5 checkboxes).  
- Gates use MUST/MUST NOT, not prefer/should.

## Backup

Pre-unification agents tree (if needed for archaeology):
`~/.agents/skills.bak-*`
