---
name: editor-diff-cycle
description: After the user edits a draft in the editor, isolate their changes by section, by grain, and by whole-piece structure, generalize each cluster into a reusable rule, and write or update a named review persona. Use when the user will polish a draft themselves, then wants those edits turned into rch-editor or another named persona.
---

# Editor diff cycle

The user edits. The agent does not. When they say they are done, the agent reads the diff, not the chat.

## Common Mistakes

- Editing the draft while they are in the file. MUST NOT.
- Diffing against a dirty baseline. MUST commit or stash first so `git diff HEAD -- <file>` is only their pass.
- One subagent on the whole file. MUST fan out by section (H2 / outline nodes) and by grain (word, sentence, paragraph, section, post).
- Fan-out that stops at H2. MUST also spawn one gestalt / holistic reviewer of the overall change of the structure of the whole piece.
- Mixing their intent with prior TONE numbers. Characterize *this* pass first, then generalize.
- Inventing a persona rule for a one-off polish that does not recur.
- Writing `rch-editor` before the grouped-rule list exists.

## Cycle

1. Pin baseline: working tree clean for the named file. Open that markdown in the editor. Stop.
2. On "done": `git diff HEAD -- <file>` is the object. Spawn read-only subagents:
   - one per section, each scoring small-to-large grains. Each returns a discrete list: location, before, after, grain, one-line characterization.
   - one gestalt / holistic reviewer of the whole piece. Returns: nested outline of conceptual progress before vs after (claims the reader now has, not headings); numbered whole-piece moves (what node moved, was cut, or was added in the walk); the through-line in one paragraph; remaining structural defect. Grain is the post. Not a merge of the section lists.
   No rewrite.
3. Parent merges those lists, groups by intent, writes a reusable rule per group. Whole-piece moves become persona rules at post grain, not only local cuts. Ask about a group that will not generalize.
4. Create or update the named persona (`rch-editor` unless they named another) from those rules only. Catalog it. Do not copy the whole TONE file into it.

## Before finishing

- Baseline SHA recorded before their edit
- Subagent lists exist per section
- Gestalt list exists: before/after walk, whole-piece moves, through-line, remaining structural defect
- Rules are grouped by intent; ambiguous groups were asked
- Persona file exists and contains only this cycle's generalizations, including post-grain structure
