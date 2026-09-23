---
name: blog-review-fleet
description: Run the full C1 engineering-blog review fleet on every post, every iteration. Section titles, missing referents, why (does without why), progressive-story, information flow, rch-editor, then scale-review and humanizer. Use after any draft edit, title pass, gist push, or new post, and before handing a post for human review.
---

# Blog review fleet

Every post, every iteration. A title pass, a lede pass, or a gist push is an iteration. MUST NOT present the post until this fleet has run on that file.

## Common Mistakes

- Shipping after one persona (titles, rch-editor, humanizer) without the rest.
- One subagent told it is the whole fleet. Personas spawn separately. Parent greps bans and samples paragraphs.
- Skipping prose-clarity. Missing referents are a fleet member, not an optional extra.
- Skipping why-review. A sentence that says what X does, and not why, is a fleet miss. The why may sit earlier in the beat; it must be clear at the use. Deep: every operational claim, not one why per section.
- Skipping progressive-story. A punchline (signed-id vs first child, named ReadOnly) before its WHY/HOW chain is a fleet miss.
- One subagent on the whole file for reconstruction. prose-clarity fans out per H2.
- Skipping section-title-review after a body edit. Headings drift when the walk moves.
- A C1 failure in the indicative (happening, or already happened). Subjunctive only: may happen; we anticipate and prevent.
- Parent applying fixes, then handing over without scale-review on the changed sentences.
- Parent keeping a lede slogan after prose-clarity listed a blank definite noun (`the rearrangements`, `the last file`, `every extra Role`). That is a failed pass. Reconstructability beats magic-trick / unreachable-list placement.
- A C1 product feature named with no dashboard excerpt or flashback gif in the post.
- Editing the gist from local unpublished `DRAFT.md`. Published file is the gist clone after pull.

## Members (all, this order)

1. `section-title-review` — H1/H2/H3 cadence and progress
2. `prose-clarity` — reconstruction, missing referents, jargon already in inventory. One read-only subagent per H2 (lede is a section)
3. `why-review` — operational sentences: what X does, and why. After-state or missed question. One read-only subagent per H2. Deep: every operational claim
4. `progressive-story` — WHY then HOW then use; punchline after the causal chain; magic trick is a closed class
5. `information-flow` — conceptual-progress outline, then reorder if the outline wins
6. `rch-editor` — ethos, scene, spoken host, closer, gist rules
7. Apply those findings
8. `scale-review` on the changed sentences (every grain; a new defect class is a failed pass)
9. Humanizer (subtractive), rebuild `read.html`, open HTML

## Pass

For each live post in the iteration (gist file if published, else `DRAFT.md`):

1. Spawn members 1–6 as separate read-only subagents. MUST NOT fold them into one "you are the fleet" agent. Members 2 and 3 fan out per H2.
2. Parent merges. Apply. Do not skip a member that returned no findings; record the empty. A prose-clarity blank definite noun in the lede is applied before any slogan from progressive-story or rch-editor.
3. Parent fills the lede definite-noun table (prose-clarity step 0) in the merge, not only in a sample. Phrase, prior quote that built the object, pass/fail. Any empty prior is a failed pass until rewritten.
4. Parent greps the result for: `Friday`, `Codd`, `cool`, ` named cut`, `Git is a cool`, `wrap` (except `XML Signature wrapping`), `But trust me`, `I'm obsessed`, `a grant that lands`, `Priya`, `Devon`, `Dana,`, `Jordan`, `Here is a case we refuse.`. Any hit is a failed pass until rewritten. Ethos must name this post's C1 surface and the CS concept it introduces. `Here is a case we refuse.` fails unless the rest of that same sentence names the refused object.
5. Members 8–9 on the result.
6. Parent samples three paragraphs at random against prose-clarity, why-review, and progressive-story. If those paragraphs fail, the fleet pass failed. A sampled punchline that precedes its WHY/HOW chain is a failed pass. A sampled `the/those/every` + noun with no prior object is a failed pass.

## Before finishing

- Every live post in the iteration was a fleet subject
- Members 1–6 ran; prose-clarity and why-review were per H2; lede definite-noun table filled; missing-referent, does-without-why, and punchline-before-chain audits ran
- scale-review ran on changed sentences
- HTML opened only after that
- C1 product feature has a dashboard excerpt or flashback gif, or an author-only comment naming the blocked capture
