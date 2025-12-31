# Design Skill: Complete Developer Experience

## Purpose

Ensure developer-facing features include all three components required for good DX: Tools, Documentation, and Agents. Missing any leg produces an incomplete experience.

## When to Apply

- Planning any developer-facing features
- Designing APIs, CLIs, SDKs, or platforms
- Evaluating if a feature is "ready to ship"
- Reviewing developer experience proposals

---

## The DX Triad

**Developer Experience requires three components in balance:**

```
           Tools
            /\
           /  \
          /    \
         /  DX  \
        /________\
   Docs          Agents
```

Remove any leg and the stool falls over.

---

## Leg 1: Tools

**Definition:** Software that developers interact with directly.

**Examples:**
- CLI commands
- Local development servers
- Build systems
- Deployment pipelines
- Testing frameworks

**Common mistake:** Thinking DX = tools

**Reality:** Tools without docs or agents = technically correct but unusable

---

## Leg 2: Documentation (Ontology)

**Not just "write docs."** It's the structure, organization, and discoverability of knowledge.

### Information Architecture

How knowledge is organized. Structure should reflect mental models, not implementation details.

**Bad (flat):**
- API Reference
- CLI Commands  
- Tutorials
- FAQ

**Good (ontological):**
```
Getting Started
├── Concepts (mental models)
├── Quickstart (first success)
└── Core Workflows (common tasks)

Building
├── Anatomy
├── Dependencies
└── Best Practices

Testing & Debugging
├── Unit Testing
├── Debugging
└── Common Errors

Reference
├── CLI
├── API
└── Config
```

### Progressive Disclosure

Show exactly what's needed at each stage, no more.

**Bad:** Everything at once
```
Functions are TypeScript modules deployed to AWS Lambda via Deno runtime
with V8 isolate sandboxing and egress control through MITM proxy...
```

**Good:** Progressive layers
```
Step 1: Create a function that lists users
[simple example]

Next: [Testing Locally] [Adding Dependencies] [Advanced Patterns]

(Lambda, Deno, V8 isolates revealed later when relevant)
```

### Example Structure

Examples must be:
- **Copy-pasteable** (no placeholders)
- **Runnable** (actually works)
- **Minimal** (only relevant code)
- **Annotated** (explain why, not just what)

**Bad:**
```typescript
// Example function
export default async function main(input: any): any {
  // Your code here
}
```

**Good:**
```typescript
/**
 * Lists users whose email matches a domain.
 * Input: { domain: "example.com" }
 */
import { SDK } from "sdk-package";

export default async function filterUsers(input: { domain: string }) {
  const sdk = new SDK();
  const users = await sdk.user.list();
  
  // Filter by domain (from input parameter)
  return users.filter(u => u.email?.endsWith(`@${input.domain}`));
}
```

### Discoverability

How developers find answers:

1. **Search** - "How do I call the API?"
2. **Navigation** - Browse logical structure
3. **Contextual links** - Error messages link to docs
4. **In-product hints** - UI suggests relevant docs

---

## Leg 3: Agents

**Definition:** AI assistants that help developers accomplish tasks through natural language.

### Task Execution (Not Just Q&A)

**Examples:**
- "Generate a function that denies odd usernames" → working code
- "Add Zod validation to this function" → code updated
- "Why did this fail?" → explanation + fix

### Context-Aware Assistance

Agent knows:
- What you're building
- Your current code
- Your platform's constraints
- Common patterns

### Multiple Integration Points

**In UI:**
- Alongside editor (Monaco, IDE)
- Generate code from intent
- Explain existing code
- Fix errors

**In CLI:**
```bash
cli generate "function that denies odd usernames"
cli explain-error
cli refine my-function "add date filtering"
```

---

## The Hierarchy of Needs

```
           ┌──────────────────────┐
           │   Delightful         │  ← Agents, polish
           ├──────────────────────┤
           │   Productive         │  ← Good docs, examples
           ├──────────────────────┤
           │   Debuggable         │  ← Can test, see errors
           ├──────────────────────┤
           │   Predictable        │  ← Deterministic, no surprises
           └──────────────────────┘
                    ▲
              FOUNDATION (Tools)
```

**Common mistake:** Starting at the top (delight) before solving the bottom (predictability)

---

## Minimum Viable DX

### Day Zero (2 legs minimum)

**Tools + Documentation = Usable**

You can ship without agents if:
- Tools work reliably
- Documentation has clear learning path
- Examples are comprehensive

**Don't ship with only tools** (technically correct but frustrating)

### Post-Launch (Complete the triad)

**Tools + Documentation + Agents = Lovable**

Add agents within 30-90 days to move from "usable" to "delightful"

---

## Assessment Checklist

### Tools
- [ ] Core workflows function correctly
- [ ] Error messages are clear
- [ ] Can develop and deploy without errors
- [ ] Local development matches production behavior

### Documentation
- [ ] Information architecture designed (not just flat reference)
- [ ] Learning path is clear (beginner → advanced)
- [ ] Examples are copy-pasteable and working
- [ ] Common errors are documented with solutions
- [ ] Search and navigation work
- [ ] Contextual links from errors to docs

### Agents
- [ ] Can generate code from natural language
- [ ] Can explain errors and suggest fixes
- [ ] Can answer questions with context
- [ ] Integrated where developers work (UI, CLI)
- [ ] Understands platform constraints and patterns

---

## Anti-Patterns

### One-Legged Stool: Tools Only
**Symptom:** "It works but developers are confused"  
**Fix:** Add comprehensive documentation with clear structure

### Two-Legged Stool: Tools + Minimal Docs
**Symptom:** "It works but developers get stuck on edge cases"  
**Fix:** Add agents for discovery and assistance

### Inverted Priority: Agents Without Tools
**Symptom:** "AI can generate code but it doesn't run"  
**Fix:** Build reliable tools first, then add agents

### Documentation Anti-Pattern: Reference Only
**Symptom:** "Everything is documented but developers can't find it"  
**Fix:** Add information architecture, learning paths, examples

---

## Implementation Order

### Phase 1: Foundation (Tools)
Build core functionality:
- Basic workflows work
- Essential commands exist
- Can complete key tasks

### Phase 2: Usability (Documentation)
Make tools discoverable:
- Design information architecture
- Write learning-path guides
- Create comprehensive examples
- Add search/navigation

### Phase 3: Delight (Agents)
Add AI assistance:
- Code generation from intent
- Error explanation
- Contextual help
- Interactive refinement

---

## Validation: The 5-Minute Test

**Can a new developer:**

1. **Understand what this is** (2 minutes reading docs)
2. **Create first thing** (3 minutes following quickstart)
3. **Get help when stuck** (agent answers questions)

If any step fails, you're missing a leg of the triad.

---

## Self-Check

Before claiming DX is complete, verify:

- [ ] All three legs identified in design
- [ ] Documentation ontology designed (not deferred)
- [ ] Examples are concrete and runnable
- [ ] Agent integration points identified
- [ ] Acceptable to ship with 2 legs (tools + docs) if timeline requires
- [ ] Plan exists for adding agents post-launch
- [ ] 5-minute test would pass for tools + docs

---

## Key Takeaway

Developer Experience is not just features.

Building great tools without documentation or AI assistance is like building a car without a manual or GPS. It might technically work, but it's unnecessarily difficult.

Budget for all three legs from the start. Don't ship a one-legged stool.
