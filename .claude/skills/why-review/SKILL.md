---
name: why-review
description: Deep review of operational sentences that say what a thing does but not why. Use on every C1 engineering-blog post, every iteration, as a blog-review-fleet member. Complements prose-clarity (what the objects are) and rch-editor (why we analyze).
---

# Why review

Read for purpose. An engineer who sees `X does Y` must already have why Y, clearly, at that use: the after-state it protects, the question it answers, or what would go wrong if we skipped it. The why may sit earlier in this beat. It must still be in view at the verb. A why that exists only in a distant section is a miss.

Default is miss. "Clear from context" is a high bar: a reader who has only this beat, and does not already know the topic, can say why without asking. If a reviewer had to ask, it was not clear. A frame (`we refuse`, `we run`, `C1 takes preventing X seriously`) is never self-clearing.

## Common Mistakes

- Scoring tone or missing referents. Those are other personas. This persona is purpose only.
- Treating "clear from context" as the default. Default is miss.
- One why per section. Deep: every operational claim.
- Adding why to glue, exhibit labels, and listings. Those may stay as what.
- "Why" as a slogan (`this is important`). Name the after-state or the missed question.
- A category with member and no why (TONE 84). Same miss.
- `C1 refuses X` / `Here is a case we refuse` with no after-state in that sentence (reviewer would certify a Role Okta never assigned).
- A counterfactual whose actor cannot produce the effect (`Suppose the Okta AWS Federation connector would have recorded AdministratorAccess` while that connector records Okta app profiles, not wrapping XML). The cause must be true of that actor. Wrapping extra Role belongs on a first-Assertion SAML consumer.

## Pass

On a whole post: one read-only subagent per H2 (lede is a section). Each returns every operational sentence, whether why is present, and a candidate rewrite that names the after-state or the missed question.

Operational: runs, skips, folds, compiles, records, locks, evaluates, reduces, projects, wraps, overlays, analyzes, checks, refuses, serves, resumes, takes seriously, prevents.

1. List those sentences.
2. For each: is the why already clear at this use (this sentence, or still in view from the previous sentence in this beat)? After-state, missed question, or customer miss if we skipped it. Not a slogan two sections ago. When in doubt, miss.
3. Rewrite the misses. Do not invent a second walk.

## Before finishing

- Every operational sentence has a why already clear at that use
- Why names an after-state or a missed question, not a slogan
- Glue and listings were not padded
- No frame (`refuses`, `takes seriously`) without after-state in that sentence
