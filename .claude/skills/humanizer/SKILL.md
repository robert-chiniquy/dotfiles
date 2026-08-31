---
name: humanizer
version: 3.0.0
disable-model-invocation: true
description: |
  Remove AI-writing patterns from text. Detects and fixes: inflated symbolism,
  promotional language, em dash overuse, rule of three, AI vocabulary, negative
  parallelisms, conjunctive pileup. Use when editing text to sound human.
allowed-tools:
  - Read
  - Write
  - Edit
  - Grep
  - Glob
  - AskUserQuestion
---

# Humanizer

Strip AI tells from text. Read `references/warp.md` for the full pattern catalog.

## Quick Reference

Most common AI patterns to fix:

1. **Em dash overuse.** AI loves "concept — which means — something." Replace most with commas or periods.
2. **Rule of three.** AI defaults to three examples, three bullets, three adjectives. Vary the count.
3. **Inflated language.** "Revolutionize", "transformative", "paradigm shift" — use plain words.
4. **Negative parallelism.** "Not X but Y" / "Less about X, more about Y" — just say Y.
5. **Conjunctive pileup.** "Moreover, furthermore, additionally" — cut most of these.
6. **Hedging stacks.** "It could potentially perhaps help to consider" — commit to the statement.
7. **Anthropomorphizing.** "The algorithm wants" / "The system tries to" — it doesn't want anything.

## Process

1. Read the text
2. Flag every AI pattern found (cite line)
3. Propose rewrites for each
4. Apply after confirmation

## The Test

Read it aloud. Would a human actually say this? If it sounds like a press release, a textbook, or a helpful assistant, it's still AI.
