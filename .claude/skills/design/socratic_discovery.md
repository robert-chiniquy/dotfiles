# Design Skill: Socratic Requirements Discovery

## Purpose

Use progressively revealing questions to build consensus and illustrate why changes are needed, rather than stating requirements directly.

## When to Apply

- Stakeholders are skeptical of proposed changes
- Requirements seem obvious to you but not to others
- Need to build consensus through discovery rather than assertion
- Want to validate assumptions before committing to solutions

---

## The 7-Step Progression

### 1. Empathy
Start with questions about the user's first experience or common workflow.

**Goal:** Establish shared understanding of current state.

**Examples:**
- "When a developer wants to create their first [thing], what steps do they take?"
- "How long does it take between [action] and [result]?"
- "What happens if [common scenario]?"

### 2. Tension
Ask questions that reveal gaps between expectations and reality.

**Goal:** Create awareness of problems without stating them.

**Examples:**
- "If they've used [competitor], what would they expect?"
- "How many separate commands do they need to learn?"
- "Can they reproduce the same result tomorrow?"

### 3. Exposure
Ask about differences between environments.

**Goal:** Reveal "works on my machine" scenarios.

**Examples:**
- "How does [feature] work locally vs in production?"
- "What happens if authentication works locally but fails in production?"
- "Can a developer test the exact flow that will run in production?"

### 4. Challenge
Ask about the number of components, steps, or configurations.

**Goal:** Reveal unnecessary complexity or over-engineering.

**Examples:**
- "How many binaries exist in this system?"
- "What is the purpose of each one, and why can't they be combined?"
- "Could this be simpler without losing functionality?"

### 5. Comparison
Ask how competitors or industry leaders solve the same problem.

**Goal:** Establish what "good" looks like.

**Examples:**
- "How does [competitor] handle this?"
- "What made [successful platform]'s approach work?"
- "What's preventing us from achieving the same level?"

### 6. Evaluation
Ask if they would make the same choices again.

**Goal:** Open the door to changing direction.

**Examples:**
- "If you were starting fresh today, would you design this the same way?"
- "What are the smallest changes that would have the biggest impact?"
- "Which problems must be solved vs nice to solve?"

### 7. Value
Ask why someone would choose your solution over alternatives.

**Goal:** Articulate the vision of success.

**Examples:**
- "What would make developers choose [your platform] over [competitor]?"
- "What would make them tell colleagues 'you have to try this'?"
- "What currently prevents that recommendation?"

---

## Key Principles

### 1. Questions, Not Answers
❌ "We need X because Y"  
✅ "What happens if Y? [leads to discovering need for X]"

### 2. Concrete Over Abstract
❌ "We need better reliability"  
✅ "Can you reproduce last week's build?"

### 3. Build from Simple to Complex
Start with basics everyone agrees on, progressively reveal deeper issues.

### 4. Use Real Scenarios
Ground questions in actual user workflows, not hypotheticals.

### 5. Allow Discovery
Let questioners arrive at conclusions. Don't shortcut to answers.

---

## Structure: Logical Arc

```
Empathy (understand current state)
    ↓
Tension (reveal problems)
    ↓
Exposure (show gaps)
    ↓
Challenge (question complexity)
    ↓
Comparison (show better ways)
    ↓
Evaluation (open to change)
    ↓
Value (articulate vision)
```

---

## Answer Key Pattern

At the end, provide an answer key showing:
- What each question was designed to reveal
- How answers lead to specific solutions
- The logical chain from question to requirement

**Example:**
```
Q: "Can you reproduce last week's production build byte-for-byte today?"
A: No (dependencies may have updated)
→ Reveals: Non-deterministic builds
→ Solution: Lock files with enforcement
```

---

## Anti-Patterns

### Leading Questions
❌ "Don't you think we should have lock files?"  
✅ "Can you reproduce a build from last month?"

### Too Abstract
❌ "How do we ensure system reliability?"  
✅ "What happens if a dependency changes between test and deploy?"

### Skipping Steps
Don't jump to comparison or evaluation before establishing empathy and tension.

### Revealing Your Solution
Don't mention your proposed solution until questioners have discovered the problem.

---

## Success Metrics

**You've succeeded when:**
- Stakeholders say "we need to fix this" before you propose a solution
- Multiple people independently suggest the same fix
- Objections are about implementation details, not whether to do it

**You've failed when:**
- Stakeholders feel lectured or manipulated
- Questions feel leading or rhetorical
- Discussion becomes defensive rather than exploratory

---

## Application Example

### Problem: Convince team that lock files are critical

**Bad approach (assertion):**
"We need lock files because builds aren't deterministic."

**Socratic approach (discovery):**

1. "If you deploy the same function code on Monday and Friday, does it behave identically?"
2. "What if a dependency released a patch on Wednesday?"
3. "How do you debug an issue that only happens in production if you can't reproduce the exact environment?"
4. "Is non-deterministic authorization behavior acceptable for our platform?"

**Result:** Team discovers the problem themselves, arrives at lock files as obvious solution.

---

## Self-Check

Before using Socratic method, verify:

- [ ] You know the problem you're trying to reveal
- [ ] You have concrete examples and scenarios ready
- [ ] Questions follow the 7-step arc
- [ ] Each question builds on previous answers
- [ ] You're prepared to let stakeholders discover (not lead them)
- [ ] You have an answer key ready to document findings
