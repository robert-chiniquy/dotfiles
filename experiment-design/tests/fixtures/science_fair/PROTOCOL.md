# Paper airplane distance

Synthetic fixture: every value in this directory is made up to exercise the
tool. It follows the science fair walkthrough in the experiment-design
skill's `references/examples.md`.

Question: which paper, fold, and nose clip make a paper airplane fly
farthest? Measurement: distance from the throw line to where the plane first
touches the floor, by tape measure, in meters. The thrower does not measure.

```yaml
version: 1
objective:
  kind: number
  direction: higher
  unit: m
threshold:
  absolute: 8.0
delta: 0.5
alpha: 0.05
coverage_q: 0.2
budget:
  runs: 60
  batch: 15
blocks: [saturday, sunday]
factors:
  - name: paper
    kind: unordered
    values: [light, heavy]
  - name: fold
    kind: unordered
    values: [dart, glider, delta]
  - name: nose_clip
    kind: unordered
    values: ["no", "yes"]
infeasible: []
baseline: {paper: light, fold: dart, nose_clip: "no"}
reasonable_moves: all
guardrails:
  - throw indoors only
```
