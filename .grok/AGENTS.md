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

### Publication (shared, do not weaken)

- Bead / `bd` IDs are **internal only** — never in commits, PRs, code, comments,
  docs, tickets, or any published text. Full rule in `~/.claude/Claude.md`.

## Always-active skills

At the start of any coding or multi-step engineering session, **read and apply**
these skill bodies only (not only their descriptions):

1. `~/.claude/skills/dry-engineering/SKILL.md` — default voice for engineering output other than C1 engineering-blog prose. That prose is rch-editor.
2. `~/.claude/skills/healthy-interaction/SKILL.md` — interaction baseline
3. `~/.claude/skills/pr-watch/SKILL.md` — PR event catch-up when squine is not connected (`pr-watch --since`). When squine MCP is connected, GitHub observe/care is squine `github_track` + `github_poll` (Claude.md dogfood).

**Context (decision-time, not every turn):**

- `open-work-recap` — end of a coding/status turn
- `project-process` — design → plan → RFC (one pipeline; no parallel methods)
- `passive-qol` — shell/dotfiles
- `engineering-guidelines` — judgment beyond Claude.md

Catalog: `~/.claude/CATALOG.md`. Every skill with a **Common Mistakes** section
must be read before work in that domain.

### Skill-tree rule

- **Write new or updated shared skills only under** `~/.claude/skills/<name>/`.
- Never recreate a divergent copy under `~/.agents/skills/` or `~/.grok/skills/`.
- `~/.agents/skills/*` must remain symlinks to Claude. Grok user skills under
  `~/.grok/skills/` are for Grok-only harness workflows (help, check-work, etc.).

## Grok harness adaptations

### Presented HTML

When the artifact the user is meant to read is HTML (this series: `read.html`), open it in Safari only:

```
open -a Safari <absolute-path>
```

Do not also `code` that post's markdown in the same turn. One viewer. Markdown without an HTML reading copy still uses `code --reuse-window --goto`. Working copies and reviewer notes stay in the background. Do not open a draft markdown file, a meta-review, an ambient note, a cold read, or a notes file beside a draft in VS Code as part of a revision. Open one only when the message is directing the user to something specific in that file.

### Models and subagents

Claude.md describes **cheap read-only subagents** for green-path build/test/git
(not main-session failure diagnosis). On Grok:

- There is no Haiku tier. Use `spawn_subagent` with a fitting `subagent_type`
  (`explore` read-only, `plan` for design, `general-purpose` for multi-step, or
  project agents such as `go-change-verifier`).
- For cheap green-path checks: fast general-purpose subagent with an explicit
  **Do NOT modify any files** brief.
- Failure diagnosis and tests expected to fail stay on the main session model.
- `git push` and any publishing still run in the main session, never delegated
  to a child that may rewrite code to satisfy hooks.

### Host sleep

Do not run `sleep-report`. The Grok `UserPromptSubmit` hook for it is removed.
Shared Claude.md still lists the skill; this harness does not.

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

1. Apply the always-active skill list above (bodies as needed).
2. If the repo uses beads: `bd prime` / `bd ready` as appropriate.
3. Prefer project `Agents.md` / `Claude.md` over inventing process.
4. Skills for the task: load from catalog; never invent a parallel procedure.

## When adding permanent guidance

If the user states an "always" rule, add it to `~/.claude/Claude.md` (shared)
unless it is Grok-harness-only (then add it here under **Grok harness
adaptations**). Do not fork the shared rulebook into a Grok-only copy.
