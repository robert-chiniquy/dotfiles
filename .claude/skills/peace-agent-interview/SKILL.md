---
name: peace-agent-interview
description: |
  Post-run elicitation protocol for subagents. Adapted from the PEACE model
  (UK College of Policing) and the Cognitive Interview (Fisher & Geiselman) —
  the evidence-based, non-coercive replacement for the Reid Technique.
  Use after a subagent returns a poor or confusing result and before redesigning
  the brief, to get the agent's uncontaminated account of what it understood,
  tried, and saw. Complements abc-agent-management: PEACE produces the clean
  behavioral data that ABC analyzes.
---

# PEACE Agent Interview

Treat the agent as an information source, not an adversary. The goal is an
accurate account, not a confession. Leading questions, pressure, and
confrontational framing produce sycophancy and confabulation in LLMs — the
same failure mode the Reid Technique produces in humans. Use structured
open-ended elicitation instead.

## The Agent's Account Is One Data Stream

An agent's self-report is a source, not ground truth. Triangulate it against
observable evidence: the tool log, the output, the brief you sent, the context
it had. Where narrative and observables conflict, trust observables.

## Mapping

| PEACE / CI element         | Agent-interview equivalent                                                                                                                  |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| P — Planning & Preparation | Collect observables first (tool calls, output, the brief). Write the specific gaps you need to close. Do not fish.                          |
| E — Engage & Explain       | Frame as diagnostic, not evaluative. "I want to understand the run so I can improve the next brief."                                        |
| A — Account                | One open invitation, uninterrupted. "Walk me through what you understood the task to be, what you tried, and what you saw."                 |
| C — Clarification          | TED questions only — Tell, Explain, Describe. Never "did you…" or "was it…".                                                                |
| C — Challenge              | Only after a full account. Present the inconsistency neutrally; do not repeat under dissent.                                                |
| C — Closure                | Reflect your understanding back; let the agent correct.                                                                                     |
| E — Evaluation             | Triangulate account vs observables. Feed the reconciled picture into ABC.                                                                   |
| CI: context reinstatement  | "Return to the moment you saw [specific output]. What was in your context? What were you weighing?"                                         |
| CI: report everything      | "Include steps that seemed irrelevant or redundant." The "irrelevant" detail is often the signal.                                           |
| CI: change order           | "Describe what you did starting from the last step and working backward." Breaks rehearsed narrative.                                       |
| CI: change perspective     | "Describe the run from the tool's point of view — what state did the filesystem / API see?" Surfaces environmental facts the agent glossed. |

## Protocol

1. **Prepare.** Pull the tool log, output, and the brief you sent. List the
   specific questions the observables do not answer. If a question is already
   answered by the log, do not ask it.

2. **Engage.** One-sentence framing: "Diagnostic, not evaluative. Your account
   helps me improve the next brief." Neutral tone. No "you failed to…".

3. **Account.** Single open prompt. Do not interrupt. Do not pre-load
   hypotheses.

4. **Clarify with TED.** Open questions to fill gaps. Free recall before cued
   recall.

5. **Apply CI mnemonics for stuck gaps.** Context reinstatement and report-
   everything give the largest lift; add change-order and change-perspective
   only if the first pass leaves specific holes.

6. **Challenge neutrally, once.** "You said you checked X; the tool log shows
   no such call. Help me reconcile." If the agent dissents, record the
   conflict and move on — do not escalate.

7. **Close.** Reflect summary. Invite correction.

8. **Evaluate.** Reconcile account vs observables. Note where they diverge;
   divergences are themselves diagnostic (confabulation points, prompt-
   comprehension gaps). Feed the reconciled picture into ABC's A and B.

## Question Forms

Use:
- "Tell me what you understood the task to be."
- "Walk me through what you tried."
- "Describe what was in your context when you decided X."
- "What did you see as the done-state?"

Do not use:
- "Why did you fail to X?" — presupposes failure and a specific cause.
- "Did you consider Y?" — yes/no, leading.
- "Don't you think X was wrong?" — invites sycophantic agreement.
- "Weren't you supposed to…?" — evaluative; contaminates the account.

## LLM-Specific Failure Modes

These are adaptations of the human-interview research to LLMs; weight them
accordingly.

- **Sycophancy under negative framing.** Any evaluative tone from the
  interviewer produces agreement with your framing rather than accurate
  report. Keep the register neutral; never escalate.
- **Confabulation of reasoning narratives.** Agents generate plausible stories
  for decisions they did not "make" in the implied sense. Prefer questions
  about inputs ("what was in your context") over questions about motive
  ("why did you decide").
- **Post-hoc rationalization.** The account produced after a run is partially
  constructed, not recalled. Observables are the cross-check.
- **Pressure cascade across turns.** Any pressure compounds over multi-turn
  interviews. If the agent starts switching its account to match your
  framing, stop the interview and rely on observables.

## Common Pitfalls

- Starting the interview before pulling the observables.
- Leading or yes/no questions in the clarification phase.
- Interrupting the account phase to probe a detail.
- Treating the narrative as ground truth.
- Repeating a challenge the agent has dissented on.
- Mixing evaluation ("this was wrong") with account-gathering.
- Using the interview to rehearse your own critique rather than to learn.

## When Not To Use

- Single-turn, low-stakes calls where iteration is not planned.
- Runs where the observable log already answers every question you have.
- Research subagents where the output is the artifact and there is no retry.

## Relation To abc-agent-management

ABC describes *what to do* with behavioral data (operational definition,
functional hypothesis, antecedent manipulation). PEACE describes *how to
produce* that behavioral data from the agent itself without contaminating
it. Use PEACE to fill gaps in A and B that the observables alone cannot
resolve; then run ABC.

## Sources

- UK College of Policing, *Investigative Interviewing* (authorized
  professional practice):
  https://www.college.police.uk/app/investigation/investigative-interviewing/investigative-interviewing
- Fisher, R. P. & Geiselman, R. E. (1992). *Memory Enhancing Techniques for
  Investigative Interviewing: The Cognitive Interview.* Charles C. Thomas.
- Fisher, R. P., Geiselman, R. E. & Amador, M. (1989). Field test of the
  cognitive interview. *Journal of Applied Psychology*, 74(5), 722–727.
