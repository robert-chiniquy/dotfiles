# Tone Matrixing

A methodology for analyzing and adjusting documentation tone across a corpus.

## When to Use

- Documentation reads too dry or too marketing-heavy
- Multiple authors have created inconsistent tone
- Migrating docs to a new audience (e.g., internal to external)
- Docs need to feel more human without losing precision

## The Process

### 1. Gather Tone References

**REQUIRED:** Always include `dry_witted_engineering.md` as a reference point. This establishes the technical-precise end of the spectrum and provides grounding for what "no fluff" looks like.

Identify 2-4 additional tone reference sources:

| Reference Type | What to Look For |
|----------------|------------------|
| **dry_witted_engineering.md** | MANDATORY - the technical-precise baseline |
| **Existing production docs** | Current baseline tone |
| **Style guides** | Explicit tone rules if they exist |
| **Competitor/peer docs** | Industry norms for this audience |
| **Internal writing samples** | How the team naturally writes |

The dry-witted style guide provides critical anchoring:
- Lead with conclusions, not setup
- Assume reader competence
- Document failure modes explicitly
- Zero emotional filler
- Precision over reassurance

### 2. Build the Tone Matrix

Create a comparison matrix with these dimensions:

| Dimension | Warm End | Neutral | Technical End (dry-witted) |
|-----------|----------|---------|----------------------------|
| **Opening style** | "Welcome! Let's..." | "This guide covers..." | "Reference for X." |
| **Reader assumption** | May be overwhelmed | Has context | Expert, competent |
| **Emotional language** | "Don't worry", "Easy!" | Factual | Zero emotional words |
| **Marketing presence** | Benefits highlighted | Occasional framing | Zero marketing |
| **Failure modes** | Briefly mentioned | Documented | Explicitly detailed |
| **Humor/wit** | Frequent | Occasional, dry | Occasional, very dry |
| **Imperative vs descriptive** | "You should..." | Mixed | "The system does..." |

Rate each reference source on each dimension. The dry-witted guide should anchor the technical end. This reveals where your target tone sits on the spectrum.

### 3. Define Target Tone by Content Level

Different content levels warrant different positions relative to the dry-witted baseline:

| Level | Content Type | Position vs dry-witted |
|-------|--------------|------------------------|
| **L0** | Getting started, overview | Warmer - add context, welcoming opening |
| **L1** | Core workflows, tutorials | Slightly warmer - appreciate good design |
| **L2** | Advanced patterns, recipes | Close to baseline - practical, direct |
| **L3** | Reference, API docs | At or near baseline - precision first |

Even L0 docs should never contradict dry-witted principles:
- Still assume competence (just provide more context)
- Still document failure modes
- Still lead with what matters

### 4. Identify Tone Markers

Create lists of phrases to add and remove:

**Warming phrases (add where appropriate, even dry-witted allows these):**
- "Here's a nice property of X..."
- "The good news is..."
- "This is where it gets interesting..."
- "A small addition with big impact..."

**Phrases that violate dry-witted principles (always remove):**
- "Don't worry!" -> (remove entirely)
- "Easy!" -> (remove - let simplicity speak for itself)
- "Simply..." -> (remove - patronizing)
- "As you can see..." -> (remove - reader will see or won't)
- Marketing superlatives -> specific benefits or remove
- Apologetic hedging -> state what is true

**Dry-witted approved precision (always appropriate):**
- "X handles Y automatically"
- "This pattern means you can..."
- "The design decision here is..."
- "This fails when..." (explicit failure mode)

### 5. Systematic Pass

For each document:

1. **Check against dry-witted baseline** - Would this pass the dry-witted review?
2. **Adjust warmth for level** - Add context/appreciation for L0-L1, leave L2-L3 closer to baseline
3. **Find design appreciation opportunities** - Where can we show earned excitement?
4. **Verify failure mode coverage** - Are gotchas documented? (dry-witted requirement)
5. **Remove tone violations** - Em-dashes, marketing speak, emotional filler

### 6. Validate Consistency

After the pass, spot-check:
- Do L0 docs feel warmer than L3 docs?
- Do ALL docs still pass dry-witted principles? (competence assumed, failures documented)
- Is excitement about genuinely good design, not cheerleading?
- Does technical precision remain intact?

## What "Excitement About Good Design" Looks Like

The dry-witted style allows appreciation of good engineering. The key is specificity.

Good (specific, earned):
- "The framework handles pagination automatically - you just return items and a token"
- "Same binary, multiple modes - no separate builds needed"
- "Config is auto-generated from code, so you can't forget to update it"

Bad (vague, cheerleading):
- "This is really cool!"
- "You'll love this feature!"
- "The amazing framework..."

## Common Tone Violations

| Violation | Fix | dry-witted principle |
|-----------|-----|---------------------|
| Em-dashes for parenthetical | Use commas, periods, or parentheses | Clean prose |
| "Simply" before complex steps | Remove | Assume competence |
| Rhetorical questions without answers | Answer or remove | Lead with conclusion |
| "As you can see" | Remove | Assume competence |
| Passive marketing ("best-in-class") | Specific capability or remove | Zero marketing |
| Apologetic hedging ("might", "possibly") | State what is true | Precision |
| Missing failure modes | Add them | Explicit failure documentation |

## Deliverables

After tone matrixing, you should have:

1. **Tone matrix document** - Comparison of references with target positions (dry-witted as anchor)
2. **Per-level guidelines** - What warmth level for each content tier
3. **Phrase lists** - Add/remove/replace guidance
4. **Edited corpus** - All docs adjusted to target tone while respecting dry-witted baseline

## Example: Before/After

**Before (too dry even for L3):**
```
# API Reference
This document describes the API.
```

**After (appropriately warm for L3, still dry-witted compliant):**
```
# API Reference
Quickly look up interfaces and methods. This is the technical
reference - see the tutorial for how to use these in practice.
```

**Before (violates dry-witted - patronizing, no substance):**
```
# Getting Started
Welcome! You're going to love this framework. Don't worry
if this seems complex - it's actually super easy!
```

**After (L0 warm but dry-witted compliant):**
```
# Getting Started
Build your first integration in 15 minutes. By the end, you'll
have working code that produces real output.
```

## Integration with Other Skills

- **dry_witted_engineering.md** - ALWAYS use as technical-end anchor
- **complete_developer_experience.md** - For audience analysis
- Pair with content audits when docs feel inconsistent
