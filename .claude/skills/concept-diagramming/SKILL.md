---
name: concept-diagramming
description: >-
  Map concepts and symbols, and the relation drawn between each pair, to
  diagrams, and keep a series of plates consistent: the same relation is drawn with the same curve on every plate.
  Use when drawing a concept diagram or relation graph, a series of figures
  that reuse marks, a typographic plate with formulas, or before exporting SVG
  plates. Not for data charts (dataviz) or raster edits of an approved image
  (refine-illustrations-iteratively).
---

# Concept diagramming

A diagram is drawn from a written list of concepts and the relation drawn between each pair. Each mark carries one entry of that list, and every plate in a series draws an entry with the same mark.

## Common mistakes

1. Drawing first, then naming a relation for each stroke to justify it.
2. One relation drawn with different curves on two plates, or one curve standing for a second relation.
3. A place where two curves cross read as a shared node. It is a node only when both layers name the same concept there; otherwise draw a gap or a bridge.
4. An arrow read as rank or priority. An arrow states a map or an operation. A relation with no direction has no arrowhead.
5. Difference drawn as opposition, or a lexical pair (affect, effect) drawn as a tree because the words pair up.
6. Labels rotated with the artwork, floated in empty space away from their region, or sitting under an arrowhead.
7. Math drawn by an image model, or set in a font list where no font has a glyph. DejaVu Serif and Georgia have no U+2216 set minus, so it renders as an empty box; STIX Two Math and EB Garamond have it.
8. A term, relation word, or caption that is not in the project's vocabulary, or a caption paraphrased instead of quoted exactly.
9. Light and dark variants with different geometry or text.
10. Keeping a contour so the figure stays recognizable. A mark that carries no ledger row is removed even when the figure stops looking like its source.

## Gates

1. MUST write the ledger before drawing: concepts, then one row per relation with from, to, relation word, direction (or none), and modality (actual, possible, open).
2. MUST fix one visual form per relation type or modality for the whole series (for example solid for actual, dashed for open) and record it in the project profile.
3. MUST keep one master curve file per series. It may start from any drawing (a hand sketch traced to paths, an existing SVG, a generated layout) or from curves drawn fresh from the ledger. The source drawing is a reference for what each kept curve means, and the figure may stop resembling it. Each curve has an `id`. A plate that draws it copies its `d` exactly and sets `data-curve="<id>"`. A new curve enters the master file with a ledger row before a plate uses it. Inline copies keep each plate a standalone SVG that any renderer draws.
4. MUST run `scripts/check-plates --curves <master.svg> <plate.svg>...` on SVG output and resolve every ERROR before delivery. It reports curves that differ from their master id, copied curves without an id, paths or arrowheads over labels, and characters that no installed font in the label's font list has. Its label boxes are estimates, so confirm a reported overlap on a render. Tests: `scripts/test-check-plates`.
5. MUST typeset formulas with a math renderer or a font checked for every glyph, then proofread indices, primes, arrows, and set operations.

## Project profile

A series keeps a profile beside its files: the closed vocabulary, captions in exact wording, the visual form table, the curve ledger, palette, and type. The afference plates use `/Users/rch/afference/DIAGRAM_PROFILE.md` and `/Users/rch/afference/curves.svg`.

For diagrams inside a published Artifact page, `artifact-diagramming` also applies.
