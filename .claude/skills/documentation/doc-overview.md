# Documentation Overview

Systematic process for technical documentation that progressively reveals context.

## When to Apply

- Developer documentation (SDKs, APIs, CLIs)
- Onboarding materials
- Complex systems with multiple components
- Any docs that build understanding progressively

## Philosophy

Documentation is gradual exploration, not persuasion. Each topic progressively reveals context, tradeoffs, techniques, and benefits.

Core principle: Reader should never encounter a concept requiring knowledge they haven't yet encountered.

## Phases: Iterative Rounds

Each run through process is a PHASE. Iterate when new information changes assumptions.

```
PHASE 1: Initial pass
    |
    v
[New info?] --yes--> Start PHASE 2
    |
    no
    v
PHASE complete
```

Phase tracking:
- Each phase produces named document (`TOPIC_PHASE_1.md`)
- Later phases incorporate earlier learnings
- Phase documents preserve evolution of thinking

When to start new phase:
- New information changes assumptions
- Contradictions need resolution
- Feedback requires rework
- Deeper understanding reveals gaps

## Levels: Criticality Prioritization

| Level | Criticality | Examples | Priority |
|-------|-------------|----------|----------|
| L0 | Critical | Getting Started, Core Concepts | First |
| L1 | Important | Main workflows, Deployment | After L0 |
| L2 | Supporting | Cookbook, Advanced | After L1 |
| L3 | Reference | Appendices, Glossary, FAQ | Last |

Work order: L0 before L1, L1 before L2, etc.

## Related Skills

- `doc-process.md` - 8-step process
- `doc-content.md` - 7-step pattern, writing guidelines
- `doc-templates.md` - Section templates
- `doc-verify.md` - Verification and audits
- `doc-learnings.md` - Hard-won lessons
- `doc-organization.md` - File structure
- `rap_documentation.md` - Agent-optimized docs
