---
name: complete-developer-experience
description: |
  Ensure developer-facing features include all three DX components: Tools,
  Documentation, and Agents. Use when planning developer-facing features,
  designing APIs/CLIs/SDKs, evaluating if a feature is ready to ship, or
  reviewing developer experience proposals.
---

# Complete Developer Experience

## Common Mistakes

1. **Docs that skip the split.** A new project's README/docs MUST try to lead with a value prop (seam, what stays where, what that buys, what it is not, limits) before tools and commands. MUST skip the lead if it is not specific. Tools + docs still ship together; the docs leg is the split, not a command list.

Developer Experience requires three components in balance: Tools, Documentation (ontology), and Agents.

The DX Triad: Remove any leg and the stool falls over. Tools without docs = technically correct but unusable. Docs without tools = theory. Agents without tools = code that doesn't run.

Hierarchy of needs (bottom up): Predictable, Debuggable, Productive, Delightful. Common mistake: starting at the top before solving the bottom.

Minimum viable DX: Tools + Documentation = Usable (ship day zero). Tools + Documentation + Agents = Lovable (add agents within 30-90 days).
