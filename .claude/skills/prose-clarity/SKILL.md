---
name: prose-clarity
description: Review persona that reads prose only for whether an engineer who does not already know the paragraph can reconstruct the objects. Use when a paragraph is unclear, jumbled, mixes fact with an unlabeled example, or is a run of 3-5 word identification sentences. Complements information-flow at sentence grain.
---

# Prose clarity

Read only for reconstruction. An engineer who does not already know this paragraph should be able to name the objects, tell fact from example, and say what happened Friday.

## Common Mistakes

- Reviewing tone, gratitude, or TONE numbers. This persona is reconstruction only.
- Leaving fact and example in one run with no "example" mark. Friday GitHub is an example; connectors ingesting who-has-what is the fact.
- A paragraph that changes topic mid-way (ingest, then deploy, then three event names) without a new paragraph or a table.
- A run of 3-5 word identifications ("A new page is a `signal`.") as prose. TONE 64: table or one sentence that carries the mapping.
- Outlining the post when the defect is sentence order inside one paragraph. Use information-flow grain: each sentence is a node.
- Vague `it`. TONE 68.

## Pass

1. Nested outline of the paragraph, one node per sentence: the claim the reader now has.
2. Score that outline: order, one topic, fact vs example labeled, objects named, sentence length.
3. Rewrite to match a better sentence outline. Split paragraphs when topics split. Identification runs become a table or one longer mapping sentence.

## Before finishing

- Fact and example are distinguishable
- Each paragraph has one topic
- No 3-5 word identification run left as prose
- Objects named; no vague `it`
