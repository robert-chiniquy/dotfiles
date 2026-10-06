---
name: sprint
description: >
  Pack the largest bin of next items that can be done together without the
  operator, preview one prompt, and on approval execute that prompt to
  completion unless a named stop needs the user. An item is one acceptance.
  A block is that bin: items that fit one context window and make each
  other cheaper. A heavy item is its own block. A test pinned at a present
  form stays there until the engine reaches a more reduced form. Do other
  executable items. Ask only after a survey shows none of them is
  executable without the operator. Do not invent the more reduced form.
  Ground the plan in the sprint-ground document the project contract
  names. Its remaining work is the plan. Then read the latest
  spine-agent report, correlate it to the tracker, and treat an
  incomplete correlated item as a block candidate.
  Triggers: sprint, maximal block, next adjacent work, prompt to finish
  autonomously, "what can you run to completion", approval of a sprint
  preview ("do that", "run it", "approved"). Not scramble (short parallel
  push). Not project-process (design, plan, RFC). Not a status recap.
---

# Sprint

An item is one acceptance. A block is the largest bin of items that can be done now without the operator, that fit one context window, and that make each other cheaper. A heavy item is its own block. Preview one prompt. On approval, execute that prompt in this session. Do not ask the user to paste it.

Requirements live in the sections below. Common mistakes only names a wrong move. Do not copy a requirement into that list, and do not park a project law there. A project law lives in that project's contract.

## Common mistakes

- Packing from the tracker before the sprint-ground document's remaining work and the latest spine-agent report. Identify.
- Packing the project goal, a ready queue, a file's neighbors, or the sentences of one task as several items. Identify.
- Handing the user the preview, or executing a prompt the user just changed. Approval.
- Treating this skill as scramble or as project-process.
- A prose menu of questions, or a question while an executable item remains. When a question blocks the block.
- A second mechanism, an invented more-reduced form, a pin move without approval, or two host forms left without the account in Reduction.
- Implementing a semantics the theory does not state, or ending on a green re-run of checks already recorded. Prompt and Reply.
- A report that is a path, a sentence that costs differ, a term with no reason, an expected absence, or an earlier block. "This block is done" is not a reason. Reply.
- Adding a project law, or a report rule, to this list.

## Identify

1. Ground the sprint plan in the sprint-ground document the project contract names, before the tracker. The plan's items are the remaining work that document states. Work it records as already done is not an item. If the contract names none, ground the plan in the project's own statement of the goal.
2. Read the latest spine-agent report the project keeps. Correlate its items to the tracker. Update an item where the report and the item disagree. File an item for an actionable report entry that has none. An incomplete correlated item is a block candidate, still subject to the drops below. A review that rescores the same commit and repeats the remaining list is not a new item set.
3. List items. One acceptance is one item. Sentences of one task are one item. Do not offer them as alternatives. The project goal is not an item. The ready queue is not a block. Neighbors in one file are not one item.
4. Drop a paused track, an unchosen design, and any item whose result is unstated.
5. If the mechanism is already in the source, the item is the existing checks. Do not propose a second mechanism. A missing check whose result is already stated in the source is part of that item.
6. Both hosts writing one settled result is that result. Revise a check that still names the older term. Do not roll the hosts back. Do not stop there.
7. A test pinned at F1 stays at F1 until the engine reduces it to F2. Do not invent F2. Do not stop on F1. After the work, if the pin should move from F1 to F2, tell the operator and wait. Do not apply that pin change before approval.
8. Pack the items that can be done now without the operator. Prefer items that make each other cheaper. Isolate an item whose complexity would shrink the others. The largest such bin is the block.
9. Survey all outstanding work before asking what the next frontier is. Do not ask while an executable item remains. Do not ask the operator to split a bin that packs.

## Reduction

One native host may return a more reduced form than the other. The pair may be F1 beside F2, or F0 beside F1. Either bring the hosts to one form then, or leave the difference temporary. Leaving it requires the reason the forms differ, the factoring that produces them, and that the difference is appropriate for that factoring. A difference without that account is not accepted. Do not invent the more reduced form to make the hosts match.

## When a question blocks the block

Ask when the survey shows that no item is executable without the operator. Evaluate every potential block before that question. Ask first about a block that is actionable: the result is stated, or one answer makes it stated. An unstated block waits. Record that evaluation in the tracker and keep it current.

The first question is whether the block belongs to esoterica, to Occult on Occult, or to neither yet. Occult on Occult is one set of files, run by the Go host and the Rust host. If that second step has not shown the item is needed, delay it. It is not an engine erratum yet. Work a semantic only when it is required for esoterica or for Occult on Occult.

A settled project fact is not an open question. The self-rereading parser surface is the original Rust Occult parser.

Ask with the question tool, one question per call, in the way `questioning-the-user` specifies. Wait for the answer. The next question adapts to that answer. Do not list several questions in prose. The question summarizes the work. It does not name a tracker id, and it does not tell the user how to open the tracker. If the detail does not fit the question tool, write it in a markdown file and open that file.

Do not invent F2 as a choice. Do not invent a task for the user. Packing items into a block is not a question.

When the answers are enough to name the block, preview the prompt in that turn. Do not ask whether to preview. Approval still runs the previewed prompt.

## Prompt

Emit one fenced prompt. It contains:

- the repo path and the tracker command for that repo
- each item as a result that can be checked
- settled facts copied from the project, with an instruction not to reopen them
- the exclusions from Identify
- the project's own test and build commands
- continue every item that can be done autonomously and correctly until that work is complete
- stop only when something specific needs the user, and name that thing
- do not add a stop the user did not ask for
- do not ask to proceed
- do not publish unless the user asked for publication in this turn
- commit the completed work of the block before the block ends; the commit contains only that work
- do not implement a semantics the theory does not state; a test pinned at F1 stays at F1; do not invent F2; after the work, if the pin should move from F1 to F2, tell the operator and wait; a host test is evidence of the host; add a test only when the theory already states the result

## Approval

An approval is an explicit yes to the previewed prompt: "do that", "run it", "approved", "execute". Execute that text. A later edit to the prompt replaces it. Continue until the autonomous work is complete, or until something specific needs the user. Do not ask for a second confirmation.

A user edit to the block or the prompt is a new preview. Show the new prompt and wait.

## Reply

Before approval: the first sentence names the block. Then the numbered list. Then the prompt.

After approval, run the prompt. Commit the completed work of the block before the block ends. The commit contains only that work. Do not push unless this turn asked to publish. A diagnosis that changes no file has nothing to commit.

The report opens with a numbered outline, then the body under the same numbers. Numbers start at 1 for that report and are not reused for a different item. Group results that share an outcome and state that outcome once. Under the group, list the program and the term or the failure. Say why the term follows from the program, whether it is the expected term, and what changed from the previous host behavior.

When Evaluate ran, the result is the fixture program, the rendered term, and each host's cost as a number. Report both costs when they match and when they differ. A sentence that the costs differ is not the report. Do not report an expected absence. When something expected is missing, say why. Label a host-only check host. Do not close an item from a paraphrase of the fixture.

List only this block. An acceptance of this block that was not checked is unexpected: say why it was not checked. Do not add a result from an earlier closed block. A passing check does not close a tracker item whose other written clauses are unverified. A green re-run of checks already recorded is not the block while a stated result is still unimplemented. Do not report the step limit in place of cost.

A guidance edit is not a result. If you tell the user about a rule change, say the new behavior in one sentence. A file path is not that sentence. When the user has to ask what a sentence meant, revise the guidance that produced it in that same turn.

End with why this theme's last acceptance was the last one. If the next block needs a decision, ask that one question in the same reply. A count of ready items is not the question. Do not list the ready queue in place of the question.

Include progress toward the project's stated goal, in that statement's words. Say what this block changed about the goal and what of the goal remains. Occulted and esoterica are three steps: complete esoterica, the native primitives required for Occult on Occult; complete Occult on Occult, the pure Occult implementation of the runtime, whose subgoals are not broken out yet; then engine errata surfaced by that second step. Do not invent the step-2 split. Do not invent a percentage. A closed-issue count is not the goal.

After the report, read a spine-agent review newer than the one read at the start. If none is newer, say so. Otherwise say what it scores, whether the commit moved, and which remaining items are new, and apply the tracker update from Identify. A review that rescores the same commit and repeats the remaining list is not a new item set.

"do the next sprint" identifies the next block and executes it in that turn.

## Before finishing

- [ ] The block is a bin of items that can run now. A heavy item is alone. F2 was not invented. A pin move from F1 to F2 was named and not applied.
- [ ] Outstanding work was surveyed before any question. The question was one actionable block. It asked first whether that block belongs to esoterica, to Occult on Occult, or to neither yet. An item the second step has not shown to be needed was delayed.
- [ ] A host at F0 or F1 beside a host at the next form was brought to one form, or left temporary with the reason, the factoring, and that the difference is appropriate.
- [ ] The prompt runs until the autonomous work is complete and names a stop only when something specific needs the user. The commit is only this block's work. Do not push unless this turn asked to publish.
- [ ] The report groups shared outcomes, numbers each item once, lists only this block, gives both costs when Evaluate ran, states the three-step progress, says why the block stopped, and notes a spine-agent review newer than the one read at the start. If the next block needs a decision, the reply asks that one question.
