---
name: terms-pass
description: One-at-a-time TERMS.md gloss pass for a multi-post series. Compile unclear words into one series TERMS.md, ask via the question tool, record each decision, do not edit drafts until remaining terms are picked. Use when the user says terms pass, remaining terms, TERMS.md, compile unclear words, gloss before first use, ask me the terms, or a draft has specialist words that need a first-use sentence.
---

# Terms pass

One list of unclear words for a whole series. The user picks each word. Drafts wait until remaining OPEN terms are picked, or the user says apply.

Canonical file is the series `_series/TERMS.md`, one per series, never a per-post `TERMS.md` (the `_series/` convention: `/Users/rch/.claude/skills/rch-editor/references/series-gist-workflow.md`). Current series: `/Users/rch/repo/research/equational-reasoning/_series/TERMS.md`.

Question protocol is `questioning-the-user`. Metaphor / named-group / failure-mode rules live in that series `TONE.md` (53–55 here). Do not copy them into TERMS.

## Common Mistakes

- Minting `NN-name/TERMS.md`. Dual-reading an old post TERMS and the series file.
- Several terms in one question-tool call, or a prose list of remaining terms after the user asked to be asked.
- Editing any `DRAFT.md` while OPEN terms remain, unless the user said apply.
- A question with no quoted sentence from a live `DRAFT.md`.
- Applying Recommended after the user declined that question. Decline leaves the term OPEN.
- Re-asking a DONE term. Renumbering. Inventing a definition; used-as is how the draft leans on the word.
- Asking a term that already has a conventional default, or that the draft already glosses.

## Compile

Scan live `DRAFT.md` files (body copy; skip Status: skeleton / `(empty)`). For each unclear word not already in TERMS, append the next unused number:

```
N. word — used-as from the draft
```

Numbers never change. A hole is a removed item, not a reuse. DONE terms are not re-asked. A word that appears in several posts is still one row.

## Ask

One OPEN term per question-tool call. Upstream-first when a later term depends on an earlier pick.

The question names the term number, the word, the full absolute TERMS.md path, and a quoted sentence from a live `DRAFT.md`. 2–4 options, Recommended first with the reason in the description. Do not add Other.

`code --reuse-window --goto` the TERMS.md when the user is deciding.

Record in TERMS.md before the next question:

```
N. word — DONE: <decision in one or two sentences>
```

or, on decline:

```
N. word — OPEN: declined this turn. Leave as-is until picked. <what still sits in the draft>
```

Later questions adapt: drop terms that became moot, reshape options that changed.

## Apply

When no OPEN remain, or the user said apply: restate the new decisions compactly, then edit every live `DRAFT.md` that uses those words. Do not re-confirm.

A DONE decision is series-wide. `classifier` decided on one post is that word in every live post.

## Before finishing

- TERMS.md is only `_series/TERMS.md`
- OPEN terms were not silently applied
- Drafts untouched if OPEN remain, unless the user said apply
- TERMS.md opened in the editor while the user was picking
