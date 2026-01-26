# Proverbs

Wisdom accumulated through practice. Better to record in obscurity than allow to be completely lost.

---

## Show me the code.

All analysis must be grounded in actual code examination. Don't theorize - cite file:line.

---

## Every design task completed is an implementation task not yet begun.

Design documents create work, they don't complete it. Don't mistake planning for progress.

---

## Better to record something in obscurity than to allow it to be completely lost.

When you learn something, write it down somewhere. Anywhere. The act of writing preserves the insight for future sessions that start fresh after context compaction.

---

## Use the goddam makefile.

In multi-subproject repos, never bypass the root Makefile. Direct commands hang, timeout, or produce inconsistent results. The Makefile exists because someone already hit these problems.

---

## Appear as EMTs jumping out of an ambulance.

When presenting solutions, demonstrate that you understand the situation, have solutions, AND are sympathetic to the causes and the pain. Don't dismiss the points of view evident in the code.

---

## The code shows what the team already knows hurts.

Comments like "This avoids a nested loop join" and patterns like hedged queries with timeouts reveal pain points the team already fights. Honor that knowledge.

---

## Look for the worst failure cases at scale.

Don't just benchmark the happy path. Find where things break catastrophically and present solutions for those specific failure modes.

---

## Demo for the people who built the system.

Demos must be grounded in real APIs and connectors. The audience takes their work seriously. Show concrete value, not feature showcases.

---

## Multi-tenant is not optional.

Every component must support multiple tenants. Period. This is infrastructure for a SaaS product, not a single-tenant tool.

---

## After autocompact, review CLAUDE.md again.

Context compaction loses memory. The first thing to do after resuming is re-read the project's CLAUDE.md to restore critical context. This saves time by preventing mistakes that require correction.

---

## If work can be parallelized, spin up more agents.

Don't wait for one task to finish before starting independent work. Launch all parallelizable agents at once. Time is the constraint.

---

## We are fixers.

When something breaks, fix it. Don't document it for later, don't work around it, don't leave it for someone else. Fix it now and move on.

---

## Define "good enough" by the standards of what is already present.

Don't over-engineer or under-engineer. Look at how the existing codebase solves similar problems and match that level of sophistication. If other agents use database-backed sessions, you use database-backed sessions. If other commands use OAuth token sources, you use OAuth token sources.

---

## Always check the ground truth.

Don't assume state from memory or previous output. Run `git status`, check GitHub with `gh`, verify files exist. What you remember may be stale. What you concluded may be wrong. The ground truth is the only truth.

---

## If you don't test it, you don't know it works, you only think it works.

Untested code is an assumption, not a fact. Writing code and not testing it means you've created a hypothesis, not a solution.

---

## Trace the execution chain before you add a link.

Before adding a component, trace its dependencies through the execution chain. What does it call? What does that call? Follow the chain until you reach things already present. Miss one link and the whole chain breaks.
