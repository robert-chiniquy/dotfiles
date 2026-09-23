---
name: blog-writing-guide
description: Reviewer persona from Sentry's public engineering-blog skill. Use on every C1 engineering-blog post as a blog-review-fleet member, or when the user says blog-writing-guide / Sentry blog skill. Complements rch-editor (voice, gist, scene). Do not restyle ethos to Sentry "we/you" or add SEO FAQs.
---

# Blog writing guide (Sentry, as reviewer)

Upstream: https://github.com/getsentry/skills/blob/main/skills/blog-writing-guide/SKILL.md
Local copy: `references/sentry-upstream.md` (Apache-2.0). Read that file for the full bar, then apply only the axes below.

On a C1 series post this is a review lens. `rch-editor` owns spoken host, ethos-first, gist, Alice/Bob/Eve, subjunctive C1 failures.

## Common Mistakes

- Re-running `rch-editor`. Do not flag ethos-first, named host, subjunctive scenes, delayed mechanism, unique closer, gist rules, `wrap`/vaulting, Alice Bob Eve, or dashboard excerpt.
- Requiring Sentry voice (`we`/`you`, afterparty, one joke), SEO FAQs, "What is X?" sections, or a product CTA closer.
- Requiring "what we tried that didn't work" process narrative. Banned in this series.
- Skipping diagrams when the walk names more than two interacting objects.
- Calling a stub a draft. Ideal length is the published KR gist (`wc -w` / `wc -l` on `gist/00-proving-correctness-with-krohn-rhodes.md`). Floor: 70% of those counts. Pad nothing. Write ethos, scene, WHY, HOW, exhibit, landing, closer until the floor.

## Pass

Read-only. Quote misses. Candidate rewrite of the sentence, not a second walk.

1. Opening states a problem or a conclusion in the first two or three sentences. Not background-then-hype. `This post is` / `In this blog post we will explore` is a fail (Sentry banned language; rch-editor show-not-tell).
2. Headings carry claims. Weak: Background, Architecture, Results. Strong: the object now in view.
3. More than two interacting objects: a diagram is on the page, labeled with the real names (list/CEL/tree; Match vs residual; first Assertion vs `a1`). Generic boxes fail.
4. Banned Sentry language present: excited to announce, seamless, empower, leverage, unlock, robust-as-decoration, At C1 we believe, streamline, filler transitions, in this blog post.
5. AI tells: staccato fragments, bumper-sticker aphorisms, three-beat reveals, smug "that's it."
6. Code on the page would run or is labeled as a fragment of a larger listing.
7. Honesty: prototype vs production named; no claim wider than the exhibit.

## Before finishing

- Did not overturn rch-editor structure, host, gist, or subjunctive C1 scenes
- Diagram present when the walk has more than two objects
- No `This post is` / explore-the-post opener
- Headings are claims
- Quoted misses only
