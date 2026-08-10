---
name: overcorrection-review
description: >-
  Review RFCs, plans, designs, code changes, and review findings for needless
  complexity, premature feature exclusions, disproportionate mitigations, and
  choices justified by unverified estimates of difficulty, cost, risk, or
  value. Use as an independent lens in adversarial design and code reviews.
---

# Overcorrection review

Test whether a decision is proportionate to evidence. This is not a mandate to
simplify: preserve explicit security, correctness, compatibility, and product
invariants.

## Hunt for

- **Complexity ratchets:** layers, modes, processes, abstractions, or controls
  without a named invariant they protect.
- **Premature exclusions:** platforms, features, recovery paths, or use cases
  ruled out because they are assumed hard, costly, risky, or low-value.
- **Questionable estimates:** decisions resting on adjectives or intuition
  where measurement, a spike, or existing evidence could change the choice.
- **Disproportionate mitigations:** a broad restriction or redesign applied to
  a narrow failure mode.
- **Irreversible uncertainty:** permanent architecture or scope choices where
  a staged, reversible decision would preserve options.
- **Review-induced scope growth:** a proposed fix that expands beyond the
  introduced defect, pulls in pre-existing debt, or forbids a valid feature.

## Tests

For each suspect choice, ask:

1. What invariant or observed failure requires it?
2. Would the decision change if its difficulty, cost, risk, or value estimate
   were wrong?
3. What is the narrowest mechanism that preserves the invariant?
4. Can uncertainty be resolved by measurement, a bounded spike, or staging?
5. Is the document staging a choice, or silently ruling it out forever?

Do not report a finding unless a plausible alternative preserves the named
invariants. Unfamiliar, multi-step, or security-sensitive work is not
overcorrection by itself. Do not use YAGNI as evidence.

## Finding format

For each finding, report:

- classification from the hunt list;
- severity and whether it blocks;
- exact location and decision;
- hidden estimate or assumption;
- evidence that makes the estimate questionable;
- narrower or more reversible alternative;
- invariant the alternative preserves;
- evidence that would justify the stricter choice.

Block only when an unsupported choice creates meaningful scope loss, failure
surface, or irreversible lock-in. Treat preference-level simplifications as
non-blocking. If clean, state which exclusions, estimates, and mitigations were
tested.
