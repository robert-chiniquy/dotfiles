# Documentation Learnings

Hard-won lessons from documentation projects.

## Avoid Documentation That Drifts

Don't document:
- Specific version numbers ("Go 1.23+ required")
- Line numbers in source citations
- Aggregate statistics ("29% of connectors support X")
- Counts that change ("150+ integrations")

Instead:
- Reference source of truth ("check go.mod")
- Link to files, not line numbers
- Focus on what reader should check for their case
- Use relative terms ("many", "some") or omit counts

## Code Is Ground Truth

Documentation describes intent; code describes reality.

When they conflict:
- Code wins
- Update documentation
- Note why divergence existed

Auto-generated content should be documented as auto-generated, not manual.

## Enumerate First, Theorize Second

Critical anti-pattern: Theory-driven codebase exploration.

DON'T:
- Start with theory about how system works
- Search for files confirming theory
- Stop when you've found "enough" evidence

This is confirmation bias. You'll miss features that don't fit your model.

Instead, enumerate systematically:

```bash
# WRONG: Search for what you expect
grep -r "GetManagers" pkg/

# RIGHT: Enumerate everything, then categorize
ls pkg/foo/*.go
ls pkg/foo/**/*.go
```

Correct process:
1. **Enumerate** - List all files/functions/types
2. **Categorize** - Group by pattern or purpose
3. **Read** - Examine each category
4. **Theorize** - Form model AFTER seeing everything

Signal you're doing it wrong: Searching for patterns rather than listing what's there.

## Anti-Patterns Are Worth Documenting

When you find code that works but shouldn't be imitated:
- Document explicitly as anti-pattern
- Explain why problematic
- Show correct approach
- Real examples beat abstract warnings

## Rhetorical Questions Work (Sparingly)

Engage readers by having them think before presenting solution:
- "What happens when user leaves but still has database access?"
- "How do you sync permissions from behind a firewall?"

One per major section max. Overuse becomes tiresome.

## 70% Rule for First Drafts

Document 70% complete and published > 100% complete never finished.

Ship early, iterate on feedback.

But: Assertion verification pass happens before publication. Catching inaccuracies post-publication erodes trust.

## Contradictions Inform Presentation

When you find contradictions:
- Don't paper over them
- Don't ignore them
- Present current state honestly
- Explain why contradiction exists
- Show path forward if one exists

This builds trust. Readers can tell when docs hide complexity.
