# Documentation Verification

Verification passes before publication.

## Technical Verification

Checklist:
- [ ] Every code example runs successfully
- [ ] All CLI flags exist in current binary
- [ ] All SDK methods exist in current version
- [ ] All file paths are correct
- [ ] All line number citations accurate

Output: Verified draft with all `[UNVERIFIED]` markers resolved.

## Assertion Verification Pass

Extract and verify every assertion before publishing.

Process:
1. Extract every assertion (claims, patterns, recommendations)
2. Review each against source code/implementations
3. Identify nuances, edge cases, exceptions
4. Update documentation to reflect reality

Output format:

| Section | Assertion | Status | Nuance Found |
|---------|-----------|--------|--------------|
| ... | ... | Verified/Partial/Wrong | ... |

Purpose:
- Catch subtle inaccuracies
- Discover undocumented patterns
- Build confidence in accuracy

## AI-Voice Audit Pass

Review for passages that read as AI-written.

| Pattern | Example | Fix |
|---------|---------|-----|
| Generic hedging | "It's important to note..." | Delete or be specific |
| Filler transitions | "Additionally, furthermore" | Simpler connectives |
| Vague enthusiasm | "This powerful feature..." | Name the capability |
| Passive padding | "It should be noted that X" | "Do X" |
| List-itis | Every section is bullets | Vary structure |
| Over-qualification | "In many cases, depending..." | State common case |
| Missing specifics | "Various options available" | Name the options |
| Robotic consistency | Identical section structure | Vary rhythm |
| Unnecessary words | "by following this workflow" | "with this workflow" |
| Weak constructions | "There is a tool that..." | "The tool..." |
| Ability phrases | "the ability to X" | "can X" |

Process:
1. Read aloud (or text-to-speech)
2. Flag corporate boilerplate
3. Delete, simplify, or add specifics
4. Verify rewrite sounds like person explaining to colleague

The test: Would you say this at a whiteboard?

## Publish Checklist

- [ ] Remove internal location references
- [ ] Replace internal citations with public links
- [ ] Remove internal statistics
- [ ] No internal project names
- [ ] All links publicly accessible

## Test Reader Journey

Before publishing, walk through as each persona:
- Can new developer go zero to working code?
- Can operator deploy without reading developer docs?
- Does each page have clear next step?

If you get stuck, documentation has a gap.
