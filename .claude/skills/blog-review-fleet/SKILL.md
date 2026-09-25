---
name: blog-review-fleet
description: Run the full C1 engineering-blog review fleet on every post, every iteration. Section titles, missing referents, why, progressive-story, information flow, rch-editor, poetic-meter, irony, ambient-research, research-voice, fact-review, blog-writing-guide, then meta-reviewer after those edits, then scale-review and humanizer. Use after any draft edit, title pass, gist push, or new post, and before handing a post for human review.
---

# Blog review fleet

Every post, every iteration. Live posts come from `INDEX.md` (unpublished `DRAFT.md` plus the published KR gist). Folded directories are archives; do not fleet `DRAFT_FOLDED.md`. Freeze a content hash of each live file before dispatch. Every reviewer returns that hash. If the draft changes, rerun affected members on the new hash. “Ran” means every required member returned for that same hash, every high-impact finding has a disposition (`fixed` / `already answered` / `false positive` / `unresolved`), and changed sentences passed scale-review. MUST NOT present HTML as reviewed while any field is open. A mid-work preview is labeled incomplete.

## Hard rule

Personas run independently of the parent. The parent does not perform a member's review, and does not report the parent's own reading as that persona's review. Dispatch the persona, wait for its output, and apply or reject that output. A sample the parent reads in the persona's place is not a pass. Step 7 below does not authorize that substitution.

The outer loop is `blog-iteration`. This fleet is one step inside it. A `TRIVIALITY.md` verdict of reject does not enter the polish members. The writer is `blog-writer`, and it is not a member of this list.

## Common Mistakes

- Shipping after one persona (titles, rch-editor, humanizer) without the rest. An owner request for an irony trial is the exception: run only `irony`, and label the HTML as that trial.
- Treating irony as sarcasm, or as a second ethos pass. It chooses the speaking position and the form of a runtime beat. An irony intent that omits the skill's prior failure modes is an incomplete direction.
- Treating poetic-meter as a request to write verse, or as a second pass of irony or rch-editor. It hears sentence stress. A short sentence is a variation. It fails when the next sentence is the same variation.
- Treating ambient research as fact-review, or as a hunt for something topical to insert. It reports what has changed around the subject and the smallest edit that serves intent, or no change.
- Treating meta-reviewer as another general critic, or rolling back a legitimate improvement. It runs after the other reviewers are applied. It repairs material drift from the subject that made the post worth writing.
- One subagent told it is the whole fleet. Personas spawn separately. Parent greps bans and samples paragraphs.
- The parent performing a persona's review and reporting it as that persona. Dispatch the persona. Wait for its file. A reading the parent did is not that review.
- Opening a reviewer file or a draft markdown file in VS Code as part of the revision. Those files stay in the background. The reading copy, when one is handed over, is the HTML. Open a working markdown file only when the message is directing the user to something specific in it.
- A first run of one persona that edits no post. Zero edits across the set is a failed pass, not a clean pass. Change that persona's instructions before trusting the run. A later run may still preserve a draft, and that preservation has to name the sentence it refused to change.
- A code example that takes two locks and then does nothing before the unlock. The critical section has to use a real resource. An empty body reads as a code error.
- A mentioned function, HTTP verb, or URL path left in running text. `Bob calls get` names `GET`. The mention goes in backticks. Do not backtick an ordinary English verb.
- Treating the current draft as the thing to conserve. Prior drafts are in git. The meta-reviewer may come back with a very substantial revision. A fresh take is always worth it.
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
7. `poetic-meter` — sentence stress, variation, continuity, and voice. Modeled on Fussell's *Poetic Meter and Poetic Form*. Not verse. A repeated sentence shape is the defect. Does not own facts or speaking position.
8. `occult-theory` — bring forward a static reading of a whole aggregate, and the open space between systems, ontologies, and categories. The wayfinder stays in the skill. Grothendieck is the model and is not named in the draft. The skill does not name a vendor. Do not give that reading to the product with a direct verb. Do not name the private research system. No disclaimer sentence. A new formal term in the beat (`empty`) gets one sentence of what it is, in objects already on the page. Iterate. More than one point edit while each is a net improvement. Stop when the next edit would not help.
9. `irony` — which position is speaking, and whether a runtime beat is a code block, a labeled stand-in, or a Q&A. Not sarcasm. A new formal term with no sentence is a speaker who already has the term. Do not cut the sentence that first gives it. Intent directions include the prior failure modes in the skill. When the owner asks for an irony trial, run only this member.
10. `ambient-research` — the field around the text, and the local research index that feeds every draft. Not fact-checking and not trend chasing. The index pass does not edit a draft. Also marks each object of attention as well-known, specialist-known, contested, novel, or unknown, and each formal term the draft uses without a sentence. An unexplained term is not "no change." That classification is a signal for focus. It is not a new citation. Every C1 draft's scope includes https://c1.ai as the most authoritative source for what C1 says. Audience references, not citations: https://blog.trailofbits.com/ and https://trailofbits.com/buttercup/. Dispatcher supplies TEXT, INTENT, SCOPE, and RESEARCH WINDOW. Smallest edit, or no change. Do not add a current reference unless that intent is served.
11. `research-voice` — claim-size vs the check on the page; query direction; encoded HOW as discovery; residual fail-closed. Does not overturn ethos/scene
12. `fact-review` — actor and effect are true of the product. Connector vs SAML consumer, grant list vs login XML. Does not score WHY or referents
13. `triviality-rejector` — the result is not a textbook or Wikipedia example wearing product nouns. A concrete listing is not a result. A known method is allowed when the page reports something the reference does not contain, in the product's solution space. Does not invent that something. Does not edit the draft. Writes `TRIVIALITY.md`.
14. `blog-writing-guide` — Sentry public skill as reviewer: diagrams for >2 objects, no `This post is`, headings as claims, banned launch language. Does not restyle ethos. Upstream https://github.com/getsentry/skills/blob/main/skills/blog-writing-guide/SKILL.md
15. Apply those findings
16. `meta-reviewer` — drift governor on the applied text. Compare it to the earliest defensible draft and the subject that made the post worth writing. If the post has a source RFC, read that RFC or the review missed the point. A series pass starts from `/Users/rch/repo/research/equational-reasoning/_series/outline_V16.md` and writes `_series/META_SERIES.md`. It does not edit drafts on that pass. A fresh take is allowed on a single post. Prior drafts are in git. Not a general critic.
17. `series-review` — once per iteration, across the live posts. Order, continuity, progression, related topics, common elements, allocation. Reads `outline_V16.md`. Writes `_series/SERIES_REVIEW.md`. Does not edit drafts on the finding pass.
18. `scale-review` on the changed sentences (every grain; a new defect class is a failed pass)
19. Humanizer (subtractive), rebuild `read.html`, open HTML

## Pass

For each live post in the iteration (gist file if published, else `DRAFT.md`):

1. Spawn members 1–14 as separate read-only subagents. MUST NOT fold them into one "you are the fleet" agent. Members 2 and 3 fan out per H2. `ambient-research` may search; it still returns findings, not a rewrite, unless INTERVENTION is edit. An irony trial spawns only `irony` and skips this list. `series-review` is not in this per-post spawn. `triviality-rejector` writes `TRIVIALITY.md` and does not edit the draft.
2. Parent merges. Apply. Do not skip a member that returned no findings; record the empty. A prose-clarity blank definite noun in the lede is applied before any slogan from progressive-story or rch-editor.
3. Parent fills the lede definite-noun table (prose-clarity step 0) in the merge, not only in a sample. Phrase, prior quote that built the object, pass/fail. Any empty prior is a failed pass until rewritten.
4. Parent greps the result for: `Friday`, `Codd`, `cool`, ` named cut`, `Git is a cool`, `wrap` (except `XML Signature wrapping`), `But trust me`, `I'm obsessed`, `a grant that lands`, `Priya`, `Devon`, `Dana,`, `Jordan`, `case we refuse`, `Here is a case we refuse`, `This post is `, `C1 takes preventing`, `Agentic development made`, `Every commit is more`, `Back to`, ` is not `, `This surface is a prototype`, `they do not compile`, `tenant`, `\bdump\b` as a listing noun, `Robert here`. ` is not ` that defines by denial (`SSA is not event sourcing`) is a failed pass. Keep a contrast only when it corrects a belief already on the page. Any hit is a failed pass until rewritten (KR may keep series-once asides). Ethos must name this post's C1 job in ordinary language first, then the surface, then the CS concept. A sampled lede sentence with three new technical nouns is a failed pass. A sampled beat that explains a simple thing (what a login is) while using a deep object (`Match`, least-model) is inconsistent leveling: a failed pass. One altitude per beat. A sampled `this`/`the` + noun with no unique instance yet is a failed pass (`this request`, `the public conversation`). A sampled social claim with no cited receipt in that beat is a failed pass. A sampled vendor bug that is not in a public record is a failed pass. `Robert here` is a failed pass. `I'm Robert` on every live post is a failed pass (variety; some ledes open on C1 with no first person). A `refuses` / `takes preventing` sentence with no after-state in that sentence is a why-review fail. A sampled `can` with no HOW is a why-review fail. Two short stamp sentences in a row are a failed pass. A tautology is a failed pass. A public protocol without spec and discourse is a failed pass.
4b. Parent HEADs every `en.wikipedia.org` URL in the live file. 404 is a fail. A `#fragment` that is not a heading on that article is a fail even when the page is 200 (`Crash_(computing)#Crash-only_software`).
4c. Parent `wc -w`s `gist/00-proving-correctness-with-krohn-rhodes.md` and each live `DRAFT.md`. Below 70% of KR words is unpublished, not a reason to pad. Ideal is KR. MUST NOT bulk out.
5. Parent runs `jscpd` on live `DRAFT.md` files as plain text (markdown tokenizer only sees fences). `jscpd --pattern '**/DRAFT.md' --formats-exts 'txt:md' --format txt --min-lines 2 --min-tokens 12 --ignore '**/gist/**' --reporters console --no-tips --absolute <series-root>`. Default `--min-tokens 50` misses a 4-word stamp. A clone that is a scene opener, closer, or ethos dump across two live posts is a failed pass. HTML comments and folded successor copies of the same epic may stay. Install: `cargo install jscpd` or `brew install jscpd`.
6. Run `meta-reviewer` on the applied text, including after an irony trial. Once per iteration, run `series-review` and the meta-reviewer's series pass from `outline_V16.md`. Then members 18–19. Skip 18–19 on an irony trial.
7. Do not sample in the parent's voice as a stand-in for a persona. Members 2, 3, 4, 7, 9, 10, 11, and 12 return their own findings. If a returned finding names a failed sentence, the fleet pass failed.

## Before finishing

- Every live post in the iteration was a fleet subject
- Members 1–14 ran, unless the owner asked for an irony trial, in which case only `irony` ran; `meta-reviewer` ran on the applied text either way; `series-review` and the meta-reviewer's series pass ran once; prose-clarity and why-review were per H2; lede definite-noun table filled; missing-referent, does-without-why, punchline-before-chain, claim-vs-check, actor-vs-effect, and triviality audits ran
- scale-review ran on changed sentences, unless this was an irony trial
- HTML opened only after that
- C1 product feature has a dashboard excerpt or flashback gif, or an author-only comment naming the blocked capture
