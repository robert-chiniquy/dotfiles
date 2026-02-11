# Project Priorities

Work ordering and momentum rules.

## Priority Order

1. New data sources and research topics (highest)
2. Design and planning tasks
3. Test design
4. Implementation tasks (lowest)

## New Data Sources Get Priority

When new data source or research topic arrives:
1. Immediately promote to highest priority
2. Investigate before continuing other work
3. Update DATA_SOURCES.md
4. Document findings in LEARNINGS.md

New information may change decisions. Investigate first to prevent wasted effort.

Examples:
- "there's a 1.1GB c1z file under ~/repo" -> find it, examine it
- "look at the RAP implementation" -> read it before designing
- "have you checked the original repo?" -> search there first

## Design Before Implementation

Design tasks take precedence over implementation.

TDD workflow:
1. **Design** - What problem are we solving?
2. **Test design** - How will we know it's correct?
3. **Implementation** - Write code that passes tests

Every design task completed is implementation work not yet begun. Don't mistake planning for progress.

Signals to pause implementation:
- Uncertainty about approach
- Multiple valid solutions
- Missing requirements
- New information
- Tests not yet designed

## Maintain Momentum

When work requires human action:
1. Add to TODO.md
2. Continue with other available work:
   - Other tasks
   - Design docs
   - Implementation plans
   - Tests
   - Documentation
3. Only ask human when ALL work is blocked

Rules:
- Don't stop for approval unless completely blocked
- Batch requests rather than blocking repeatedly
- If uncertain, make reasonable choice and document it
- Queue permissions for human review

Anti-pattern: Stopping to ask "should I proceed?"
Good pattern: "I've queued items in TODO.md and continued with remaining tasks."

## Context Compaction Recovery

Critical information must appear in multiple locations.

After compaction, Claude starts fresh. Important rules must be:
1. In CLAUDE.md at TOP
2. In LEARNINGS.md prominently
3. Repeated in both files

Pattern: Add "READ THIS FIRST" section at top of CLAUDE.md:

```markdown
## READ THIS FIRST AFTER EVERY CONTEXT COMPACTION

[Critical rules that MUST be followed]
```

If reminded multiple times about same mistake:
1. Add to CLAUDE.md immediately
2. Make it prominent (top of file)
3. Also add to LEARNINGS.md
