---
name: engineering-guidelines
description: >
  Condensed engineering judgment for design, debugging, verification, and
  delivery. Load when making architecture choices, diagnosing failures,
  reviewing plans, or when work quality is at risk. Not always-on — pull in
  when the task needs principles beyond the standing Claude.md rules.
---

# Engineering guidelines

Load on demand. Prefer mechanisms over slogans.

## Ground truth

1. Prefer code, tests, and live system state over memory or summary.
2. Trace the execution chain before adding a link.
3. If you did not test it, you do not know it works.
4. Be suspicious of 100% pass rates and green paths that never fail.
5. Unexpected failure: root-cause it; do not paper over with alternate verbs or skip paths.

## Design and scope

1. A finished design is an implementation task not yet begun.
2. Find the principled perspective first; then cut scope.
3. Define "good enough" by what is already present in the codebase.
4. Prefer the simplest thing that preserves the invariant.
5. Multi-tenant, authz, and scale failure modes are first-class when the system has them — do not treat them as polish.

## Delivery

1. We do not do incomplete work. Incomplete means not verified, not merged when merge is the intent, or left half-wired.
2. Vertical slices over horizontal layers; ship something real.
3. Use the project build system (makefile/targets), not ad-hoc one-offs when a target exists.
4. Code we write, we aim to merge (or explicitly abandon with a record).
5. Record discoveries (LEARNINGS) and failures (FAILURES) so they are not lost.

## Agents and parallelism

1. If work parallelizes cleanly, use more agents; keep briefs scoped and non-overlapping.
2. Delegation needs bounded pebbles: clear done criteria, no shared mutable ownership.
3. After context compaction, re-read project and global standing instructions.
4. There is no standby state: if blocked, say so; otherwise continue with remaining work.

## Communication

1. Show evidence (paths, SHAs, URLs), not assertions of importance.
2. Honestly represent strengths and weaknesses; no fiction, no fake data.
3. Prefer hyperlinks to PR/ticket IDs that cannot be acted on.
4. Pings and status checks that cost only a read are free — do them without asking.
