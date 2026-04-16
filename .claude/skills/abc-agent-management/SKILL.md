---
name: abc-agent-management
description: |
  Functional-analysis protocol for iteratively improving subagent performance.
  Adapted from Antecedent-Behavior-Consequence (ABC) analysis in applied
  behavior analysis. Use when delegating to a subagent, when an agent returns
  a poor result, when a pattern of similar failures appears across runs, or
  when coaching a prompt toward reliability.
---

# ABC Agent Management

Treat subagent failures as behaviors with antecedents (the prompt/context
that produced them) and consequences (the feedback loop that sustains them),
not as traits of the agent. Fix the environment, not the "agent."

## Mapping

| ABA construct | Agent equivalent |
|---|---|
| Antecedent (immediate) | The brief given to the agent, constraints, exemplars |
| Setting event (distal) | System prompt, prior turns, tool set, context-window state |
| Behavior (observable) | Tool calls, output, artifacts, the scope the agent chose |
| Consequence | Your response: accept, correct, retry, reject, downstream effect |
| Function | What "need" the behavior met (escape, completeness theater, pattern-match, reinforcement history) |
| Reinforcement | Anything that makes the behavior more likely next time |
| Extinction | Removing what maintained the behavior; expect a burst before improvement |

## Protocol

1. **Operationally define the behavior.** No trait words ("sloppy", "lazy"). State what was observed, measurable. "Emitted 200-line plan, 0 tool calls, on an implementation task" — not "didn't do the work."

2. **Record A, B, C before intervening.** Skipping this is how you fix the wrong thing.
   - **A:** Exact brief. Tool set. What was in the agent's window?
   - **B:** Exact observed behavior. Counts where possible (tool calls, files touched, lines emitted).
   - **C:** What happened next. Accept, correct inline, silent retry? A silent accept reinforces the shape.

3. **Hypothesize function.** Why was this the path of least resistance given A and C? Recurring agent functions:
   - *Escape/avoidance*: ambiguous task → deflect to planning, research, or questions.
   - *Completeness theater*: unclear done-state → over-scope, exhaustive output.
   - *Pattern-match*: exemplar in context (even ill-fitting) → produce its shape.
   - *Reinforcement history*: prior run was accepted → repeat the shape regardless of fit.

4. **Prefer antecedent changes over consequence changes.** Tightening the setup almost always beats tuning the reaction. See *Antecedent Levers* below.

5. **Differential reinforcement: name the replacement.** Never only suppress. "Instead of producing a plan, make the first edit" beats "don't over-plan."

6. **Change one variable at a time.** Single-subject design. Edit the brief, the tool set, and the exemplar simultaneously and you learn nothing when it works.

7. **Expect an extinction burst.** Removing a condition that was reinforcing a behavior can make the first retry look *worse*, not better. Do not roll back on one bad run.

8. **Log the iteration.** Each ABC cycle: antecedent changed, behavior observed, whether it moved. Patterns that recur across agents belong in `bd remember` so future sessions inherit the lesson.

## Antecedent Levers

Concrete edits to the brief/context, in rough order of effect:

- **Done-state.** Name the exact artifact or check that means "done." Removes completeness theater.
- **Scope fence.** State what is *not* in scope. Removes scope creep.
- **Exemplar.** Provide one example of the desired output shape. Agents pattern-match; supply the right pattern.
- **Tool constraint.** Provide only the tools needed. Extra tools invite detours.
- **Context prune.** Remove irrelevant context. A long plan already in the window predicts more planning.
- **Failure hypothesis.** State the specific mistake to avoid, with the reason. "Do not open a PR; we want the diff reviewed first because X."
- **Initiation cue.** Name the first observable action. "Start by running the failing test." Removes where-to-begin ambiguity.

## Common Pitfalls

- **Trait framing.** "This agent is bad at X." Behavior is a function of environment; the same agent behaves differently under different antecedents.
- **Imprecise behavior definition.** You will fix whatever imprecise thing you wrote down. Write it precisely.
- **Stacked changes.** Five prompt edits at once leaves no signal.
- **Silent accept.** Merging a mediocre result without comment reinforces the shape. Either correct it or state the acceptance criteria met.
- **Consequence obsession.** Hours tuning retry feedback usually lose to five minutes tightening the antecedent.
- **Conflating setting events with immediate antecedents.** A bloated context window (setting event) and the specific brief line (immediate antecedent) are both editable, but the levers differ. If every agent run fails, suspect the setting event; if only this run failed, the brief.

## When Not To Use

- Single-turn, low-stakes agent calls where iteration is not planned.
- Research subagents where the output *is* the artifact and there is no retry.

## Source

Adapted from ABC analysis / functional behavior assessment. Reference:
Indiana Resource Center for Autism (Indiana University Bloomington),
*Observing Behavior Using A-B-C Data*:
https://iidc.indiana.edu/irca/articles/observing-behavior-using-a-b-c-data.html
Underlying framework: applied behavior analysis (Cooper, Heron, Heward,
*Applied Behavior Analysis*, 3rd ed.).
