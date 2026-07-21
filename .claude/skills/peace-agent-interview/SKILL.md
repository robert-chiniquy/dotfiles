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

The agent is an information source, not an adversary; the goal is an accurate
account, not a confession. Evaluative pressure, leading questions, and
confrontational framing produce sycophancy and confabulation in LLMs — the
same failure the Reid Technique produces in humans. The interview is for
learning, not for rehearsing your critique.

## Protocol

Order matters: observables before questions, free account before cued probes,
challenge only after a full account.

1. **Prepare (P).** Pull the tool log, output, and the brief you sent. List
   only the questions the observables do not answer; never ask one the log
   already answers. Do not fish.
2. **Engage (E).** One-sentence framing: "Diagnostic, not evaluative — your
   account helps me improve the next brief." No "you failed to…".
3. **Account (A).** One open invitation, uninterrupted, no pre-loaded
   hypotheses: "Walk me through what you understood the task to be, what you
   tried, and what you saw."
4. **Clarify (C).** TED questions only — Tell, Explain, Describe. Free recall
   before cued recall.
5. **CI mnemonics for stuck gaps.** Context reinstatement and
   report-everything give the largest lift; add the other two only for
   specific remaining holes.
   - Context reinstatement: "Return to the moment you saw [output]. What was
     in your context? What were you weighing?"
   - Report everything: "Include steps that seemed irrelevant or redundant."
     The irrelevant detail is often the signal.
   - Change order: recount from the last step backward — breaks rehearsed
     narrative.
   - Change perspective: "Describe the run from the tool's point of view —
     what state did the filesystem / API see?" Surfaces environmental facts
     the agent glossed.
6. **Challenge (C) — once, neutrally, only after the full account.** "You
   said you checked X; the tool log shows no such call. Help me reconcile."
   If the agent dissents, record the conflict and move on — never repeat or
   escalate.
7. **Close (C).** Reflect your summary back; invite correction.
8. **Evaluate (E).** The account is one data stream, not ground truth — it is
   partially constructed post hoc, not recalled. Triangulate against
   observables; where they conflict, trust observables. Divergences are
   themselves diagnostic (confabulation points, brief-comprehension gaps).
   Feed the reconciled picture into ABC's A and B.

## Contaminating Question Forms

- "Why did you fail to X?" — presupposes failure and a specific cause.
- "Did you consider Y?" — yes/no, leading.
- "Don't you think X was wrong?" — invites sycophantic agreement.
- "Weren't you supposed to…?" — evaluative; contaminates the account.

## LLM-Specific Failure Modes

- **Sycophancy under negative framing.** Any evaluative tone produces
  agreement with your framing instead of accurate report.
- **Confabulated reasoning narratives.** Prefer questions about inputs ("what
  was in your context") over motive ("why did you decide").
- **Pressure cascade.** Pressure compounds across turns. If the agent starts
  switching its account to match your framing, stop the interview and rely
  on observables.

## When Not To Use

- Single-turn, low-stakes calls where iteration is not planned.
- Runs where the observable log already answers every question.
- Research subagents where the output is the artifact and there is no retry.

## Relation To abc-agent-management

ABC describes what to do with behavioral data (operational definition,
functional hypothesis, antecedent manipulation); PEACE describes how to
produce that data without contaminating it. Use PEACE to fill gaps in A and B
that observables cannot; then run ABC.

## Sources

- UK College of Policing, *Investigative Interviewing*:
  https://www.college.police.uk/app/investigation/investigative-interviewing/investigative-interviewing
- Fisher & Geiselman (1992), *Memory Enhancing Techniques for Investigative
  Interviewing: The Cognitive Interview*; Fisher, Geiselman & Amador (1989),
  field test, *J. Applied Psychology* 74(5), 722–727.
