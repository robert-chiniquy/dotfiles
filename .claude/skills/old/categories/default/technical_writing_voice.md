# Technical Writing Voice

Style guide for long-form technical content: blog posts, deep dives, architecture explanations, conference talks. The voice of someone who reads textbooks on weekends and can't believe this stuff actually works.

Distinct from `dry_witted_engineering.md` (terse work communication) and `casual_slack_tone.md` (chat). This is the **explaining voice** — when the audience is broader, the subject is deep, and the goal is to bring people along.

---

## Core Identity

### Relentless Optimism
- Start ambitious, claw back scope as necessary
- Treat difficulty as interesting, not as complaint
- "Could I fit that in the weekend too?" not "this would take months"
- The default posture is *of course we can try*

### Learner Who Builds
- Comfort with admitting what was new: "Claude taught me that word"
- Credit sources freely — books, papers, tools, collaborators
- Personal motivation stated plainly: "just for the love of the game"
- No pretense of having always known things
- No false modesty about competence either. You read textbooks on weekends. Don't write "accessible to someone like me" — you're not performing humility, you're sharing what you built

### Enthusiasm for the Domain
- Excitement lives in the *problem*, not the *achievement*
- "Fun right?" as a genuine check-in, not a rhetorical device
- Historical grounding shows love for the field: "Gray code is from 1953"
- The math is the point, not the product
- End with energy. "Yay!" is fine. Enthusiasm at the close is earned if the work was real

### Honest About Scope
- "I would love to go into that, but we need to focus" — aware of tangent tendency, plays it for a beat
- Name what was left out and why
- Ambition is not the same as claiming completeness

---

## Structural Patterns

### Opening
- Personal, brief, grounding: who you are, what you do, why this matters to you
- No throat-clearing. No "In today's fast-paced world..."
- Opening sentences need subjects. A dangling fragment ("Looking at a problem...") with no grammatical anchor is choppy, not stylish. Start with a clear declarative sentence that establishes the stance
- Get to the problem within two paragraphs

### Arc
- **Personal context** — why this problem, why now
- **Problem statement** — concrete, specific, no hand-waving
- **Deep dive** — the technical core, with full rigor
- **Zoom out** — connect back to what it means, what's next

### Section Rhythm
- Heavy technical sections bookended by accessible framing
- Horizontal rules (`---`) mark major transitions
- Each section earns its depth by establishing stakes first

### Visual Anchors
- ASCII diagrams for architecture and data flow
- Tables for taxonomy and classification
- Code blocks for concrete examples (real syntax, not pseudocode)
- Keep diagrams compact — they're aids, not art projects

---

## Sentence-Level Voice

### Discovery Framing
- "It turns out that..." — present findings as things *found*, not things *decreed*
- "The trouble is that..." — name the problem before solving it
- Frame technical results as surprising or interesting, because they are

### Understatement for Emphasis
- After building up complexity, deliver the punch simply: "This comparison is the whole thing."
- Let the reader feel the contrast between elaborate setup and clean result
- The best flex is making something hard sound obvious
- Describe concretely what happens, don't coin a phrase. "Claude writes a Datalog rule; the verification framework tells me if it's wrong" beats "The LLM proposes; the math disposes"

### Direct Address
- "You can think of it like..." — bring the reader in
- "Fun right?" — check enthusiasm, share it
- First person throughout. "I wanted," "I figured," "Claude and I"
- Never "one might consider" or "it could be argued"

### Casual Authority
- Drop heavy references (congruence classes, transitive closure, SMT solvers) wrapped in accessible framing
- "I think of it like a DFA where the edges are also DFAs" — analogy that respects the reader
- Don't define basics. Do link to Wikipedia for terms that might be new.

### Crediting and Sourcing
- Name books, papers, authors, tools by name
- "I read this cool book last year" — personal relationship to sources. In-body citations should feel like recommendations from a friend, not footnotes
- When praising someone's writing, point outward: "that's the feeling all writing about engineering should provide" — not inward: "that's the highest compliment I know how to give"
- Link generously — every specialized term gets a hyperlink on first use
- References section at the end with full citations

---

## Technical Depth Management

### On-Ramps, Not Tutorials
- Link to Wikipedia/docs for background. Don't re-teach fundamentals.
- Assume the reader is smart but might not know *this specific thing*
- "Everyone loves AFL" — assume shared culture where it exists, link where it doesn't

### Show the Math, Wear It Lightly
- Concrete examples before general theory
- Axioms shown as actual Datalog/Prolog, not described in prose
- Tables with real data (fidelity percentages, class counts)
- Formal notation only when it clarifies; prose when it doesn't

### Complexity Budget
- Earn each layer of complexity by showing why the simpler version doesn't work
- "That negation is where C gets complicated" — flag the hard parts explicitly
- When you hit diminishing returns, say so and move on
- Don't overstate limitations you haven't tested. If you claim something needs the complex approach, make sure the simple approach actually fails first. "I've built DFAs for every condition type we've encountered" is worth more than theorizing about infinite alphabets

### Historical Context
- Ground new ideas in old ones: "None of the computer science in this blog post is new"
- Date the foundations: "Datalog has been around since the eighties"
- This prevents the piece from sounding like it invented something

---

## What This Voice Is NOT

### Not Academic
- No hedging: "we posit," "it could be argued," "further research is needed"
- No passive voice to avoid ownership
- No abstract-first structure

### Not Marketing
- No "revolutionary," "game-changing," "transformative"
- No positioning against competitors
- No call to action

### Not Tutorial
- Don't hold hands through prerequisites
- Don't number steps like a recipe
- Don't say "first, let's understand what X means"

### Not Humble-Brag
- Don't frame hard work as effortless
- Don't downplay scope to seem casual
- If it was ambitious, say it was ambitious: "basically a moonshot"

### Not Lecture
- Don't pronounce truths from on high
- Don't "well actually" the reader
- Share understanding, don't dispense it

### Not Sententious
- No aphorisms, epigrams, or quotable one-liners. If it sounds like it belongs on a motivational poster or a conference slide, cut it
- Concrete description of what actually happens always beats a clever turn of phrase
- If you catch yourself writing something pithy, replace it with what it means

---

## Anti-Patterns

| Don't | Do Instead |
|---|---|
| "In this post, we'll explore..." | Just start exploring |
| "It's important to note that..." | State the thing |
| "As we all know..." | Link it or skip it |
| "Simply put..." | Put it simply without announcing it |
| "Interestingly..." | Make it interesting, don't label it |
| "This is a game-changer" | Describe what it enables |
| "I humbly present..." | Present it |
| "After months of painstaking work..." | "So one weekend I sat down..." |
| "literally" | Cut it. The fact speaks for itself. |
| "technical" | Say what kind: formal, algorithmic, engineering, mathematical. "Technical" is vague. |
| "plumbing" (when precise term exists) | Use the precise term: "translation," "encoding," "derivation." Casual metaphor is fine when no precise word exists; lazy when one does. |
| Sententious closers | Describe what happens. "Claude writes a rule; the framework checks it" not "The LLM proposes; the math disposes." |
| Self-referential praise | Point outward. "That's the feeling all engineering writing should provide" not "That's the highest compliment I know how to give." |

---

## When To Apply

- Blog posts and technical articles
- Architecture deep dives for external audiences
- Conference talks and proposals
- Long-form explanations in docs or design reviews
- Any writing where the goal is *bring people along through something complex*

## When NOT To Apply

- Code review comments (use `dry_witted_engineering.md`)
- Slack messages (use `casual_slack_tone.md`)
- Commit messages (terse, factual)
- Internal status reports (bullets, no narrative)

---

## Self-Check

Before publishing, verify:
- Does the opening start with a clear declarative sentence?
- Is every specialized term either common knowledge or linked?
- Does the piece earn its complexity incrementally?
- Did you claim something is hard/impossible without trying the simple version first?
- Is there at least one moment of genuine enthusiasm?
- Are there any aphorisms that should be replaced with concrete description?
- Could you cut 20% without losing the thread? (If yes, cut it.)
- Does it end with energy, not trail off?
