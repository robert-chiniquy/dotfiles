---
name: editor-diff-cycle
description: After the user edits a draft in the editor, isolate their changes by section, by grain, and by whole-piece structure, generalize each cluster into a series-wide rule, and write or update a named review persona the user gave this cycle. Use when the user will polish a draft themselves, then wants those edits turned into a named persona.
---

# Editor diff cycle

The user edits. The agent does not. When they say they are done, the agent reads the diff, not the chat. Isolation may quote the source draft. The persona MUST NOT mention or reference that draft. It applies to the rest of the posts.

If the post is on the series gist, the gist file is the latest draft; pull before mining and do not treat a local `DRAFT.md` as the object (`/Users/rch/.claude/skills/rch-editor/references/series-gist-workflow.md`). A gist edit is a cycle: generalize it into the series persona.

## Common Mistakes

- Editing the draft while they are in the file. MUST NOT.
- Diffing against a dirty baseline. MUST commit or stash first so `git diff HEAD -- <file>` is only their pass.
- Mining a local unpublished `DRAFT.md` after the post is on the gist. MUST pull the gist clone. That file is the latest draft.
- One subagent on the whole file. MUST fan out by section (H2 / outline nodes) and by grain (word, sentence, paragraph, section, post).
- Fan-out that stops at H2. MUST also spawn one gestalt / holistic reviewer of the overall change of the structure of the whole piece.
- Mixing their intent with prior TONE numbers. Characterize *this* pass first, then generalize.
- Inventing a persona rule for a one-off polish that does not recur.
- Writing the persona as a recap of the source draft (path, title, SHA, that walk's nodes, named exhibits). MUST NOT. Isolation keeps the quotes. The persona names the class.
- Copying that draft's outline into the persona as the walk every post must take. Whole-piece moves become properties of a walk. Do not force every post through the same nodes.
- Writing the persona file before the grouped-rule list exists.

## Cycle

1. Pin baseline: working tree clean for the named file. If the post is on the series gist, the named file is the gist clone after `git pull --ff-only`. Open that markdown in the editor. Stop.
2. On "done": `git diff HEAD -- <file>` is the object (gist clone if published). Spawn read-only subagents:
   - one per section, each scoring small-to-large grains. Each returns a discrete list: location, before, after, grain, one-line characterization.
   - one gestalt / holistic reviewer of the whole piece. Returns: nested outline of conceptual progress before vs after (claims the reader now has, not headings); numbered whole-piece moves (what node moved, was cut, or was added in the walk); the through-line in one paragraph; remaining structural defect. Grain is the post. Not a merge of the section lists.
   No rewrite.
3. Parent merges those lists, groups by intent, writes a reusable rule per group. Apply a critical eye: name the class, not the exhibit. Whole-piece moves become properties of a walk (ethos before scene, a mechanism lives where it is the object under study, landing then closer), not a numbered recap of that draft's nodes. A rule that cannot fire on another post in the series is not reusable; ask, or drop.
4. Create or update the named persona the user gave this cycle, from those rules only. If they did not name one, ask. MUST NOT assume a persona name. Catalog it. Do not copy the whole TONE file into it. The persona MUST NOT mention or reference the source post.

## Before finishing

- Baseline SHA recorded before their edit (gist tip if published)
- Subagent lists exist per section; gestalt list exists (walk, moves, through-line, remaining structural defect)
- Rules grouped by intent, named as classes; groups that will not fire on the rest of the series were asked or dropped
- Persona exists, names no source post, contains post-grain properties not that draft's outline
