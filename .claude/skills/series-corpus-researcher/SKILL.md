---
name: series-corpus-researcher
description: Search local C1 and related code, docs, and the author's GitHub work since 2025-10 for material adjacent to the C1 engineering-blog series. Use when the user asks for new post topics, more context for current drafts, or a corpus pass over GitHub plus local code.
---

# Series corpus researcher

Find unused C1 work that belongs next to a live draft or as a new post. Write a dated report. Do not edit drafts unless asked.

Public phrase: logic programming paradigms in use at C1. MUST NOT name Occult, latchkey, bead ids in suggested public copy. Author-only sources may use local paths.

## Common Mistakes

- Suggesting a post that restates a live INDEX title.
- Naming the engine product or latchkey in a public suggestion.
- Stopping at surfaces already in INDEX. The job is unused C1 work that can become a post, including surfaces the series has not named yet (MCP OAuth, findings, connectors, proto, CLI, Terraform, SDKs, every baton connector).
- Capping the list early. Paginate GitHub. Walk every local codebase path that exists. One line per distinct product+CS pair. Dozens to 100+ is the expected grain if the corpus supports it. MUST NOT stop at a dozen because the first hits were on CEL and SAML.
- A claim with no source path or PR URL.
- Inventing UI or product behavior not in the source.

## Corpus

Live and published titles: `/Users/rch/repo/research/equational-reasoning/INDEX.md` and gist `01-OUTLINE.md` (https://gist.github.com/robert-chiniquy/aa2ef88faf0eac08b4a1f86cc6ad8027).

Local (read; skip missing): every path in `/Users/rch/.claude/codebases.json`, plus `/Users/rch/repo/c1`, `/Users/rch/repo/cone`, `/Users/rch/repo/docs`, `/Users/rch/repo/occult-sigil`, `/Users/rch/repo/occult`, and every `/Users/rch/repo/baton-*` directory that exists.

GitHub since 2025-10-01 (author started at C1 ~11 months before 2026-09). Paginate. Do not inherit the current workspace repo. Explicit owner filters:

```
gh search prs --author=robert-chiniquy --owner=ductone --created=">=2025-10-01" --limit 100
gh search prs --author=robert-chiniquy --owner=ConductorOne --created=">=2025-10-01" --limit 100
gh search prs --author=robert-chiniquy --created=">=2025-10-01" --limit 100
```

Same for commits (`--author-date`). Full PR URL at every mention. Record how many PRs and how many local repos were actually opened.

## Pass

1. One line per live/published post: C1 surface + CS concept.
2. Search local code/docs for those surfaces and adjacent ones (Okta SAML roles, session CEL, vault epochs, last-write indexes, connector draft names, role mining preview).
3. Search GitHub PRs/commits in the same window.
4. For each hit: already in a draft, add-to-draft, or new-post.
5. Write a new dated report (do not overwrite an earlier same-day report). Two lists: Add to a current draft; New post. Each item: title or draft id, one-sentence claim, source (absolute local path or full GitHub URL). New-post titles are one CS keyword + one C1 product keyword. Methodology must name PR count and repo count surveyed.

## Before finishing

- Report path exists
- Every item has a source
- No restated INDEX title as a new post
- Public suggestions omit Occult, latchkey, bead ids
