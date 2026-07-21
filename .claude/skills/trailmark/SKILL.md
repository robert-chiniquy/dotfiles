---
name: trailmark
description: "Builds and queries multi-language source code graphs for security analysis. Includes pre-analysis passes for blast radius, taint propagation, privilege boundaries, and entry point enumeration. Use when analyzing call paths, mapping attack surface, finding complexity hotspots, enumerating entry points, tracing taint propagation, measuring blast radius, or building a code graph for audit prioritization. Supports 16 languages including Solidity, Cairo, Circom, Rust, Go, Python, C/C++, TypeScript."
---

# Trailmark

Parses source code into a directed graph of functions, classes, calls, and
semantic metadata for security analysis. Static only. One language per graph:
in polyglot repos, build the full graph per component with the right
`--language` (FFI boundaries are where bugs cluster — don't sample).

Mutation testing triage belongs to the genotoxic skill, which calls trailmark
internally. Run `engine.preanalysis()` before handing off to genotoxic or
`diagramming-code`.

## Installation

If `uv run trailmark` fails: `uv pip install trailmark`. Do not fall back to
manual code reading as a substitute; if installation fails, report the error.

## Quick Start

```bash
uv run trailmark analyze --summary {targetDir}          # Python (default)
uv run trailmark analyze --language rust {targetDir}
uv run trailmark analyze --complexity 10 {targetDir}    # complexity hotspots
```

### Programmatic API

```python
from trailmark.query.api import QueryEngine

engine = QueryEngine.from_directory("{targetDir}", language="rust")  # default "python"

engine.callers_of("function_name")
engine.callees_of("function_name")
engine.paths_between("entry_func", "db_query")
engine.complexity_hotspots(threshold=10)
engine.attack_surface()
engine.summary()
engine.to_json()

result = engine.preanalysis()

# Subgraphs exist only after preanalysis()
engine.subgraph_names()
engine.subgraph("tainted")
engine.subgraph("high_blast_radius")
engine.subgraph("privilege_boundary")
engine.subgraph("entrypoint_reachable")

from trailmark.models import AnnotationKind
engine.annotate("function_name", AnnotationKind.ASSUMPTION,
                "input is URL-encoded", source="llm")
engine.annotations_of("function_name")
engine.annotations_of("function_name", kind=AnnotationKind.BLAST_RADIUS)
```

## Pre-Analysis Passes

Blast radius, taint, and privilege data exist only after
`engine.preanalysis()`. Four passes:

1. **Blast radius** — downstream/upstream node counts per function,
   critical high-complexity descendants
2. **Entry point enumeration** — entrypoints by trust level, reachable
   node sets
3. **Privilege boundary detection** — call edges where trust level changes
   (untrusted -> trusted)
4. **Taint propagation** — marks nodes reachable from untrusted entrypoints

Results are stored as annotations and named subgraphs. Details:
[references/preanalysis-passes.md](references/preanalysis-passes.md).

## Supported Languages

| Language | `--language` value | Extensions |
| --- | --- | --- |
| Python | `python` | `.py` |
| JavaScript | `javascript` | `.js`, `.jsx` |
| TypeScript | `typescript` | `.ts`, `.tsx` |
| PHP | `php` | `.php` |
| Ruby | `ruby` | `.rb` |
| C | `c` | `.c`, `.h` |
| C++ | `cpp` | `.cpp`, `.hpp`, `.cc`, `.hh`, `.cxx`, `.hxx` |
| C# | `c_sharp` | `.cs` |
| Java | `java` | `.java` |
| Go | `go` | `.go` |
| Rust | `rust` | `.rs` |
| Solidity | `solidity` | `.sol` |
| Cairo | `cairo` | `.cairo` |
| Haskell | `haskell` | `.hs` |
| Circom | `circom` | `.circom` |
| Erlang | `erlang` | `.erl` |

## Graph Model

**Node kinds:** `function`, `method`, `class`, `module`, `struct`,
`interface`, `trait`, `enum`, `namespace`, `contract`, `library`

**Edge kinds:** `calls`, `inherits`, `implements`, `contains`, `imports`

**Edge confidence:** `certain` (direct call, `self.method()`), `inferred`
(attribute access on non-self object), `uncertain` (dynamic dispatch).
Account for `uncertain` edges in security claims — dynamic dispatch is
where type confusion bugs hide.

Per code unit: parameter/return/exception types, cyclomatic complexity and
branch metadata, docstrings, annotations (`assumption`, `precondition`,
`postcondition`, `invariant`, `blast_radius`, `privilege_boundary`,
`taint_propagation`).

Project level: dependencies (imported packages), entrypoints with trust
levels and asset values, named subgraphs.

## Key Concepts

**Declared contract vs. effective input domain:** trailmark separates what a
function *declares* it accepts from what can *actually reach* it via call
paths. Mismatches are where vulnerabilities hide:
- **Widening**: unconstrained data reaches a function that assumes validation
- **Safe by coincidence**: no validation, but only safe callers exist today

Low-complexity functions on tainted paths are high-value targets — combine
complexity with taint and blast radius data.

Common security analysis patterns:
[references/query-patterns.md](references/query-patterns.md).
