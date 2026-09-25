---
name: questioning-the-user
description: Protocol for putting pending decisions to the user one at a time as extended multiple choice. Use when a plan or work product surfaces multiple decisions the user must make, when asking clarifying questions, or when the user says "ask me the questions", "one at a time", or "walk me through the decisions".
---

# Questioning the User

When decisions are pending for the user, put them to the user one at a
time with the question tool (AskUserQuestion) — never as a prose list
of questions, and never several questions in one call.

- One question per call; wait for the answer before asking the next.
  Later questions must adapt to earlier answers: drop ones that became
  moot, reshape options that changed.
- Upstream-first ordering: the decision that shapes other decisions
  goes first.
- Extended options: 2-4 per question, each with a short label and a
  description carrying the implication and trade-off, so the user can
  decide without re-reading source material. Put the recommended
  option first with "(Recommended)" in its label and the reason in its
  description.
- A question that rests on a design doc, plan, RFC, or tracker item
  gives the material's full absolute path (or the exact tracker
  command, e.g. `bd show <id>`) in the question or the message
  immediately before it, plus enough inline summary to decide without
  leaving the conversation.
- Multi-select only for genuinely non-exclusive choices. The tool adds
  "Other" automatically — never add a catch-all option yourself.
- After the final answer, restate all decisions compactly and proceed
  on them; don't re-confirm.
- Don't manufacture questions: decisions with a conventional default,
  or answers checkable from the code/repo, are not user questions.
