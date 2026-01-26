# Project Organization

Directory structure and build system conventions.

## CRITICAL: cd && npx/tsc Commands Timeout

Claude Code bug: Commands like `cd directory && npx something` or `cd directory && tsc something` timeout frequently.

Workaround: Create Makefile target in parent directory.

| Wrong | Right |
|-------|-------|
| `cd web && npx tsc --noEmit` | `make web/check` |
| `cd web && npm run build` | `make web/build` |
| `cd pkg && go test ./...` | `make pkg-test` |

Makefile targets avoid the timeout bug that plagues chained cd+command patterns.

## Status Subdirectories

Large projects use subdirectories to keep top-level focused.

```
project/
├── old/           # Superseded versions
├── design/        # DESIGN_*.md, *_DESIGN.md
├── plans/         # PLAN_*.md
├── analysis/      # Research, axioms, assessments
├── reference/     # Concepts, guides, inventories
├── retro/         # Retrospectives, critiques, failures
└── [process docs] # Only active files at top level
```

Top-level files (keep at root):
- CLAUDE.md, README.md, project.md
- DATA_SOURCES.md, GLOSSARY.md, LEARNINGS.md
- TODO.md, COMPLETED.md, HUMAN_TODOS.md
- DEMO.md

Categorization:
- **old/**: V1 when V2 exists, superseded approaches
- **design/**: DESIGN_*.md, *_DESIGN.md
- **plans/**: PLAN_*.md
- **analysis/**: *_ANALYSIS.md, *_ASSESSMENT.md
- **reference/**: Concepts, how-tos, inventories
- **retro/**: FAILURES.md, *_CRITIQUE.md, *_GAPS.md

Cleanup trigger: ~50 files at top level.

## Scripts Directory

All scripts saved to `./scripts/` in project root.

```
project-root/
└── scripts/
    ├── analyze_connectors.sh
    ├── check_coverage.py
```

Rules:
- Never save to /tmp/ or ephemeral locations
- Scripts are project artifacts
- Include comment header explaining purpose

## Root Makefile (Multi-Subproject)

Root Makefile propagates targets to subprojects.

```makefile
SUBDIRS = 01-classifiers 02-egraph 03-datalog

build:
	@for dir in $(SUBDIRS); do \
		$(MAKE) -C $$dir build || exit 1; \
	done

# Specific targets
01-classifiers-build:
	$(MAKE) -C 01-classifiers build
```

When adding subprojects:
1. Add to SUBDIRS
2. Add specific targets: `<subproject>-build`, `<subproject>-test`
3. Include in aggregate targets

Argument passthrough:
```bash
make test -- -v        # Pass -v to go test
make build -- -race    # Build with race detector
```

Implementation:
```makefile
ARGS = $(filter-out $@,$(MAKECMDGOALS))

test:
	@for dir in $(SUBDIRS); do \
		$(MAKE) -C $$dir test EXTRA_FLAGS="$(ARGS)"; \
	done
```

## Build System Discipline

Never bypass root Makefile in multi-subproject repos.

| Wrong | Right |
|-------|-------|
| `cd sub && go test` | `make sub-test` |
| `cd sub && make test` | `make sub-test` |

Direct commands may hang due to toolchain config handled by root Makefile.

If commands timeout, first check: are you using root Makefile?
