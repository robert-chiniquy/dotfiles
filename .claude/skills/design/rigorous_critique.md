# Design Skill: Rigorous Critique

## Purpose

Systematically identify problems in a design before implementing it, focusing on unnecessary complexity, missing fundamentals, and feasibility.

## When to Apply

- After creating an implementation plan but before starting work
- When a design feels overly complex
- To validate benefit vs cost and identify scope creep
- As a forcing function for "double back" step in design process

---

## The Three Lenses

### Lens 1: Unnecessary Complexity

**Question:** What can we delete without losing value?

**Look for:**
- New abstractions that wrap simple operations
- Multiple binaries/packages doing similar things
- Features that are "nice to have" disguised as "required"
- Packages created for <100 lines of inline code

**Example:**
```
Proposed: Create pkg/watcher/ with watcher.go, debounce.go, recursive.go (200 lines)

Critique: fsnotify + timer is 30 lines inline. Creating a package adds:
- Another package to understand
- Another API to learn
- More files to navigate
- Abstraction where none is needed

Recommendation: Inline the file watching. Extract later if needed.
```

**Questions:**
- Could this be 3 lines instead of a new package?
- Are we creating abstractions for single use cases?
- What happens if we just delete this entire component?
- Is this feature differentiated or just "keeping up"?

---

### Lens 2: Missing Fundamentals

**Question:** Are we building polish before platform?

**The Level Framework:**
```
Level 2: Polish (progress bars, tab completion, error message quality)
    ↑
Level 1: Workflow (CLI commands, hot reload, templates)
    ↑
Level 0: Platform (auth, schema, data model, deployment lifecycle)
```

**Rule:** Never plan Level 2 before Level 0 is solid.

**Look for:**
- CLI polish (Level 2) without auth solution (Level 0)
- Template features (Level 1) without deployment plan (Level 0)
- Hot reload (Level 1) without schema definition (Level 0)

**Example:**
```
Proposed: Unified CLI with progress bars, shell completion, typo correction

Critique: CLI polish is Level 2. What about:
- Level 0: How does auth work? How do users configure API keys?
- Level 0: What's the schema/data model? How do deployments happen?
- Level 1: What are the core workflows? Can developers test locally?

Recommendation: Solve auth, schema, deployment first. CLI polish later.
```

**Questions:**
- Is the platform architecture defined?
- Can we actually build this, or are core questions unanswered?
- Are we assuming things will "just work" without designing them?
- What Level 0 questions are we avoiding?

---

### Lens 3: Feasibility and Value

**Question:** Is the juice worth the squeeze?

**Look for:**
- "Killer features" that require massive infrastructure
- Features that sound great but have low actual usage
- Features that are "table stakes" (can't differentiate)
- Scope creep (started with 10 features, now 40)

**Example:**
```
Proposed: Replay testing - capture production requests, replay locally

Critique: The value proposition requires production capture, which needs:
- Backend API changes
- Security/privacy review
- Data retention policies
- Auth for fetching production data

Without production capture, replay is just saving test inputs to JSON files.
That's `curl -d @input.json` with extra steps. ~500 lines for marginal value.

Recommendation: Cut replay from v1. Revisit when production capture is feasible.
```

**Questions:**
- What does this feature require that we don't have?
- Would users actually use this, or does it just sound cool?
- Can we deliver 80% of the value with 20% of the work?
- What's the next best alternative that's simpler?

---

## Critique Checklist

Run through these questions for each proposed feature/component:

### Complexity
- [ ] Could this be inline instead of a package?
- [ ] Are we creating abstractions for single-use cases?
- [ ] How many new files/packages does this add?
- [ ] What's the simplest version that still works?

### Fundamentals
- [ ] Have we solved Level 0 (platform) before designing Level 1 (workflow)?
- [ ] Is the architecture defined, or are we assuming it'll work?
- [ ] What core questions are unanswered?
- [ ] Are we skipping hard problems and focusing on easy polish?

### Feasibility and Value
- [ ] What infrastructure does this require?
- [ ] Does the benefit justify the cost?
- [ ] Is this differentiated or just table stakes?
- [ ] Can we deliver a simpler version first?

### Scope
- [ ] How many features have we proposed total?
- [ ] Can we cut 50% and still deliver value?
- [ ] What's the MVP that proves the concept?
- [ ] What can we defer to v2?

---

## Output Format

For each problem identified:

1. **State current proposal** (what the design says)
2. **Identify the problem** (over-engineering, missing fundamentals, or feasibility)
3. **Explain why it's a problem** (maintenance burden, unanswered questions, low benefit vs cost)
4. **Propose alternative** (simpler approach, solve Level 0 first, cut feature)

**Example:**
```markdown
## Problem 3: Replay Testing Without Production

**Current proposal:**
- File-based capture of invocations
- Replay locally with hot reload
- ~500 lines of code

**The problem:**
Replay testing's value proposition requires production capture. Without it, 
this is just saving test inputs to JSON files. That's not differentiated 
enough to justify 500 lines of code.

**Why it matters:**
We'd be building infrastructure for a feature that doesn't deliver its 
core value. Effort could go toward features that work today.

**Alternative:**
Cut replay from v1 entirely. Revisit when production capture infrastructure 
exists. Use those 500 lines for features that work today.
```

---

## Common Patterns

### Pattern: Binary Proliferation
**Symptom:** Multiple binaries doing similar things  
**Question:** Why can't these be subcommands of one binary?  
**Fix:** Unified CLI with subcommands

### Pattern: Premature Abstraction
**Symptom:** New package/module for <100 lines of code  
**Question:** Why not inline this?  
**Fix:** Inline the 30 lines. Extract later if needed elsewhere.

### Pattern: Level Inversion
**Symptom:** Planning CLI polish before platform architecture  
**Question:** How will auth work? What's the deployment model?  
**Fix:** Identify Level 0 gaps, solve those first.

### Pattern: Infrastructure Dependency
**Symptom:** Feature requires major infrastructure not yet built  
**Question:** What's the simpler version that works today?  
**Fix:** Cut feature or build simpler version without infrastructure.

### Pattern: Scope Creep
**Symptom:** 40+ features in initial proposal  
**Question:** What's the 10-feature MVP?  
**Fix:** Ruthlessly cut to MVP, defer rest to v2.

---

## Success Metrics

**Good critique:**
- Cuts 30-50% of proposed features
- Identifies 3+ Level 0 gaps
- Finds at least one "delete this package" opportunity
- Results in clearer, simpler v2 plan

**Bad critique:**
- Approves everything as-is
- Only suggests additions, no cuts
- Focuses on bike-shedding (naming, formatting)
- Doesn't question fundamentals

---

## The Critique Session Process

### Preparation
1. Complete initial design (steps 1-6 of design process)
2. Have written implementation plans
3. Know your feature count and estimated scope

### Mindset
- Assume the design has problems (it does)
- The goal is to find them now, not during implementation
- Deleting features is success, not failure

### Execution
1. **Present the design** (5-10 minutes)
2. **Apply Lens 1: Complexity** - Walk through each component, identify over-engineering
3. **Apply Lens 2: Fundamentals** - Check Level 0 coverage, identify missing architecture
4. **Apply Lens 3: Feasibility** - Evaluate benefit vs cost, identify scope creep
5. **Consolidate findings** - List problems, agree on alternatives

### After the Critique
- **Double back:** Return to earlier design steps with new knowledge
- **Document:** Preserve critique findings (shows the thinking)
- Reference in consolidated plan

---

## Self-Check

Before claiming critique is complete, verify:

- [ ] Applied all three lenses
- [ ] Identified at least 3 problems
- [ ] Proposed alternatives for each problem
- [ ] Created explicit cut list
- [ ] Reduced scope by 30-50%
- [ ] Checked Level 0 coverage
- [ ] Questioned all new abstractions/packages
- [ ] Documented findings in numbered problem format
