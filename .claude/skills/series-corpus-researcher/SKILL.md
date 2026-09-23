---
name: series-corpus-researcher
description: Search repos the author created, commits they shipped to main, and PRs they originated, since 2025-10, for material adjacent to the C1 engineering-blog series. Use when the user asks for new post topics, more context for current drafts, or a corpus pass.
---

# Series corpus researcher

Find unused work the author originated that belongs next to a live draft or as a new post. Write a dated report. Do not edit drafts unless asked.

Public phrase: logic programming paradigms in use at C1. MUST NOT name Occult, latchkey, bead ids in suggested public copy. Author-only sources may use local paths.

## Common Mistakes

- Citing C1 material the author only touched (review, follow-up, rename, CI, a later edit on someone else's surface). Original author only.
- Searching PRs only. Also: repos they created, and commits they shipped directly to `main`.
- Stopping at a GitHub 403. Read `gh api rate_limit`. If remaining is 0, wait until `reset`, then continue. Checkpoint to a file under `reports/`. Finish the dataset.
- Suggesting a post that restates a live INDEX title.
- Naming the engine product or latchkey in a public suggestion.
- Stopping at surfaces already in INDEX.
- Capping the list early. Paginate. Walk every local path that exists. One line per distinct product+CS pair the author originated.
- A claim with no source path, PR URL, or commit SHA.
- Inventing UI or product behavior not in the source.

## Authorship gate

Cite a fact only when the author originated it. Evidence, in order:

1. They created the repo (`gh repo view --json isPrivate,createdAt,owner` plus creator; `gh search repos --owner=robert-chiniquy`; org repos whose creator is `robert-chiniquy`).
2. They shipped the introducing commit to `main` (or `master`): `git log origin/main --author='Robert Chiniquy' --author='rchiniquy@' --since=2025-10-01`. For a path, the first add on main is theirs (`git log origin/main --diff-filter=A --follow --format='%an %ae %H' -- <path>`).
3. They opened the PR that introduced the surface, and they wrote the body of that change. A later PR that only touches, renames, tests, or reviews someone else's work does not count.

If the first commit on the path is someone else, omit it, or label it `touched-not-cited` in methodology. MUST NOT put touched-not-cited items on Add-to-draft or New-post.

Author emails seen in-tree: `rchiniquy@yahoo.com`, `robert.chiniquy@conductorone.com`. GitHub login: `robert-chiniquy`. Window: 2025-10-01 onward (C1 start ~11 months before 2026-09).

## Corpus

Live and published titles: `/Users/rch/repo/research/equational-reasoning/INDEX.md` and gist `01-OUTLINE.md`.

Local (skip missing): every path in `/Users/rch/.claude/codebases.json`, plus `/Users/rch/repo/c1`, `/Users/rch/repo/cone`, `/Users/rch/repo/docs`, `/Users/rch/repo/occult-sigil`, `/Users/rch/repo/occult`, every `/Users/rch/repo/baton-*`.

GitHub (do not inherit the workspace repo):

```
gh search repos --owner=robert-chiniquy --limit 100
gh search prs --author=robert-chiniquy --owner=ductone --created=">=2025-10-01" --limit 100
gh search prs --author=robert-chiniquy --owner=ConductorOne --created=">=2025-10-01" --limit 100
gh search commits --author=robert-chiniquy --author-date=">=2025-10-01" --limit 100
git -C <repo> log origin/main --author='Robert Chiniquy' --since=2025-10-01 --pretty='%h %ad %s' --date=short
```

Paginate search with `created:` windows. Full PR URL or commit SHA at every mention. Record: repos created, commits-to-main counted, PRs listed, local repos opened.

## Rate limits

```
gh api rate_limit --jq '{core: .resources.core, search: .resources.search}'
```

If `remaining` is 0, wait until `reset` (Unix seconds), then resume from the checkpoint. MUST NOT drop the rest of the dataset. MUST NOT background the wait with `&`. Foreground wait is allowed. Checkpoint: `/Users/rch/repo/research/equational-reasoning/reports/_corpus_authored_checkpoint.md` (listings, windows done, remaining queries).

## Pass

1. One line per live/published post: C1 surface + CS concept.
2. List repos the author created. List commits they shipped to `main`. Then PRs they originated.
3. Authorship gate each hit.
4. Classify: already in a draft, add-to-draft, or new-post.
5. Write a new dated report (do not overwrite an earlier same-day report). Two lists: Add to a current draft; New post. Each item: title or draft id, one-sentence claim, source (path, PR URL, or commit SHA) plus a one-line authorship reason (`created repo`, `first commit on path`, `originating PR`). Methodology names counts and how many candidates failed the authorship gate.

## Before finishing

- Report path exists
- Every listed item has a source and an authorship reason
- No touched-not-cited item on the two lists
- Rate-limit waits completed; checkpoint windows are done or named as remaining
- Public suggestions omit Occult, latchkey, bead ids
