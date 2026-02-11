# Bar Chart Comparison Skill

When the user asks for a comparison between items (metrics, counts, coverage, etc.), present the data as a narrow ASCII bar chart that fits in a terminal.

## Format

```
Title (units)
=============

Item A |######## 800
Item B |## 200
Item C |################### 1,900

Legend: # = N units (scale to fit)
```

## Rules

1. **Keep it narrow** - Max 40 characters wide for the chart area
2. **Scale appropriately** - Use `#` to represent units, pick a scale that makes differences visible
3. **Right-align numbers** - Put actual values after bars
4. **Add legend** - Show what each `#` represents
5. **Group related items** - Use blank lines or headers to organize
6. **Use `-` for zero/none** - Don't leave blank, show explicit zero

## When to Use

- User asks to "compare" things
- User asks for "how does X stack up against Y"
- User wants to see relative sizes/counts
- Summarizing coverage gaps
- Before/after comparisons

## Example: Documentation Comparison

```
Doc Coverage (lines)
====================

Product     |############ 14,700
Connectors  |################################### 51,300
Developer   |# 224
API         |# 285
c1in-docs   |###### 7,200

# = ~1,500 lines
```

## Example: Feature Coverage

```
Feature Support
===============
          Before  After
Auth        ##      ####
Sync        ###     ###
Provision   -       ##
Events      -       #

# = 25% coverage
```

## Anti-patterns

- Don't use emojis or unicode blocks
- Don't make charts wider than 50 chars total
- Don't omit the scale/legend
- Don't use for single items (just state the number)
