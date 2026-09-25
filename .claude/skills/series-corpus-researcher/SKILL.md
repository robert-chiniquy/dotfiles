---
name: series-corpus-researcher
description: Find unused work the author originated for the C1 engineering-blog series. Repos they created, commits they shipped to main, originating PRs. Use when the user asks for new post topics, more draft context, or a corpus pass.
---

# Series corpus researcher

Write a new dated report. Do not edit drafts unless asked. Do not reuse an old corpus report; start the survey from empty.

Public phrase: logic programming paradigms in use at C1. MUST NOT name Occult, latchkey, or bead ids in suggested public copy. Author-only sources may use local paths.

## Common Mistakes

- Citing C1 material the author only touched (review, follow-up, rename, CI, a later edit). Original author only.
- Searching PRs only. Also: repos they created, and commits they shipped to `main`.
- Stopping at a GitHub 403. Read `gh api rate_limit`. If remaining is 0, wait until `reset`, checkpoint, continue. Finish the dataset.
- A new-post item that is only an implementation fact (`refresh failures are classified from the typed error class`). Every new post MUST carry one observation of interest worth public discussion under the author's name: a stake a C1 engineering-blog reader would argue about, not a typed-error cleanup.
- Restating a live INDEX title.
- Naming the engine product or latchkey in public copy.
- Inventing UI.
- Putting an as-yet-unfixed C1 production defect on Add-to-draft or New-post as public copy. Surface those privately. Public items use a class and a constructed exhibit, or wait until the hole is closed.

## Authorship gate

Cite only when the author originated the surface.

1. They created the repo.
2. They shipped the introducing commit to `main`/`master`: first add on that path is theirs.
3. They opened the PR that introduced the surface and wrote that change. Review-only, CI-only, rename-only, and follow-ups on someone else's design fail.

If the first commit on the path is someone else: omit, or list under methodology as `touched-not-cited`. MUST NOT put those on Add-to-draft or New-post.

Author: GitHub `robert-chiniquy`. Emails: `rchiniquy@yahoo.com`, `robert.chiniquy@conductorone.com`. Window: 2025-10-01 onward, plus earlier work they originated that the series can still use (Rust Occult, static analysis).

## Interest gate (new posts)

For each New post, write:

- Title: one CS keyword + one C1 product keyword
- Claim: one sentence
- Observation of interest: one sentence a reader would discuss under this author's name (failure, invariant, trick, or unrepresentable extra). If you cannot write that sentence, drop the item.
- Source + authorship reason (`created repo`, `first commit on path <sha>`, `originating PR <url>`)

Fail example: "Typed class in MCP OAuth refresh" with no stake.
Pass example: "A checkpoint write that omits AssignedTo = this worker would let a second consumer replace the live grant sequence."

## Corpus

INDEX: `/Users/rch/repo/research/equational-reasoning/INDEX.md`. Gist outline: `/Users/rch/repo/research/equational-reasoning/gist/01-OUTLINE.md`.

Must open:

- `/Users/rch/repo/occult-go-analysis` (README title occult-static-analysis)
- `/Users/rch/repo/occult-rust` (original Rust Occult)
- `/Users/rch/repo/occult` (`goanalysis/`)
- `/Users/rch/repo/occult-sigil`
- `/Users/rch/repo/c1`, `/Users/rch/repo/cone`, `/Users/rch/repo/docs`
- every `/Users/rch/repo/baton-*` that exists
- every path in `/Users/rch/.claude/codebases.json` that exists
- repos they created (`gh search repos --owner=robert-chiniquy`; org repos whose creator is `robert-chiniquy`)

GitHub (do not inherit the workspace repo). Paginate. Commits to main on local clones:

```
git log origin/main --author='Robert Chiniquy' --since=2025-10-01 --pretty='%h %ad %s' --date=short
```

First add on a cited path:

```
git log origin/main --diff-filter=A --follow --format='%an %ae %H %s' -- <path>
```

## Rate limits

```
gh api rate_limit --jq '{core: .resources.core, search: .resources.search}'
```

Remaining 0: wait until `reset` in the foreground (no `&`). Checkpoint: `/Users/rch/repo/research/equational-reasoning/reports/_corpus_checkpoint.md`. Resume until windows are done.

## Pass

1. Live/published titles, one line each.
2. Repos created, then commits to main, then originating PRs.
3. Authorship gate, then interest gate.
4. Classify: already in a draft, add-to-draft, new-post.
5. Write `/Users/rch/repo/research/equational-reasoning/reports/REPORT_SERIES_CORPUS_YYYY-MM-DD.md` (new file; never overwrite). Methodology: repos created, commits-to-main, PRs listed, opened local repos, rejected-touched count, items dropped for no public stake.

Skip as new posts: exclusive Kinesis lease (already in KR gist); duplicate Assertion id (already in 15).

## Before finishing

- Report path exists and is a new file
- Every listed item has source, authorship reason, and (for new posts) an observation of interest
- No touched-not-cited item on the two lists
- Rate-limit waits finished or remaining windows named
- Public copy omits Occult, latchkey, bead ids
