---
name: retcon
description: >
  Deep-scrub a git repo so code, learnings, and commit-message text survive,
  while author identity and every clock disappear. After retcon, git log and
  the tree look like they were created in a burst of seconds immediately
  before share/commit/push. Triggers: retcon, scrub history, anonymize
  commits, strip dates before sharing, make it look just-written, remove
  author/committer/timestamps, unsign commits, share without provenance.
---

# Retcon

Rewrite a repo you own so a recipient cannot tell it existed before the
share. Keep trees, learnings, and commit-message text. Strip identity and
time from git metadata **and** file contents. Git still needs a name/email;
use a supplied synthetic identity, never the machine's `user.name` /
`user.email`.

## Common Mistakes

1. **Rewriting a branch already on a remote and calling it unpublished.**
   Forges and anyone who fetched keep the old objects. Retcon is a **new
   object graph**. Share a new clone or a new remote. MUST NOT force-push
   over a URL others already have and treat that as a scrub.
2. **Squashing to one commit.** That deletes the commit messages. Replay
   every commit (and merges) with the same topology.
3. **Using `git config user.name` / `user.email`.** That **is** the author
   leak. Identity comes from the operator (`RETCON_NAME`, `RETCON_EMAIL`)
   or one question. MUST NOT default to the host git identity.
4. **Author date only.** `git log --format=fuller` still shows committer
   date. Set **both**. Signatures (`gpgsig`) also prove identity and time:
   drop them.
5. **`git filter-repo`, BFG, or Python.** Python is banned. MUST use git
   plumbing (`fast-export` / `fast-import`, or `filter-branch` env+msg
   filters) plus a rewriter in the **repo's** language if a program is
   required. No Python one-liners.
6. **Leaving clocks in the tree.** LEARNINGS `## YYYY-MM-DD HH:MM`,
   "Locked 2026-08-24", RFC/CHANGELOG dates, `Date:` headers, ISO-8601 in
   comments, copyright year **ranges**, image EXIF, PDF metadata. Metadata
   rewrite without a content scrub still leaks.
7. **Stripping citation years.** Paper years, language editions (`edition
   = "2024"`), protocol versions stay. Clocks and bylines go.
8. **Leaving trailers in messages.** Drop `Co-Authored-By`, `Signed-off-by`,
   `Generated with`, `Made-with:`, and any `Key: value` trailer block. Drop
   ISO timestamps in the subject/body.
9. **Forgetting `refs/original`, notes, reflog, tags, packed-refs.** After
   rewrite: delete `refs/original/*`, `refs/notes/*`, expire reflog, `gc
   --prune=now`. Move tags onto the new commits (same names).
10. **Retcon in the only clone.** Snapshot the unretoned repo first (a
    sibling directory, not system temp). The original clocks are not
    recoverable from the retconed graph.
11. **Worktree-only rewrite.** Retcon the object store. Submodules are
    separate repos: fail closed unless the operator asks to retcon each.
12. **Publishing bead/tracker ids or local paths.** `/Users/…`, hostnames,
    tracker ids, and "Generated with …" in files are identity. Strip if
    the dest will leave the machine.

## Procedure

Work on a **copy**. Original stays untouched.

1. **Identity.** Require `RETCON_NAME` and `RETCON_EMAIL`. If unset, ask
   once. MUST NOT read `git config user.*`.
2. **Backup.** Clone or `git clone --mirror` the source to a sibling path.
   Record `git rev-parse HEAD` of the original.
3. **Inventory clocks and bylines** in tracked files (not ignored junk):
   - `git log --format='%an%n%ae%n%cn%n%ce'` (unique) — those strings in
     the tree are leaks
   - `## YYYY-MM-DD` headers, `\d{4}-\d{2}-\d{2}([ T]\d{2}:\d{2})?`
   - emails, `/Users/`, `/home/`, `Host:`, `Date:`
   - image/PDF metadata if any binaries are tracked
   Structured files (YAML, JSON, TOML, XML): parse, do not regex. Plain
   prose/markdown: line edits are fine.
4. **Content scrub on the copy's working tree**, then the history replay
   must include those trees. LEARNINGS: keep topic + body; restamp headers
   into the burst window (order preserved) or drop the clock, keep the
   title. Do not delete the learning.
5. **History replay** into a fresh repo (empty `git init`):
   - Export: `git fast-export --all --signed-tags=strip --tag-of-filtered-object=rewrite`
   - Rewrite every `author` / `committer` line to
     `{RETCON_NAME} <{RETCON_EMAIL}> {t} +0000`
   - `{t}` is Unix time **now**, then +1 second per commit in export
     order (monotonic, burst).
   - Strip message trailers and message-body clocks.
   - Drop `gpgsig` / `sign` blocks.
   - Import: `git fast-import --reset` in the fresh repo.
   Preserve marks so merges and tags still point at the rewritten
   commits.
6. **Purge residue** in the fresh repo: no `refs/original`, no notes,
   `git reflog expire --expire=now --all`, `git gc --prune=now`.
7. **Verify** (fail the retcon if any check fails):
   - `git log --format=fuller --all`: every author/committer is the
     synthetic identity; every date is within the burst (seconds, not
     days).
   - `git cat-file -p HEAD` (and a merge commit if any): no `gpgsig`.
   - `git log --format=%B --all`: no Co-Authored-By / Signed-off-by /
     Generated with / ISO date.
   - `git rev-list --objects --all` paths: no leftover original emails,
     local home paths, or LEARNINGS clocks.
   - `git log --show-signature --all`: unsigned.
   - Object count may shrink (signatures/notes gone); trees of tracked
     source must still build/test if the original did.

## Burst clock

Commits keep order. Dates are `now`, `now+1s`, `now+2s`, … so `git log`
reads as one sitting. MUST NOT keep original relative gaps (those are a
calendar). MUST NOT use a single identical timestamp for every commit
(some receivers treat that as a rewrite artifact; one-second steps look
like rapid local work).

## Share path

The retconed repo is what you `git add` remote / push / bundle. The
backup clone never goes to the remote. If a PR or remote already has the
old SHAs, that host is not retconed; open a new repo or accept the leak.

## Before finishing

- [ ] Unretoned backup exists and `HEAD` was recorded
- [ ] `git log --format=fuller --all` is synthetic identity + burst dates
- [ ] Messages have no trailers and no clocks
- [ ] Tree has no bylines, home paths, LEARNINGS clocks, EXIF
- [ ] No signatures, notes, `refs/original`, or reflog
