---
name: blog-review-fleet
description: Run the full C1 engineering-blog review fleet on every post, every iteration. Section titles, missing referents, why (does without why), information flow, rch-editor, then scale-review and humanizer. Use after any draft edit, title pass, gist push, or new post, and before handing a post for human review.
---

# Blog review fleet

Every post, every iteration. A title pass, a lede pass, or a gist push is an iteration. MUST NOT present the post until this fleet has run on that file.

## Common Mistakes

- Shipping after one persona (titles, rch-editor, humanizer) without the rest.
- Skipping prose-clarity. Missing referents are a fleet member, not an optional extra.
- Skipping why-review. A sentence that says what X does, and not why, is a fleet miss. The why may sit earlier in the beat; it must be clear at the use. Deep: every operational claim, not one why per section.
- One subagent on the whole file for reconstruction. prose-clarity fans out per H2.
- Skipping section-title-review after a body edit. Headings drift when the walk moves.
- A C1 failure in the indicative (happening, or already happened). Subjunctive only: may happen; we anticipate and prevent.
- Parent applying fixes, then handing over without scale-review on the changed sentences.
- A C1 product feature named with no dashboard excerpt or flashback gif in the post.
- Editing the gist from local unpublished `DRAFT.md`. Published file is the gist clone after pull.

## Members (all, this order)

1. `section-title-review` — H1/H2/H3 cadence and progress
2. `prose-clarity` — reconstruction, missing referents, jargon already in inventory. One read-only subagent per H2 (lede is a section)
3. `why-review` — operational sentences: what X does, and why. After-state or missed question. One read-only subagent per H2. Deep: every operational claim
4. `information-flow` — conceptual-progress outline, then reorder if the outline wins
5. `rch-editor` — ethos, scene, spoken host, closer, gist rules
6. Apply those findings
7. `scale-review` on the changed sentences (every grain; a new defect class is a failed pass)
8. Humanizer (subtractive), rebuild `read.html`, open HTML

## Pass

For each live post in the iteration (gist file if published, else `DRAFT.md`):

1. Spawn members 1–5 as read-only subagents. Members 2 and 3 fan out per H2.
2. Parent merges. Apply. Do not skip a member that returned no findings; record the empty.
3. Members 7–8 on the result.

## Before finishing

- Every live post in the iteration was a fleet subject
- Members 1–5 ran; prose-clarity and why-review were per H2; missing-referent and does-without-why audits ran
- scale-review ran on changed sentences
- HTML opened only after that
- C1 product feature has a dashboard excerpt or flashback gif, or an author-only comment naming the blocked capture
