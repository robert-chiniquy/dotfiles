---
name: rch-editor
description: Author's voice for C1 engineering-blog prose. Use when editing a post in the series, or when the user says rch-editor.
---

# rch-editor

C1 engineering-blog prose. Dry-engineering does not govern this register. The published Krohn-Rhodes gist is the sentence-length exemplar, not a template to copy.

## Common Mistakes

- Chat notation in a post: fragments, stamp pairs, tautologies, "X is not Y", "This post is".
- A sentence a cold reader cannot answer: what is this, who did it, why is it here, how could that happen.
- Inconsistent leveling. One altitude per topic, one grain per beat. Do not explain a simple thing to a reader you also treat as expert on the deep object, and do not presume the deep object while the topic is the simple thing.
- A hypothetical with no sourced mechanism (CVE, public incident, documented protocol issue, or a named code path). Write cause, then path, then effect. C1 failures stay subjunctive. No source, no scenario.
- A greeting stamp on every post, including "Robert here" and "I'm Robert". Open on the C1 job. Named host is optional.
- `this` or `the` before one instance is on the page. Use "a" or "an individual".
- A claim about discourse, industry practice, or a vendor with no receipt in that beat. Do not invent a vendor bug.
- An unfixed C1 production defect named as a present fact.
- Sentences added to meet a word count. Length is a readout.
- A sentence copied from a skill or from a sibling post.
- A narrated trace of calls (`one job locks A then B; the other locks B then A; Unlock runs only after both Locks return`). Put the calls in a code block. Prose states only what the block cannot show: which position is speaking, and the consequence. A runtime walk is that block, a labeled stand-in, or the series Q&A. Do not improvise a narrator walking the trace. Which position speaks is the irony skill.
- Two locks with an empty critical section. The body between the second lock and the unlock has to use a real resource, a read or a write of the thing those locks protect. An empty body reads as a code error.

## Classes

This skill owns cadence, scenario honesty, and series stamps. Other classes live in one skill each: prose-clarity (unusable words, leveling, articles), why-review (what without why or how), fact-review (untrue or conflated). Owner-flagged sentences live in `/Users/rch/repo/research/equational-reasoning/_series/FAILURE_CORPUS.md`, not here.

## Before a hypothetical

Name the mechanism and its source before the scene. Then write the scene in that order.

## Before finishing

- Opener is full sentences a cold reader can follow
- No fragment used as a worked example
- No skill sentence pasted into the draft
