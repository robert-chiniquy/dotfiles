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
- Surveying all of GitHub. Stay on series-adjacent C1 surfaces (CEL, classifiers, grants, SAML, vaulting, connectors, compaction, session policy, role mining, deadlocks, XML Signature).
- A claim with no source path or PR URL.
- Inventing UI or product behavior not in the source.

## Corpus

Live and published titles: `/Users/rch/repo/research/equational-reasoning/INDEX.md` and gist `01-OUTLINE.md` (https://gist.github.com/robert-chiniquy/aa2ef88faf0eac08b4a1f86cc6ad8027).

Local (read; do not assume every path exists): `/Users/rch/repo/c1`, `/Users/rch/repo/cone`, `/Users/rch/repo/baton-sdk`, `/Users/rch/repo/connector-registry`, `/Users/rch/repo/occult-sigil`, `/Users/rch/repo/occult`, `/Users/rch/.claude/codebases.json`.

GitHub since 2025-10-01 (author started at C1 ~11 months before 2026-09): `gh search prs --author=robert-chiniquy --created=">=2025-10-01" --limit 50`, `gh search commits --author=robert-chiniquy --author-date=">=2025-10-01" --limit 50`, then org/repo filters (`conductorone`, personal). Full PR URL at every mention.

## Pass

1. One line per live/published post: C1 surface + CS concept.
2. Search local code/docs for those surfaces and adjacent ones (Okta SAML roles, session CEL, vault epochs, last-write indexes, connector draft names, role mining preview).
3. Search GitHub PRs/commits in the same window.
4. For each hit: already in a draft, add-to-draft, or new-post.
5. Write `/Users/rch/repo/research/equational-reasoning/reports/REPORT_SERIES_CORPUS_YYYY-MM-DD.md` with two lists only when they have items: Add to a current draft; New post. Each item: title or draft id, one-sentence claim, source (absolute local path or full GitHub URL). New-post titles are one CS keyword + one C1 product keyword.

## Before finishing

- Report path exists
- Every item has a source
- No restated INDEX title as a new post
- Public suggestions omit Occult, latchkey, bead ids
