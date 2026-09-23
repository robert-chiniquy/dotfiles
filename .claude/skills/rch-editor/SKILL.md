---
name: rch-editor
description: Author's voice for C1 engineering-blog prose. Use when editing a post in the series, or when the user says rch-editor.
---

# rch-editor

Ethos before the scene. One walk; end at the landing, then one closer. Named host is a series fact, not a lede stamp.

The unifying goal is unique, interesting, meaningful content. Repetition is the miss. A sentence, aside, closer, ethos dump, or scene frame that already appeared in a sibling post is not content.

Another way to think about the series: teach a magic trick for something the reader thought was unreachable (list every rearrangement, every lock order, every CEL path). The trick is a closed representation. MUST NOT dump the mechanism in the lede. MUST NOT name that list with nouns the walk has not built (`The rearrangements have no last file, so refusing every extra Role looks unreachable.`). Name the topic. Teach WHY each named value is true, then HOW the surprising split is possible from design issues, then the use (`progressive-story`). The unreachable list lands when a file is on the page.

Published posts live in the series gist; the gist file is the latest draft. The gist and `_series/` workflow (pull before edit, do not overwrite from a local `DRAFT.md`, image URLs, `00-` file order) is `/Users/rch/.claude/skills/rch-editor/references/series-gist-workflow.md`. A gist edit is a persona cycle: generalize it into this file.

## Common Mistakes

- Opening on the failure. Who is speaking, what C1 is, and which jobs this post sits in come first. Then the scene.
- Copying another post's ethos. The greeting and one C1 sentence may repeat. After that: this post's C1 surface and the CS concept it will introduce, not a roster of identity records, access reviews, and a support path.
- Inventing first names for the people in a scene (Priya, Devon). Alice, Bob, Eve are the security personas, and only for people. MUST NOT name a queue worker, consumer, job, shard, or other software component Alice, Bob, or Eve. Name the component (`the owner`, `the stale consumer`, `AssignedTo`). Name the job.
- Naming connectors (or grants, wraps, sessions) because a sibling post did. If this post is not about ingest, downloads, or grant-applying jobs, connectors stay off the page.
- Copying another post's Imagine-if. The scene is the failure mode this topic prevents, not a weekend deadlock unless this post is about lock-order. MUST NOT open every scene with `Imagine if`.
- Stamping `Hi! My name is Robert`, `I'm Robert`, or `Robert here` on every post. MUST NOT `Robert here`. Named host appears in the series, not in every lede. Prefer opening on C1's job in ordinary language. First person is one option among several; I-language is not the default.
- Three technical nouns in one sentence with no prior job (`Session policies attach CEL to a live login`). Williams old-before-new: a sentence introduces at most one technical noun the previous sentence did not already make usable. First-use reconstructability beats concision; later-use concision beats re-gloss. Terseness is not concision. Worked fail: `Match evals that expression on this request's ctx`. Worked fail: `C1 keeps a login` (actor and object unrecoverable). Worked pass: After a person signs in, later requests still have to count as that person. Then CEL. Then session policy. Then Match as the run on an individual request.
- `this` / `the` on a class member that has not been singled out. Prefer `a` / `an individual` until one instance is on the page.
- Inconsistent leveling. Anyone who can sail a boat can tie their shoes: do not explain a simple thing to a reader you also treat as a domain expert, and do not presume a deep subdomain while the topic is the simple thing. One altitude for the topic. Same for granularity: a beat that defines JSON-RPC and then uses least-model, or that teaches what a login is and then uses `Match`, has two altitudes. Worked fail: `After a person signs in, later HTTP requests still have to count as that person` in a post whose reader already ships session policy. Worked pass: a C1 session policy is CEL run on each later request; `Match` is that run.
- A claim about discourse, industry practice, or a vendor bug with no receipt in that beat. Link the post, spec, or table first. MUST NOT invent a Cloudflare missed-`if`. Their published 2,594-tool listing is a size measurement.
- Linking [How complex systems fail](https://how.complexsystems.fail/) in every post. Cite that page at most once in the series. Later posts may say the class is latent without the URL, or skip that sentence when the scene already shows the class.
- A product or job noun in the scene before ethos has named that job.
- Parking a later mechanism in the lede. Resume, durability, representation, and cousins live where they are the object under study.
- An operational sentence that says what X does and not why (`We run static analysis on that Go` with no after-state). why-review.
- `This post is ____`. Show the object. Do not announce the post.
- The refrain `C1 takes preventing those incidents seriously` (and cousins: shipping is fast, every commit is more software, agentic development made listing harder). Once in the series is already too often if it is copied. Unique why we analyze, in this post's objects.
- A weird metaphor for a product verb (`Dynamic groups write who is in Engineering`). Use the product's verb (names, lists, decides membership). Stretch metaphor stays `Think of X as "Y"` for a named structure, not an invented action.
- `X is not Y` as a definition (`SSA is not event sourcing`). Name each object and the job that uses it. Cross-domain comparison (SSA from compilers, git gc from version control) must say why that domain is on the page in the same beat.
- `Back to the example above.` The reader cannot say which example. Name the object (`Alice's Wednesday delete`, `Okta's signed Assertion a1`).
- A C1 product name in the ethos with no two-to-four sentences of what that surface is for, from c1.ai docs or a public C1 post, before the CS concept. Why this surface is on the page has to be obvious (Jobs to be Done) before the math word. Worked fail: agent memory, then classifier, with no deposited notes or who they open for. A gateway, catalog, or other architecture noun without the value of that architecture (what the customer admin gets: agent does not inherit every GitHub route the human could click) is the same miss.
- A tautology: a sentence that restates the definition of its subject (`A fact is held true`). Cut it. The definition already did that work.
- Two short stamp sentences in a row (`Those are two questions. Grants answers who.`). The sentence equivalent of repetitive spondees. Fold them into one sentence that carries the architecture or the after-state.
- Adjacent sentences that share one subject and one job, left unconsolidated. Reviewers try one sentence first.
- `tenant` in public copy. Internal word. Say customer, customer admin, user, admin, or customer id.
- `dump` as a noun for a listing or inventory (HTTP dump, wire dump). Displeasing. Say listing, inventory, OpenAPI walk, `tools/list`. MUST NOT use displeasing words; public copy wants euphony.
- A public protocol (MCP, SAML, CEL) named without the public discourse it already has: spec link, what the protocol actually structures, and the conversation at scale (MCP: `tools/list` is a bag of names; Code Mode / code execution with MCP is the published response to tool-list blowup). Speak in the light of the public discourse.
- Academic citations on a background language (Datalog papers) that dilute the works the reader should care about (Krohn-Rhodes). One link to introduce Datalog. Balance is a writing priority.
- Adding sentences to meet a word-count floor. Length is a publication threshold. MUST NOT bulk out. Write the walk; if it is short, it is unpublished, not padded.
- A CS term (`classifier`, `residual`, `Datalog`) in ethos before the C1 job is in view. Name the job, then the term when membership of a set appears.
- Disclaiming unless the owner asked (`This surface is a prototype`, `Production still evals`, `they do not compile`). Write the experiment as the object. Do not volunteer shipping status.
- Session policies (or any sibling surface) named without the two-to-four sentence product context, or named in ethos when this post's object is a different surface.
- Operational semantics (Winskel, ⟨c, σ⟩) appearing first in a late H2. Name the command-and-store step in the opener if the post will use it.
- A counterfactual only as `Suppose` prose when a Q/A beat would make the after-state and the repair consecutive. Format:
  Q: What if a deploy failed?
  A: Then the session key would still be live.
  Q: How does the store still have a present?
  A: Last-write of the log, tombstone included.
- Invented product wording (`connector draft`) when docs already name Functions drafts or connector versions.
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
- Repeating `C1 takes preventing those incidents seriously` / `especially now with how fast we ship` / `Every commit is more software to verify` / `Agentic development made listing harder` from a sibling. Unique sentence, this post's object.
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
- `Suppose two runtime files in one customer's connector draft would share a name and differ in content` with no job that put two puts on one name. The draft keys files by name, so a replacement is a second put of the same key. That job belongs in the Suppose.
- `The rearrangements have no last file, so refusing every extra Role looks unreachable.` Definite nouns with no object yet. prose-clarity step 0. Reconstructability beats the magic-trick slogan.
- `Here is a case we refuse.` as its own sentence. `a case` has no object. The next sentence is not the referent. Name the refused object in that sentence.
- A scene counterfactual whose actor cannot produce the effect. `Suppose the Okta AWS Federation connector would have recorded AdministratorAccess` has no WHY: that connector records assignment profiles. Wrapping extra Role is a first-Assertion SAML consumer.
- Reusing a scene stamp across posts (`case we refuse`, `Imagine if`, `Here is a case`). Each scene opener is unique. MUST NOT paste `case we refuse` into a second post.
- Combined "you are the whole fleet" subagent. Personas spawn separately. Parent greps the bans. Parent fills the lede definite-noun table.
- Treating the KR 70% word floor as a writing target. Ideal is KR `wc -w`. Floor is a publication threshold. MUST NOT add sentences to meet it. A short walk stays short and unpublished.
- Publicly naming an as-yet-unfixed defect in C1 production code. Subjunctive anticipated failures and constructed exhibits (PairLedger, a table) are the public form. Unfixed production holes stay in conversation with the author and in the tracker. MUST NOT put a live unfixed C1 path in the gist or a live `DRAFT.md`.

## Whole piece

The post is one walk. Score these properties before local cuts. Do not force every post through the same nodes.

1. Ethos is node one: this post's C1 job in ordinary language first. Product names and CEL/Match/`ctx` wait until that job is old information. Named host is optional in this lede. MUST NOT dump identity-records / access-reviews / support-path unless those are this post's objects. 24x7 only if those jobs run that way and this post is about that run.
2. If the scene is an incident: one customer, the failure this topic may produce, which we anticipate and prevent. Opener varies. The verbs stay subjunctive (`would`, `might`, `were`). MUST NOT narrate a C1 outage as past fact (`Monday a reviewer still listed Alice`).
3. Why now is this topic's combinations. Shipping and agentic development only if this post is about analysis volume. MUST NOT paste "open-source connectors" into a post that is not about connectors.
4. Name the traditional method as a category (what it is, what a member is, why it exists), then C1's exhibit in the language the shop actually uses.
5. A job or mechanism lives in the beat where it is the object under study.
6. Hold at the turn, then the result this post is for.
7. Name Classifiers at the membership joint.
8. After the result lands, a cousin that already did the categorical version of this move is allowed. Then one concluding paragraph unique to this post: this topic's after-states, in this post's words. MUST NOT paste `Correctness in the security domain is everything: a grant that lands, a review of all current state, a removed privilege that does not come back` or any other closer from a sibling. Nothing in the series is repeated verbatim.

MUST NOT open on the scene. MUST NOT park a later mechanism in ethos. MUST NOT start a second essay after the landing.

## Spoken host

Named host is a series fact. Greeting varies; many posts open on C1 with no first person in the lede. MUST NOT `Robert here`. MUST NOT stamp `I'm Robert` on every post. First person, when used, is named and not the first three words.

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
6. When a subsection resumes an earlier exhibit, name the object (`Alice's Wednesday delete`, `Okta's signed Assertion a1`). MUST NOT write `Back to the example above`.
7. When a decomposition lets you skip enumeration, the remaining parts no longer *need* to be listed. Obligation, not a completed skip.
8. Closer: security domain, everything. After-states, not avoided failures. Real C1 SaaS codebases, any scale. Not one application. Not an enumeration bound.
9. Title is a keyword pair people can pick: one CS concept and one security / SaaS / reliability / product concept, in many posts, not all (a stamp of that pair is the same miss). Series H1s introduce concepts in publish order. A later title uses a concept an earlier title named. MUST NOT re-introduce `a set` after classifiers already named it. MUST NOT stamp every H1 as `X is Y`. Role mining belongs in the exact-comparison title. Vaulting belongs in the weakest-precondition title. Two clocks as a product is a fold candidate into vaulting or session product, not a required standalone. The title may name the running example (`Detecting deadlocks with Krohn-Rhodes`). MUST NOT use `(and more!)`.
10. Section titles are a walk in the post title's register. They progress: what is in view, or how much of the program is in the table. A numbered sequence keeps one template (`One operator`, `One observed run`, `One function's table`). Two headings for one node collapse. MUST NOT mix a concept-name H2 (`Acquisition order`) with a claim H2 in the same post.
11. C1 failures: may happen; we anticipate and prevent. Subjunctive. Not underway. Not already done. MUST NOT name an as-yet-unfixed defect that is present in C1 production. If the analysis found a live hole, surface it to the author; public copy uses the class and a constructed exhibit.
12. When a C1 product feature is named, at least once per post include a screenshot excerpt of that feature in the C1 dashboard, or a flashback replay gif if the action is a live session rather than a page. If the walk names more than two interacting objects (list/CEL/tree; Match vs residual; first Assertion vs signed id), include a diagram in the body (Sentry blog-writing-guide: diagrams for systems). Crop to the feature, not a full-window dump. Caption names the feature. MUST NOT invent UI. If capture is blocked, an author-only HTML comment names the feature and screenshot vs flashback; the public body does not get a placeholder. Gist figures use gistusercontent raw URLs.
13. A fictive scene is one blockquote. Fact stays outside. The quote names the refused object and why those values are true. Do not mix the Suppose into a paragraph of product fact. (`C1 refuses a Grants row that would name AdministratorAccess for that assignment, so an access reviewer cannot certify a Role Okta never assigned.`). WHY includes why the named values are true, not only the after-state of the refuse. Worked fail: `Suppose two runtime files in one customer's connector draft would share a name and differ in content` (no job that put two puts on one name). A replacement is a second put of the same key because the draft keys files by name; that job belongs in the Suppose. A `would have recorded` / `would have listed` sentence names a cause that is true of that actor. MUST NOT `Here is a case we refuse.` MUST NOT reuse `case we refuse` in any post. Each scene opener is unique. Name the job (Jobs to be Done). People in the exhibit are Alice, Bob, Eve. Software is named as the component (`the owner`, `the stale consumer`, `AssignedTo`). MUST NOT give a worker, job, or consumer a human first name. MUST NOT invent other first names (Priya, Devon, Dana, Jordan). The calendar day is the next day after the event for that job. MUST NOT reuse Friday across the series.
14. Cite only what the reader needs at that sentence. MUST NOT mention Codd. MUST NOT call Git a cool library. MUST NOT praise tools everyone already has. `cool` is rare; once in the series is enough.
15. Do not say `cut` for a named time or history prefix. Say time, epoch, as-of, or the writes up to that time.
16. A thin post folds into the post that already owns the object (crash/tombstone and two maps on the identity store belong with last-write history). Session product and vaulting two clocks are different objects; MUST NOT mix them in one lede.
17. Before keeping a retitle, compare it to the original heading. If the new title dropped the words that still name the topic (compaction, time, history, residuation), the retitle lost. Self-critical; no epicycles.
18. Functions draft: public docs name Save draft, Test draft, Publish (`https://www.c1.ai/docs/product/admin/functions-create`). MUST NOT say connector draft. If the object is Functions, ethos names Functions.
19. A wrapping scenario is a story. First: WHY the honest Role is on the signed Assertion (Okta assignment → profile → AWS Role attribute → IdP signs those statements). Then: HOW an extra Role is possible, from SAML/XML design, one issue at a time (enveloped signature inside the payload; C14N of a tree the signer is modifying; `URI=#id` names a subtree, not the first child; the schema allows siblings and `Advice`; finding the Signature already parses the file). Then Eve's unsigned first Assertion. MUST NOT dump `signature covers a named subtree by id; a consumer that reads the first Assertion can see a different Role` in the lede. MUST NOT put wrapping extra Role on the Okta AWS Federation connector: that connector records assignment profiles, not login XML. A first-Assertion SAML consumer is the actor that would record AdministratorAccess. Authorization uses authenticated data.
20. When a post has a source RFC or planning corpus, the draft uses that corpus. A title that names the product job while the body stays a thin set-algebra lecture is a miss.
21. Magic trick: the unreachable list (XSW1–XSW8 has no last file) then a closed class that makes the extra value unrepresentable. Enumeration is the foil, not the landing.
22. A check written as membership in a closed class is a static artifact with two jobs: an implementation you can verify, and an introspectable spec other implementations can be compared against. State both at the closed-class landing, in two sentences, without `not only / but`.

## Before finishing

- Ethos before scene; jobs are this post's; scene is this topic's failure mode; greeting and scene opener are not the series stamp; C1 failures stay subjunctive
- One walk; no unearned instrument; no second essay after the landing
- Classifiers named if membership of program objects appears
- Closer unique to this post; no series-verbatim aside; no `wrap`; no sentence starting with a backtick'd noun
- Ethos opens on the C1 job; greeting is not a series stamp; outline not announced; published post matches gist after pull; section titles progress
- Lede known-new: each technical noun was usable from the previous sentence; first-use reconstructability, later-use concision
- One C1-feature dashboard excerpt or flashback gif, or an author-only comment naming the blocked capture
- Magic trick named without dumping the mechanism; no lede definite noun whose object is a later H2; WHY then HOW then use; closed-class artifact has both jobs (check, spec)
- Alice, Bob, Eve only name people; software uses component names
- Did not bulk out to meet the KR word floor; length is a publication threshold
- No tautology; no two-stamp spondee pair; no `tenant`; no listing-as-`dump`; public protocol named with its spec and discourse
