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
- A referent that does not name both sides ("makes a comparison"; "this picture"; "those standards"). TONE 80. The engineer must be able to say what is compared to what, in the sentence that uses the word.
- Halted rhythm: three or more consecutive same-shape short sentences. TONE 81.
- "Instead of A and B": two avoided outcomes in one contrast. TONE 82.
- One defect class named as the whole analysis. TONE 83.
- A category word (`model checker`, `model`) used without saying what the category is, what a member is, and why the category exists. TONE 84.
- `` Name is the set `` on first use of a coined name. TONE 85: `` `Legal` means ``.
- A comparison to a generic noun ("a board"). TONE 86: name chess, and how the squares relate.
- Agentless passive ("is told"). TONE 87.
- `this class of` / `that product` / `those cases` with no named object in the previous sentence. TONE 88. Audit: walk every demonstrative + category noun.
- Generic role-pairs (`authors and operators`) with no names, no job, and no act. TONE 89.

## Pass

On one paragraph: outline each sentence, score, rewrite.

On a whole post: spawn one read-only subagent per H2 (lede is its own section). Each returns a sentence outline, issues, and a candidate rewrite of that section. Parent merges. Do not have the parent outline the sections.

1. Nested outline, one node per sentence: the claim the reader now has.
2. Score that outline: order, one topic, fact vs example labeled, objects named, sentence length.
3. Rewrite to match a better sentence outline. Split paragraphs when topics split. Identification runs become a table or one longer mapping sentence.
4. Audit every `this`/`that`/`those` + noun, every `is told` / `is asked`, every `like a [generic noun]`, and every first-use `Name is the set`. Each must resolve.

## Before finishing

- Fact and example are distinguishable
- Each paragraph has one topic
- No 3-5 word identification run left as prose
- Objects named; no vague `it`
- Every comparison names both sides in that sentence or the one before
- No halted same-shape run of three short sentences
- One contrast per sentence; defect classes not collapsed to one name
- Category words named as category, member, and why before the instance
- Demonstratives, passives, and `like a [generic]` each resolve
