---
name: incomplete-work-audit
description: |
  Systematically locate, analyze, and resolve incomplete work markers in a
  codebase. Use when user asks to find TODOs, show unfinished work, audit
  incomplete items, or find and fix TODOs. Covers TODO, FIXME, TBD, XXX,
  HACK, stub, placeholder, and documentation omission markers.
---

# Incomplete Work Discovery and Resolution

Systematically locate, analyze, and resolve incomplete work markers in a codebase (TODO, FIXME, TBD, XXX, HACK, stub, placeholder, etc.).

Three phases: Discovery (search for markers, present structured table), Planning (user-triggered, write implementation plans as code comments), Implementation (user-triggered, execute plans incrementally).

Searches explicit markers (TODO, FIXME, XXX, HACK), implicit markers (placeholder, stub, not implemented), panic/error markers, and documentation omission markers (does NOT include, Not Modeled, Out of scope, Future work).
