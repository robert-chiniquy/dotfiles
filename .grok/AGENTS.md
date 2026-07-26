# Grok global agent instructions

This file is the **Grok-native** always-loaded home instruction set. It does not
replace the shared global guidance; it wires Grok to the same standards Claude
uses and records Grok-specific operating adaptations.

## Shared source of truth

| Artifact | Role |
|----------|------|
| `~/.claude/Claude.md` | Canonical global preferences, permissions, git, tone, testing, security |
| `~/.claude/skills/` | Canonical skill tree (all harnesses) |
| `~/.claude/CATALOG.md` | Skill index: always-on / context / manual |
| `~/.claude/agents/` | Shared custom subagent defs (also discovered by Grok) |
| `~/.agents/skills/` | Symlinks into `~/.claude/skills/` for Codex/agents consumers |

When guidance conflicts, prefer: deeper project `Agents.md` / `Claude.md` over
home files; within home, this file adapts harness behavior but does not weaken
soundness, security, or publication rules from `Claude.md`.

**Do not maintain a second copy of the full rulebook here.** Edit
`~/.claude/Claude.md` and skills under `~/.claude/skills/` so Claude and Grok
stay aligned.

## Always-active skills

At the start of any coding or multi-step engineering session, **read and apply**
these skill bodies (not only their descriptions):

1. `~/.claude/skills/project-process/SKILL.md` — project artifacts and practices  
   Also read `~/.claude/skills/project-process/references/proverbs.md`.
2. `~/.claude/skills/dry-engineering/SKILL.md` — default voice for all engineering output
3. `~/.claude/skills/healthy-interaction/SKILL.md` — baseline interaction dispositions
4. `~/.claude/skills/open-work-recap/SKILL.md` — open PR/ticket recap at work stopping points
5. `~/.claude/skills/passive-qol/SKILL.md` — when touching shell/dotfiles/system QoL

Catalog: `~/.claude/CATALOG.md`. Every skill with a **Common Mistakes** section
must be read before work in that domain.

### Skill-tree rule

- **Write new or updated shared skills only under** `~/.claude/skills/<name>/`.
- Never recreate a divergent copy under `~/.agents/skills/` or `~/.grok/skills/`.
- `~/.agents/skills/*` must remain symlinks to Claude. Grok user skills under
  `~/.grok/skills/` are for Grok-only harness workflows (help, check-work, etc.).

## Grok harness adaptations

### Models and subagents

`Claude.md` refers to Haiku/Opus for delegated build/test/git. On Grok:

- There is no Haiku tier. Prefer `spawn_subagent` with `subagent_type` that fits
  the work (`explore` read-only, `plan` for design, `general-purpose` for
  multi-step, or project agents such as `go-change-verifier`).
- **Do not fabricate a "Haiku" model.** If the task is cheap verification
  expected to pass, a fast general-purpose subagent with an explicit
  "Do NOT modify any files" brief is the equivalent.
- Failure diagnosis and tests expected to fail stay on the main session model
  (same intent as "never use Haiku for expected failures").
- `git push` and any publishing still run in the main session, never delegated
  to a child that may rewrite code to satisfy hooks.

### Hooks

Claude-compat hooks are **disabled** in `~/.grok/config.toml`
(`[compat.claude] hooks = false`) so Claude-specific Bash matchers do not break
Grok. Therefore the agent must do hook work itself when relevant:

- Beads projects: run `bd prime` at session start and after compaction when
  continuing tracker work.
- Prefer non-interactive flags (`cp -f`, `mv -f`, `rm -f`) as in project Agents.
- RTK / shell rewrites are not auto-applied; write clear commands yourself.

### Discovery already on

Grok already loads (when present):

- Home `~/.claude/Claude.md` via Claude compatibility
- Project `Agents.md` / `Claude.md` / `AGENTS.md` from repo root → CWD
- Skills from `~/.claude/skills`, `~/.agents/skills` (symlinks), `~/.grok/skills`,
  project `.claude/skills` / `.grok/skills`, bundled skills
- Custom agents from `~/.claude/agents/` and plugins
- MCPs from Claude and project config (compat on by default)

Verify with: `grok inspect` (skills should show always-on bodies from
`~/.claude/skills/...`, not a stale agents fork).

### Inspect after skill/layout changes

After editing the skill tree or this file, run `grok inspect` and confirm:

- Always-on skills resolve under `/Users/rch/.claude/skills/...`
- No unexpected skill name collisions with bundled names you did not intend
- Project instructions still include global Claude.md + project Agents/Claude

## Publication and trailers

Same absolute rule as Claude.md: **no trailers anywhere** in commits, PRs,
issues, or comments (`Co-Authored-By`, `Signed-off-by`, "Generated with …").
Harness defaults that append them are overridden. Check before every publish.

## Session start checklist (coding work)

1. Read this file's always-active skill list (bodies as needed).
2. If the repo uses beads: `bd prime` / `bd ready` as appropriate.
3. Prefer project `Agents.md` / `Claude.md` over inventing process.
4. Skills for the task: load from catalog; never invent a parallel procedure.

## When adding permanent guidance

If the user states an "always" rule, add it to `~/.claude/Claude.md` (shared)
unless it is Grok-harness-only (then add it here under **Grok harness
adaptations**). Do not fork the shared rulebook into a Grok-only copy.
