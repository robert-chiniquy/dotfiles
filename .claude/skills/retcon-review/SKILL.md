---
name: retcon-review
description: >
  Reviewer persona for provenance leaks: author identity, clocks, trailers,
  local paths, and missing work-local retcon derivatives. Distinct from
  `retcon` (the rewrite procedure) and from `secrets-in-llm-output` (secret
  material). Use when reviewing a diff, RFC, or plan that will be shared or
  published, when a repo has `.claude/retcon.md` / work-local derivatives,
  or when the user asks for a retcon pass. Triggers: retcon review, provenance
  leak, author date, committer date, LEARNINGS timestamp, Co-Authored-By,
  share without provenance, just-written, strip clocks.
---

# Retcon review

One axis: could a recipient tell this existed before the share? Load
`$retcon` for the rewrite. This persona only reports leaks and missing
derivatives. Do not rewrite in this skill.

## Common Mistakes

1. **Global list only.** Each work set has its own retcon derivatives
   (extra names, dated formats, artifact types this work invented).
   Missing those is a failed review even if git metadata is clean.
2. **Treating citation years as clocks.** Paper years, language editions,
   protocol versions stay. Bylines and calendars go.
3. **Reviewing only the tree.** Commit messages, tags, signatures, and
   the RFC grounding block carry clocks and identity too.
4. **Demanding retcon on local-only research.** If the artifact never
   leaves the machine, this lens does not run.
5. **Fixing leaks by deleting learnings.** Keep the content; strip or
   restamp the clock.

## Derivatives

Look in, and merge, all that exist:

- `<repo>/.claude/retcon.md` or `RETCON.md`
- `plans/<topic>/retcon.md` (RFC-scoped)
- a `## Retcon` section in the RFC or project instructions

Those files are additive leak surfaces, not a replacement for `$retcon`.
A share with no derivatives file is a finding: write one (even "this
work adds none").

## Hunt

Diff, commit messages (`git log --format=fuller` / `%B`), and RFC text:

- author/committer names and emails
- author/committer dates outside a now-burst
- `Co-Authored-By`, `Signed-off-by`, `Generated with`, other trailers
- `gpgsig` / signed tags
- LEARNINGS `## YYYY-MM-DD HH:MM`, "Locked YYYY-MM-DD", ISO-8601 in
  prose, `Date:` headers, copyright year ranges
- `/Users/…`, `/home/…`, hostnames, tracker ids if the dest publishes
- work-local derivative strings not yet listed

Structured files: parse, do not regex.

## When to include

Add this persona to the review/RFC roster when any of:

- the repo or `plans/<topic>/` has retcon derivatives
- the user asked for retcon

Not a default lane on every internal PR.

## Output

```
<file>:<line> — LEAK | MISSING-DERIVATIVE | KEEP
<what leaks or what derivative is absent>
<concrete strip or file to add>
```

Cap ~15. Do not edit the branch.

## Before finishing

- [ ] Work-local derivatives loaded or flagged missing?
- [ ] Messages and signatures checked, not only the tree?
- [ ] Citation years not stripped?
