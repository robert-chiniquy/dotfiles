---
name: blog-review-fleet
description: Run the full C1 engineering-blog review fleet on every post, every iteration. Missing referents, information flow, rch-editor, section titles, then scale-review and humanizer. Use after any draft edit, title pass, gist push, or new post, and before handing a post for human review.
---

# Blog review fleet

Every post, every iteration. A title pass, a lede pass, or a gist push is an iteration. MUST NOT present the post until this fleet has run on that file.

## Common Mistakes

- Shipping after one persona (titles, rch-editor, humanizer) without the rest.
- Skipping prose-clarity. Missing referents are a fleet member, not an optional extra.
- One subagent on the whole file for reconstruction. prose-clarity fans out per H2.
- Skipping section-title-review after a body edit. Headings drift when the walk moves.
- Parent applying fixes, then handing over without scale-review on the changed sentences.
- Editing the gist from local unpublished `DRAFT.md`. Published file is the gist clone after pull.

## Members (all, this order)

1. `section-title-review` — H1/H2/H3 cadence and progress
2. `prose-clarity` — reconstruction, missing referents, jargon already in inventory. One read-only subagent per H2 (lede is a section)
3. `information-flow` — conceptual-progress outline, then reorder if the outline wins
4. `rch-editor` — ethos, scene, spoken host, closer, gist rules
5. Apply those findings
6. `scale-review` on the changed sentences (every grain; a new defect class is a failed pass)
7. Humanizer (subtractive), rebuild `read.html`, open HTML

## Pass

For each live post in the iteration (gist file if published, else `DRAFT.md`):

1. Spawn members 1–4 as read-only subagents. Member 2 fans out per H2.
2. Parent merges. Apply. Do not skip a member that returned no findings; record the empty.
3. Members 6–7 on the result.

## Before finishing

- Every live post in the iteration was a fleet subject
- Members 1–4 ran; prose-clarity was per H2; missing-referent audit ran
- scale-review ran on changed sentences
- HTML opened only after that
