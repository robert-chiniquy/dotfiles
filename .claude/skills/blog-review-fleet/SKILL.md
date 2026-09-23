---
name: blog-review-fleet
description: Run the full C1 engineering-blog review fleet on every post, every iteration. Section titles, missing referents, why, progressive-story, information flow, rch-editor, research-voice, fact-review, blog-writing-guide, then scale-review and humanizer. Use after any draft edit, title pass, gist push, or new post, and before handing a post for human review.
---

# Blog review fleet

Every post, every iteration. Live posts come from `INDEX.md` (unpublished `DRAFT.md` plus the published KR gist). Folded directories are archives; do not fleet `DRAFT_FOLDED.md`. Freeze a content hash of each live file before dispatch. Every reviewer returns that hash. If the draft changes, rerun affected members on the new hash. “Ran” means every required member returned for that same hash, every high-impact finding has a disposition (`fixed` / `already answered` / `false positive` / `unresolved`), and changed sentences passed scale-review. MUST NOT present HTML as reviewed while any field is open. A mid-work preview is labeled incomplete.

## Common Mistakes

- Shipping after one persona (titles, rch-editor, humanizer) without the rest.
- One subagent told it is the whole fleet. Personas spawn separately. Parent greps bans and samples paragraphs.
- Skipping prose-clarity. Missing referents are a fleet member, not an optional extra.
- Skipping why-review. A sentence that says what X does, and not why, is a fleet miss. The why may sit earlier in the beat; it must be clear at the use. Deep: every operational claim, not one why per section.
- Skipping progressive-story. A punchline (signed-id vs first child, named ReadOnly) before its WHY/HOW chain is a fleet miss.
- Skipping research-voice. Claim wider than the check, encoded HOW as discovery, inverted query, residual as empty, missing exhibit, launch CTA, numbers without exclusions, or judge-as-landing is a fleet miss. Do not re-run rch-editor under this persona.
- Skipping fact-review. An actor that cannot produce the named effect (connector records wrapping extra Role) is a fleet miss. Do not score WHY or referents here.
- Public copy that names an as-yet-unfixed C1 production defect. Surface that privately. Public form is the class plus a constructed exhibit.
- One subagent on the whole file for reconstruction. prose-clarity fans out per H2.
- Skipping section-title-review after a body edit. Headings drift when the walk moves.
- A C1 failure in the indicative (happening, or already happened). Subjunctive only: may happen; we anticipate and prevent.
- Parent applying fixes, then handing over without scale-review on the changed sentences.
- Parent keeping a lede slogan after prose-clarity listed a blank definite noun (`the rearrangements`, `the last file`, `every extra Role`). That is a failed pass. Reconstructability beats magic-trick / unreachable-list placement.
- A C1 product feature named with no dashboard excerpt or flashback gif in the post.
- Editing the gist from local unpublished `DRAFT.md`. Published file is the gist clone after pull.
- Presenting a stub as a draft. Parent `wc -w`s the published KR gist, then each live `DRAFT.md`. Below 70% of KR is a publication fail, not a writing fail. MUST NOT add sentences to meet it. Identify missing causal beats or fold; never pad.
- Treating `DRAFT_FOLDED.md` or a local KR copy as a live post. Live list is `INDEX.md` unpublished successors plus the gist KR file.
- Owner negative fixtures the fleet must still flag: `the rearrangements have no last file`, `Here is a case we refuse`, connector records wrapping extra Role, two files share a name with no second put, `source_ip` empty with no hop, `This post is`, `C1 takes preventing those incidents seriously`.
- Reviews without a gate on the final version: findings dismissed in bulk, reviewers reading older text, HTML presented before dispositions. Freeze a hash, every member returns that hash, every high-impact finding has `fixed` / `already answered` / `false positive` / `unresolved`, then HTML. A Safari open during work is a reading copy. The footer must say review incomplete until dispositions match the hash.
- `Back to` as a recap. Name the object.
- `SSA is not event sourcing` and other `X is not Y` definitions.
- A C1 feature named without product context (what the surface is for, from c1.ai or a public C1 post) before the CS concept.
- Why this C1 surface is on the page is not obvious in ethos.
- A CS term (`classifier`) before the C1 job.
- A disclaimer the owner did not ask for (`This surface is a prototype`, `Production still evals`, `they do not compile`).
- Session policies named in another post's ethos without product context.

## Members (all, this order)

1. `section-title-review` — H1/H2/H3 cadence and progress
2. `prose-clarity` — reconstruction, missing referents, jargon already in inventory. One read-only subagent per H2 (lede is a section)
3. `why-review` — operational sentences: what X does, and why. After-state or missed question. One read-only subagent per H2. Deep: every operational claim
4. `progressive-story` — WHY then HOW then use; punchline after the causal chain; magic trick is a closed class
5. `information-flow` — conceptual-progress outline, then reorder if the outline wins
6. `rch-editor` — ethos, scene, spoken host, closer, gist rules
7. `research-voice` — claim-size vs the check on the page; query direction; encoded HOW as discovery; residual fail-closed. Does not overturn ethos/scene
8. `fact-review` — actor and effect are true of the product. Connector vs SAML consumer, grant list vs login XML. Does not score WHY or referents
9. `blog-writing-guide` — Sentry public skill as reviewer: diagrams for >2 objects, no `This post is`, headings as claims, banned launch language. Does not restyle ethos. Upstream https://github.com/getsentry/skills/blob/main/skills/blog-writing-guide/SKILL.md
10. Apply those findings
11. `scale-review` on the changed sentences (every grain; a new defect class is a failed pass)
12. Humanizer (subtractive), rebuild `read.html`, open HTML

## Pass

For each live post in the iteration (gist file if published, else `DRAFT.md`):

1. Spawn members 1–9 as separate read-only subagents. MUST NOT fold them into one "you are the fleet" agent. Members 2 and 3 fan out per H2.
2. Parent merges. Apply. Do not skip a member that returned no findings; record the empty. A prose-clarity blank definite noun in the lede is applied before any slogan from progressive-story or rch-editor.
3. Parent fills the lede definite-noun table (prose-clarity step 0) in the merge, not only in a sample. Phrase, prior quote that built the object, pass/fail. Any empty prior is a failed pass until rewritten.
4. Parent greps the result for: `Friday`, `Codd`, `cool`, ` named cut`, `Git is a cool`, `wrap` (except `XML Signature wrapping`), `But trust me`, `I'm obsessed`, `a grant that lands`, `Priya`, `Devon`, `Dana,`, `Jordan`, `case we refuse`, `Here is a case we refuse`, `This post is `, `C1 takes preventing`, `Agentic development made`, `Every commit is more`, `Back to`, ` is not `, `This surface is a prototype`, `they do not compile`, `tenant`, `\bdump\b` as a listing noun, `Robert here`. ` is not ` that defines by denial (`SSA is not event sourcing`) is a failed pass. Keep a contrast only when it corrects a belief already on the page. Any hit is a failed pass until rewritten (KR may keep series-once asides). Ethos must name this post's C1 job in ordinary language first, then the surface, then the CS concept. A sampled lede sentence with three new technical nouns is a failed pass. A sampled beat that explains a simple thing (what a login is) while using a deep object (`Match`, least-model) is inconsistent leveling: a failed pass. One altitude per beat. A sampled `this`/`the` + noun with no unique instance yet is a failed pass (`this request`, `the public conversation`). A sampled social claim with no cited receipt in that beat is a failed pass. A sampled vendor bug that is not in a public record is a failed pass. `Robert here` is a failed pass. `I'm Robert` on every live post is a failed pass (variety; some ledes open on C1 with no first person). A `refuses` / `takes preventing` sentence with no after-state in that sentence is a why-review fail. A sampled `can` with no HOW is a why-review fail. Two short stamp sentences in a row are a failed pass. A tautology is a failed pass. A public protocol without spec and discourse is a failed pass.
4b. Parent HEADs every `en.wikipedia.org` URL in the live file. 404 is a fail. A `#fragment` that is not a heading on that article is a fail even when the page is 200 (`Crash_(computing)#Crash-only_software`).
4c. Parent `wc -w`s `gist/00-proving-correctness-with-krohn-rhodes.md` and each live `DRAFT.md`. Below 70% of KR words is unpublished, not a reason to pad. Ideal is KR. MUST NOT bulk out.
5. Parent runs `jscpd` on live `DRAFT.md` files as plain text (markdown tokenizer only sees fences). `jscpd --pattern '**/DRAFT.md' --formats-exts 'txt:md' --format txt --min-lines 2 --min-tokens 12 --ignore '**/gist/**' --reporters console --no-tips --absolute <series-root>`. Default `--min-tokens 50` misses a 4-word stamp. A clone that is a scene opener, closer, or ethos dump across two live posts is a failed pass. HTML comments and folded successor copies of the same epic may stay. Install: `cargo install jscpd` or `brew install jscpd`.
6. Members 11–12 on the result.
7. Parent samples three paragraphs at random against prose-clarity, why-review, progressive-story, research-voice, fact-review, and blog-writing-guide. If those paragraphs fail, the fleet pass failed. A sampled punchline that precedes its WHY/HOW chain is a failed pass. A sampled `the/those/every` + noun with no prior object is a failed pass. A sampled claim wider than the exhibit on the page is a failed pass. A sampled actor that cannot produce the named effect is a failed pass. A sampled `Suppose` that names objects without why those values exist (two files sharing a name; ReadOnly on the Assertion) is a failed pass. Include the scene opener in the sample set every time.

## Before finishing

- Every live post in the iteration was a fleet subject
- Members 1–9 ran; prose-clarity and why-review were per H2; lede definite-noun table filled; missing-referent, does-without-why, punchline-before-chain, claim-vs-check, and actor-vs-effect audits ran
- scale-review ran on changed sentences
- HTML opened only after that
- C1 product feature has a dashboard excerpt or flashback gif, or an author-only comment naming the blocked capture
