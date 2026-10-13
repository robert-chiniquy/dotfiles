# File layouts

One directory per experiment. All files are markdown. Values a program reads
are in one fenced YAML block or one markdown table per file; read them with a
YAML library and a markdown parser with table support.

## PROTOCOL.md

Prose: the question, the measurement procedure, blinding, guardrails. Then
one YAML block:

```yaml
version: 1
objective:
  kind: number          # number | pass_rate | preference
  direction: higher     # higher | lower (number only)
  unit: m
threshold:
  absolute: 8.0         # or margin_over_baseline: 0.3
delta: 0.5              # win-rate margin when kind is preference
alpha: 0.05
coverage_q: 0.2
budget:
  runs: 60
  batch: 15
blocks: [saturday, sunday]
factors:
  - name: paper
    kind: unordered     # ordered | unordered | integer | continuous
    values: [light, heavy]
  - name: fold
    kind: unordered
    values: [dart, glider, delta]
  - name: nose_clip
    kind: unordered
    values: ["no", "yes"]
infeasible: []
baseline: {paper: light, fold: dart, nose_clip: "no"}
reasonable_moves: all   # or a list of factor names whose one-step moves count
guardrails:
  - throw indoors only
```

An attributed factor lists levels with attribute values:

```yaml
factors:
  - name: headline
    kind: unordered
    attributes:
      - {name: length_words, kind: ordered, values: [short, medium, long], standard: "word count bands set in this protocol"}
      - {name: idiom, kind: unordered, values: [question, imperative, statement], standard: "sentence mood"}
    levels:
      - {id: h01, text: "Ready to save?", length_words: short, idiom: question}
      - {id: h02, text: "Save today", length_words: short, idiom: imperative}
    can_produce_levels: true
```

Distance between configurations: per factor, steps apart divided by steps in
the span (ordered), 0 or 1 (unordered), or the fraction of attributes that
differ (attributed); summed over factors.

## ROUND_<n>.md

Header: round, start, incumbent, current step of each ordered factor,
purpose (reasonable neighbors, or reasonable neighbors plus a frontier sample
with its seed), guardrails. Then one table, rows in run order. The
person running it fills in `result`, `time`, `notes`.

| run | block | paper | fold | nose_clip | result | time | notes |
|---|---|---|---|---|---|---|---|
| r1-01 | saturday | light | glider | no | | | |
| r1-02 | saturday | light | dart | no | | | |

For a preference objective, each row is one duel: columns `left`, `right`
(configuration ids), and `winner`.

## DECISIONS.md

Append one section per round; never edit earlier sections. Each holds a YAML
block and a few sentences reading it.

```yaml
round: 1
start: 1
rule: counting
comparisons:
  - {neighbor: "light/glider/no", test: beats_by_delta, p: 0.05, reject: true}
  - {neighbor: "light/dart/yes", test: beats_by_delta, p: null, reject: false}
dropped_runs: []
move: "light/glider/no"
steps: unchanged
verdict: null
```
