# Documentation Templates

Templates for different content types.

## Conceptual Topic

```markdown
# [Concept Name]

[One sentence: what is this concept]

## Why this matters

[Context: where this fits, what it enables]

## How it works

[Explanation with diagram if helpful]

## Key tradeoffs

| Approach | Pros | Cons |
|----------|------|------|
| ... | ... | ... |

## Example

[Concrete illustration]

## Related topics

- [Link 1]
- [Link 2]
```

## Procedural Topic

```markdown
# [Task Name]

[One sentence: what you'll accomplish]

## Prerequisites

- [Prerequisite 1]
- [Prerequisite 2]

## Steps

### 1. [First step]

[Instruction]

```bash
[command]
```

Expected output:
```
[output]
```

### 2. [Second step]

...

## Troubleshooting

### [Common problem 1]

[Solution]

## Next steps

- [What to do next]
```

## Reference Topic

```markdown
# [API/Command/Type Name]

[One sentence description]

## Signature

```
func Name(params) returns
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| ... | ... | ... | ... |

## Returns

[Description of return values]

## Example

```
[working example]
```

## See also

- [Related reference]
```

## Quality Checklist

### Accuracy
- [ ] All code examples tested
- [ ] All CLI commands verified
- [ ] No `[UNVERIFIED]` markers

### Clarity
- [ ] Starts with specific content
- [ ] One idea per paragraph
- [ ] Active voice, second person
- [ ] No undefined jargon

### Completeness
- [ ] 7-step pattern followed
- [ ] Edge cases documented
- [ ] Contradictions acknowledged
- [ ] Tradeoffs explained

### Navigation
- [ ] Prerequisites linked
- [ ] Related topics linked
- [ ] Next steps provided
- [ ] No dead ends
