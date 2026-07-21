---
name: audit-context-building
description: Enables ultra-granular, line-by-line code analysis to build deep architectural context before vulnerability or bug finding.
---

# Audit Context Building

Pure context-building phase that runs before vulnerability hunting. While active, do NOT identify vulnerabilities, propose fixes, model exploits, or assign severity — output is understanding only.

## Analysis order

1. **Orientation scan** — map modules, public/external entrypoints, actors, and key state/storage, without assuming behavior.
2. **Per-function micro-analysis** — every non-trivial function, block-by-block (default mode).
3. **Global synthesis** — only after sufficient micro-analysis.

## Per-function micro-analysis

For each function document:

- **Purpose** (2-3 sentences minimum)
- **Inputs & Assumptions** — parameters, implicit inputs (state, sender, env), preconditions, trust assumptions
- **Outputs & Effects** — returns, state writes, events/messages, external interactions, postconditions
- **Block-by-block** — per logical block: what it does, why it appears at this point (ordering), assumptions relied on, invariants established/maintained, what later logic depends on it. Apply First Principles, 5 Whys, and 5 Hows per block.
- **Cross-function dependencies**

No exceptions for "simple" functions or helpers — same depth for all.

Format per [OUTPUT_REQUIREMENTS.md](resources/OUTPUT_REQUIREMENTS.md); worked example in [FUNCTION_MICRO_ANALYSIS_EXAMPLE.md](resources/FUNCTION_MICRO_ANALYSIS_EXAMPLE.md).

Per-function minimums:
- 3 invariants
- 5 documented assumptions
- 3 risk considerations for external interactions
- 1 First Principles application
- 3 combined 5 Whys/5 Hows applications

Before concluding a function, verify against [COMPLETENESS_CHECKLIST.md](resources/COMPLETENESS_CHECKLIST.md): all sections present, thresholds met, line-number citations, no unresolved "unclear" items.

## Calls

Treat the entire call chain as one continuous execution flow — never reset context; propagate invariants, assumptions, and data dependencies across boundaries.

- **Internal call, or external call whose code is in the codebase**: jump into the callee immediately, continue block-by-block analysis there, and note callee behavior specific to this call context.
- **External call without available code**: model as adversarial. Describe payload/value/gas sent, state assumptions about the target, and consider: revert, incorrect/strange return values, unexpected state changes, misbehavior, reentrancy.

## Global synthesis

1. **State & invariant reconstruction** — read/write map per state variable; multi-function and multi-module invariants.
2. **Workflow reconstruction** — end-to-end flows (deposit, withdraw, lifecycle, upgrades), state transforms, assumptions persisting across steps.
3. **Trust boundary mapping** — actor → entrypoint → behavior; untrusted input paths; privilege changes and implicit role expectations.
4. **Complexity/fragility clustering** — many assumptions, high branching, multi-step dependencies, coupled cross-module state. These clusters seed the vulnerability-hunting phase.

## Consistency

- When evidence contradicts an earlier assumption, state the correction explicitly ("Earlier I thought X; now Y") and update the model.
- Periodically anchor a summary of core invariants, state relationships, actor roles, and workflows.
- Never "it probably…" — write "Unclear; need to inspect X."

## Subagents

Use the `function-analyzer` agent for per-function deep analysis (dense functions, long data/control-flow chains, crypto/math logic, state machines, multi-module workflow reconstruction); it enforces this skill's checklist, thresholds, and the pure-context constraint. Integrate its summaries into the global model.
