---
name: rch-editor
description: Author's voice for C1 engineering-blog prose. Use when editing a post in the series, or when the user says rch-editor.
---

# rch-editor

Spoken host is the standing register. Ethos before the scene. One walk; end at the landing, then one closer.

The unifying goal is unique, interesting, meaningful content. Repetition is the miss. A sentence, aside, closer, ethos dump, or scene frame that already appeared in a sibling post is not content.

Another way to think about the series: teach a magic trick for something the reader thought was unreachable (list every rearrangement, every lock order, every CEL path). The trick is a closed representation. MUST NOT dump the mechanism in the lede. Name the topic and the unreachable list. Teach WHY each named value is true, then HOW the surprising split is possible from design issues, then the use (`progressive-story`).

Published posts live in the series gist (clone `/Users/rch/repo/research/equational-reasoning/gist/`, https://gist.github.com/robert-chiniquy/aa2ef88faf0eac08b4a1f86cc6ad8027). A file linked from that `OUTLINE.md` is the latest draft of that post. MUST `git pull --ff-only` before editing it. MUST NOT overwrite it from a local unpublished `DRAFT.md`. A gist edit is a persona cycle: generalize it into this file. Gist markdown images MUST use `https://gist.githubusercontent.com/<user>/<id>/raw/<file>` with no commit SHA. A relative `![...](file.png)` is rewritten to a `#file-` fragment and does not render. GitHub may inject a SHA into the *rendered* `img src`; that tracks the viewed revision. MUST NOT copy that href back into the markdown. Published `.md` files MUST sort first (`00-` prefix). Gist lists files alphabetically; a PNG as the first file suppresses the markdown preview.

## Common Mistakes

- Opening on the failure. Who is speaking, what C1 is, and which jobs this post sits in come first. Then the scene.
- Copying another post's ethos. The greeting and one C1 sentence may repeat. After that: this post's C1 surface and the CS concept it will introduce, not a roster of identity records, access reviews, and a support path.
- Inventing first names for the people in a scene (Priya, Devon). Alice, Bob, Eve are the security personas. Name the job.
- Naming connectors (or grants, wraps, sessions) because a sibling post did. If this post is not about ingest, downloads, or grant-applying jobs, connectors stay off the page.
- Copying another post's Imagine-if. The scene is the failure mode this topic prevents, not a weekend deadlock unless this post is about lock-order. MUST NOT open every scene with `Imagine if`.
- Stamping `Hi! My name is Robert` and the full C1 platform sentence on every post. First person, named, in ethos. The greeting varies. The platform one-liner belongs once in the series, not in every lede.
- Linking [How complex systems fail](https://how.complexsystems.fail/) in every post. Cite that page at most once in the series. Later posts may say the class is latent without the URL, or skip that sentence when the scene already shows the class.
- A product or job noun in the scene before ethos has named that job.
- Parking a later mechanism in the lede. Resume, durability, representation, and cousins live where they are the object under study.
- An operational sentence that says what X does and not why (`We run static analysis on that Go` with no after-state). why-review.
- Contrasting a language the exhibit is not written in. The listing is the object.
- Stopping the walk to define an instrument the next node does not use (a second Wikipedia link, a homonym aside, a Q&A that recodes an operation already in view).
- Slogans that a picture of a structure cannot model a program. Name the design blanks (omitted cells from purpose or design) and the consequence (decomposition is hard).
- Hiding Classifiers. When program objects become membership in a set, name C1's set model.
- Past tense for a living theorem or analysis. The year of a result stays past.
- One model of the source. Many models; several questions of one body. The analysis factors the model a semantics presents, not a unique table of the file.
- A C1 failure as a present fact, or as something that definitely happened. C1 failures are subjunctive: they may happen. We prevent them by anticipating. MUST NOT be happening. MUST NOT have happened. A listing on the page (PairLedger, a table) may run in the indicative; C1 production does not. The opener of that scene varies (`Suppose`, a would-unfold Friday, the object gone wrong). Not `Imagine if` as the series default.
- A second essay after the result has landed (other checkers, other writings of the same cut). End at the cousin that already did this categorically, then one closer. If there is no cousin, the landing is the result itself, then the closer.
- Closing on a sibling's closer. This post's after-states, this post's words. Nothing verbatim anywhere in the series.
- Repeating `But trust me, this is all going somewhere` or `I'm sorry, I'm obsessed with this topic` after it has already appeared in the series.
- The word `wrap` (vaulted secrets). Say vaulting.
- A sentence that begins with a backtick'd noun.
- Naming a C1 product feature with no dashboard excerpt (or flashback gif) anywhere in the post.
- A review that is merely not frozen. The review is of all current state.
- A privilege that does not come back, unnamed. It is a removed privilege that does not come back.
- One C1 application. C1 is SaaS codebases.
- Overwriting a published gist file from the local unpublished tail. The gist is the latest draft.
- Polishing an unread tail after the author asked to stop. That cut is the intended end.
- Citation baggage (series number, press, year) next to a title the reader can click. Keep the year when it is the result.
- Mixing a concept-name H2 with a claim H2 in the same post. Section titles progress. A numbered sequence keeps one template. Two headings for one node collapse.
- Reusing Friday. Recycled ethos. Unlabeled fictive scene. `cut` as a named time. Codd. Git as a cool library. Gratuitous citation. `cool` as filler.
- Mixing session-policy product with vaulting clocks in one post.
- A retitle that dropped the topic words the original still needed.
- A wrapping exhibit that dumps the signed-id vs first-child split before teaching HOW that split is possible from SAML/XML design (enveloped signature, C14N, `URI=#id` vs document order, schema siblings).
- `The signed Assertion named ReadOnly` with no WHY (Okta assignment → profile attribute → AWS Role attribute → IdP signs those statements).
- Combined "you are the whole fleet" subagent. Personas spawn separately. Parent greps the bans.

## Whole piece

The post is one walk. Score these properties before local cuts. Do not force every post through the same nodes.

1. Ethos is node one: name, C1 in one sentence, then this post's C1 surface together with the CS or math concept the post will introduce (Access Profiles and emptiness of a difference; session policies and residuation; dynamic groups and unfinished CEL). MUST NOT dump identity-records / access-reviews / support-path unless those are this post's objects. 24x7 only if those jobs run that way and this post is about that run.
2. If the scene is an incident: one customer, the failure this topic may produce, which we anticipate and prevent. Opener varies. The verbs stay subjunctive (`would`, `might`, `were`). MUST NOT narrate a C1 outage as past fact (`Monday a reviewer still listed Alice`).
3. Why now is this topic's combinations. Shipping and agentic development only if this post is about analysis volume. MUST NOT paste "open-source connectors" into a post that is not about connectors.
4. Name the traditional method as a category (what it is, what a member is, why it exists), then C1's exhibit in the language the shop actually uses.
5. A job or mechanism lives in the beat where it is the object under study.
6. Hold at the turn, then the result this post is for.
7. Name Classifiers at the membership joint.
8. After the result lands, a cousin that already did the categorical version of this move is allowed. Then one concluding paragraph unique to this post: this topic's after-states, in this post's words. MUST NOT paste `Correctness in the security domain is everything: a grant that lands, a review of all current state, a removed privilege that does not come back` or any other closer from a sibling. Nothing in the series is repeated verbatim.

MUST NOT open on the scene. MUST NOT park a later mechanism in ethos. MUST NOT start a second essay after the landing.

## Spoken host (standing)

First person, named, somewhere in ethos. Greeting varies. MUST NOT open every post with `Hi! My name is Robert.`

Asides (`But trust me, this is all going somewhere`, `I'm sorry, I'm obsessed with this topic`, `Awesome right?`, `Believable.`) are each allowed at most once in the series. MUST NOT paste any of them into a second post. A hold at a turn is still allowed in other words.

MUST NOT begin a sentence with a backtick'd noun. Write the job in English, then the token.

`wrap` is forbidden in every post. Vaulting is vaulting: vault, vaulted secret, epoch, member snapshot, catch-up, open. MUST NOT use wrap, unwrap, or wrapping for that path. XML Signature wrapping is a different attack; name that attack in full, not `wrap`.

Hold at a turn. MUST NOT announce the outline (`this section covers`).

## Local rules

1. Cite author and title. Drop series-number and press-year unless the year is the result.
2. Walk an exhibit in English (from/to) before or as you show it; mark it ordinary.
3. Illegal or excluded cases: the combinations a test might already name.
4. Stretch metaphor: `Think of X as "Y"`, not `X is Y`.
5. Present tense for what the analysis still does. Past tense for the year of a result.
6. When a subsection resumes an earlier exhibit, name the resume (`Back to the example above`).
7. When a decomposition lets you skip enumeration, the remaining parts no longer *need* to be listed. Obligation, not a completed skip.
8. Closer: security domain, everything. After-states, not avoided failures. Real C1 SaaS codebases, any scale. Not one application. Not an enumeration bound.
9. Title is a keyword pair people can pick: one CS concept and one security / SaaS / reliability / product concept, in many posts, not all (a stamp of that pair is the same miss). Series H1s introduce concepts in publish order. A later title uses a concept an earlier title named. MUST NOT re-introduce `a set` after classifiers already named it. MUST NOT stamp every H1 as `X is Y`. Role mining belongs in the exact-comparison title. Vaulting belongs in the weakest-precondition title. Two clocks as a product is a fold candidate into vaulting or session product, not a required standalone. The title may name the running example (`Detecting deadlocks with Krohn-Rhodes`). MUST NOT use `(and more!)`.
10. Section titles are a walk in the post title's register. They progress: what is in view, or how much of the program is in the table. A numbered sequence keeps one template (`One operator`, `One observed run`, `One function's table`). Two headings for one node collapse. MUST NOT mix a concept-name H2 (`Acquisition order`) with a claim H2 in the same post.
11. C1 failures: may happen; we anticipate and prevent. Subjunctive. Not underway. Not already done.
12. When a C1 product feature is named, at least once per post include a screenshot excerpt of that feature in the C1 dashboard, or a flashback replay gif if the action is a live session rather than a page. Crop to the feature, not a full-window dump. Caption names the feature. MUST NOT invent UI. If capture is blocked, an author-only HTML comment names the feature and screenshot vs flashback; the public body does not get a placeholder. Gist figures use gistusercontent raw URLs.
13. A fictive scene is labeled as a case we refuse, not a gear-shift into past-tense report. Name the job (Jobs to be Done). The people in the exhibit are the usual security personas: Alice, Bob, Eve. MUST NOT invent other first names (Priya, Devon, Dana, Jordan). The calendar day is the next day after the event for that job. MUST NOT reuse Friday across the series. MUST NOT jump into the scene without a rhetorical frame.
14. Cite only what the reader needs at that sentence. MUST NOT mention Codd. MUST NOT call Git a cool library. MUST NOT praise tools everyone already has. `cool` is rare; once in the series is enough.
15. Do not say `cut` for a named time or history prefix. Say time, epoch, as-of, or the writes up to that time.
16. A thin post folds into the post that already owns the object (crash/tombstone and two maps on the identity store belong with last-write history). Session product and vaulting two clocks are different objects; MUST NOT mix them in one lede.
17. Before keeping a retitle, compare it to the original heading. If the new title dropped the words that still name the topic (compaction, time, history, residuation), the retitle lost. Self-critical; no epicycles.
18. Connector draft: the bundle C1 will serve for one connector version. Runtime file: one named file in that bundle (script, schema, policy). Define both before any scene that uses them.
19. A wrapping scenario is a story. First: WHY the honest Role is on the signed Assertion (Okta assignment → profile → AWS Role attribute → IdP signs those statements). Then: HOW an extra Role is possible, from SAML/XML design, one issue at a time (enveloped signature inside the payload; C14N of a tree the signer is modifying; `URI=#id` names a subtree, not the first child; the schema allows siblings and `Advice`; finding the Signature already parses the file). Then Eve's unsigned first Assertion. MUST NOT dump `signature covers a named subtree by id; a consumer that reads the first Assertion can see a different Role` in the lede. Authorization uses authenticated data.
20. When a post has a source RFC or planning corpus, the draft uses that corpus. A title that names the product job while the body stays a thin set-algebra lecture is a miss.
21. Magic trick: the unreachable list (XSW1–XSW8 has no last file) then a closed class that makes the extra value unrepresentable. Enumeration is the foil, not the landing.
22. A check written as membership in a closed class is a static artifact with two jobs: an implementation you can verify, and an introspectable spec other implementations can be compared against. State both at the closed-class landing, in two sentences, without `not only / but`.

## Before finishing

- Ethos before scene; jobs are this post's; scene is this topic's failure mode; greeting and scene opener are not the series stamp; C1 failures stay subjunctive
- One walk; no unearned instrument; no second essay after the landing
- Classifiers named if membership of program objects appears
- Closer unique to this post; no series-verbatim aside; no `wrap`; no sentence starting with a backtick'd noun
- Spoken host present; outline not announced; published post matches gist after pull; section titles progress
- One C1-feature dashboard excerpt or flashback gif, or an author-only comment naming the blocked capture
- Magic trick named without dumping the mechanism; WHY then HOW then use; closed-class artifact has both jobs (check, spec)
