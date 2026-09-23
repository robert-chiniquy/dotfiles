---
name: talks-diff-cycle
description: After the user edits a talk deck, isolate their changes by cluster and whole-deck walk, generalize into reusable rules, and propose rch-editor Talks patches for confirmation. Use when they polish slides or yaml themselves, then want those edits turned into standing talk rules.
---

# Talks diff cycle

Same job as `editor-diff-cycle`. The object is a deck, not a gist post. No HTML, no gist pull.

The user edits. The agent does not. On "done", the agent reads `git diff`, not the chat. Isolation may quote this pass. The persona MUST NOT mention this deck, these slide files, or this yaml. Proposed rules go to the user first. MUST NOT write the persona until they confirm.

Standing persona for this cycle is `rch-editor`, Talks heading. Talks rules MUST NOT fire on blog posts. Blog rules MUST NOT fire on decks.

## Common Mistakes

- Editing slides, yaml, or OUTLINE while they are in those files. MUST NOT.
- Diffing a dirty tree. MUST commit or stash first so `git diff HEAD` is only their pass.
- Mining `OUTLINE_V#` / versioned snapshots. The live `OUTLINE.md`, live deck yaml, and live slide files are the object.
- Treating one markdown file as the whole pass when they also reordered yaml. The yaml order is a change.
- One subagent on the whole deck. MUST fan out by topic cluster (yaml spans) and by grain (token, caption, slide, cluster, deck).
- Fan-out that stops at clusters. MUST also spawn one gestalt reviewer of the whole-deck walk (reorder, cuts, inserts, flashback placement).
- Writing persona rules before a grouped-rule list exists, or writing them before confirmation.
- Inventing a rule for a one-off polish that will not fire on the next talk.
- Copying SPIRIT or TONE into the persona. SPIRIT stays the talk's standing constraints. The persona is revision class.
- Putting gist / HTML / dashboard-screenshot rules on a deck cycle.
- Naming this talk's exhibits (flashback marks, mix-table rows, a named still) in the persona. Isolation keeps the quotes. The persona names the class.

## Cycle

1. Pin baseline: working tree clean for the deck files they will touch (yaml, slides, live OUTLINE). Open the live yaml or the slide they will edit. Stop.
2. On "done": `git diff HEAD --` those files is the object. Spawn read-only subagents:
   - one per yaml cluster, each scoring small-to-large grains. Each returns a discrete list: location, before, after, grain, one-line characterization.
   - one gestalt / holistic reviewer of the whole deck. Returns: nested outline of conceptual progress before vs after (claims the audience now has, not slide titles); numbered whole-deck moves (what node moved, was cut, or was added in the walk); the through-line in one paragraph; remaining structural defect. Grain is the deck. Not a merge of the cluster lists.
   No rewrite.
3. Parent merges those lists, groups by intent, writes a reusable rule per group. Name the class, not the exhibit. Whole-deck moves become properties of a walk (connect-back before illustration, a mechanism lives where it is the object under study), not a recap of this yaml. A rule that cannot fire on another talk is not reusable; ask, or drop. Ideas, not sample sentences.
4. Show the grouped rules to the user. Wait for confirmation (keep, drop, reword). Then patch `rch-editor` Talks from the confirmed rules only. Catalog if new. MUST NOT copy the whole SPIRIT file into it. The persona MUST NOT mention this deck.
5. Later slide work in this and later talks MUST follow the confirmed Talks rules.

## Before finishing

- Baseline SHA recorded before their edit
- Cluster lists exist; gestalt list exists (walk, moves, through-line, remaining structural defect)
- Rules grouped by intent, named as classes; one-offs asked or dropped
- Grouped rules shown; confirmed subset written to rch-editor Talks; no source-deck names in the persona
