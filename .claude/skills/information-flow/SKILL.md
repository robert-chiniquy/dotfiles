---
name: information-flow
description: Proof-read large-scale information flow. A subagent outlines conceptual progress (topics and themes, not headings). The parent reviews that outline for effective, coherent, consistent, complete structure, then reorders the post to match a better outline. Use when the user asks for information flow, conceptual progress, nested outline of a post, whether the sections are in the right order, or this editorial pass.
---

# Information flow

Proof-reading for whether the reader meets each claim in an order that earns the next. Headings are a symptom. The outline is the object.

## Common Mistakes

- Outlining H2 titles. The outline is conceptual progress: after this beat, the reader has this claim.
- Parent writing the outline, then reviewing it. MUST spawn a subagent for the outline. Parent only reviews.
- Reordering sentences inside a beat when the defect is section order.
- Splitting an exhibit from the claim it supports.
- Adding a claim, a name, or a second concept to paper over a hole (TONE 21). Reorder only.
- Undoing an owner-locked order (exhibit before any reference; Carter diagrams as the first group picture; scene then C1 object then mechanism) unless the user is revisiting that lock.
- Shipping a new outline without rewriting the post to match, or rewriting the post without writing the outline first.
- Skipping ethos. Author named, one sentence of the C1 job this post sits in, is a node. An outline that jumps from the scene to `2^n` without that node is incomplete.
- Treating a product as a pile of named surfaces. C1 is one picture (who has access to what). Later names are parts of that picture.
- A list of components with mixed stature (connectors next to session `match_cel` next to C1's Go) with no job on each line. TONE 69.
- Vague `it` for a technical object. TONE 68. The outline names the object.
- "This is part of a series" with no following job. TONE 70. Not a node.
- Identifying a code defect with a product-picture defect. TONE 71.
- Treating a regular coloring or group picture as a full program. TONE 72.
- `machine` for an abstract table. TONE 73.
- A C1 practice with no why. TONE 74.

## Outline (subagent)

Spawn a read-only subagent. Brief: nested outline of conceptual progress in the named draft. Each node is a claim the reader now has, in the order they get it. Depth 2-3. Not a table of contents. If a heading and a claim diverge, note it in one line under that node. MUST include, when present or missing: ethos (author named); the product as one object; each component list with job and stature. Do not edit files. Do not recommend a reorder.

## Review (parent)

Score the outline, not the headings, on four axes:

1. Effective: each node is used by a later node, or it is the close.
2. Coherent: one through-line. A second object that appears, vanishes, then returns is a break. The product stays one object.
3. Consistent: the same object keeps the same name and grain. No `it` where the outline should name the object.
4. Complete: a later use was introduced; an exhibit is on the page before a reference; the author is named; the product is one object; each component list shares stature and names a product job per item.

If the current order already wins on those axes, stop. Write the verdict. Do not shuffle for novelty.

If a better order exists, write the new nested outline, then move whole beats of the draft to match. Keep each beat's internal walk. Rebuild `read.html` when the series uses it. Open the HTML, not the markdown.

## Before finishing

- Outline was produced by a subagent
- Review names the four axes, including ethos and component-list stature
- Post order matches the chosen outline, or the verdict was keep
- Owner-locked orders still hold unless the user reopened them
