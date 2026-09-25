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
- A scene opener that names objects with no WHY those values exist. Worked fail: `Suppose two runtime files in one customer's connector draft would share a name and differ in content.` The reader cannot say why two files, why the same name, why different content. A replacement is a second put of the same key because the draft keys files by name. Name that job in the Suppose sentence (or the sentence before). After-state of the refuse (`serve would keep the last file`) does not excuse a blank WHY on the named values. Same miss as `The signed Assertion named ReadOnly` with no assignment.
- `this request has not filled source_ip yet` with no job that left the field empty (proxy did not copy an IP; Match fills empty strings). Same miss: `If a type error were swallowed as false` with no expression that type-checks fail (`source_ip == 5`). Same miss: `A SAML consumer can record AdministratorAccess` with no HOW (enveloped signature, `URI=#id`, first child). `can` is not a mechanism. Name the design issue or a public CVE, then the extra value.
- A declarative claim with no example and no HOW (`An imperative gateway that forgot an if still has the route in the listing`). Name a published measurement or class, then the mechanism.
- An operational result whose consequence is left for the reader to invent. Worked fail: `A job that does not finish leaves that list without that update.` The missing update is not the consequence. Say what the next person would do with that list (a reviewer would certify grants that download never delivered). Subjunctive when it has not happened.

## Pass

On a whole post: one read-only subagent per H2 (lede is a section). Compact issue records only. Do not list every operational sentence. Do not rewrite a pass.

Operational: runs, skips, folds, compiles, records, locks, evaluates, reduces, projects, wraps, overlays, analyzes, checks, refuses, serves, resumes, takes seriously, prevents. Scene openers (`Suppose`, the refused object) count.

For each miss, one record:

```
line:
operation:
reader cannot answer:
nearby sentence that may already answer: (quote or "none")
verdict: miss
smallest edit:
```

Deduplicate the same unanswered question within a beat. Glue, exhibit labels, listings: skip. No formulaic `so` padding. A candidate edit only on a miss. Parent dispositions: `fixed` / `already answered (quote)` / `false positive (reason)` / `unresolved`. Rejecting a rewrite does not close the finding.

High-impact only for the parent gate: scene values without a job; `can` without HOW; actor cannot produce the effect; refuse/prevent with no after-state; this post's C1 surface with no reason it is on the page (agent memory with no deposited notes); architecture named without value prop; public protocol named without spec and discourse.

## Before finishing

- Misses are compact records, not a padded rewrite of every verb
- Why names an after-state or a missed question, not a slogan
- Glue and listings were not padded
- No frame (`refuses`, `takes seriously`) without after-state in that sentence
- No Suppose / scene opener that names values without why those values are true
- Duplicate questions in one beat collapsed to one record
