# Protogen Project Structure

Directory layout and Makefile conventions.

## CRITICAL: cd && npx/tsc Commands Timeout

Claude Code bug: Commands like `cd directory && npx something` or `cd directory && tsc something` timeout frequently.

Workaround: Create Makefile target in parent directory.

| Wrong | Right |
|-------|-------|
| `cd web && npx tsc --noEmit` | `make web/check` |
| `cd web && npm run build` | `make web/build` |
| `cd pkg && go test ./...` | `make pkg-test` |

Makefile targets avoid the timeout bug that plagues chained cd+command patterns.

## Directory Layout

```
my-app/
├── protos/
│   ├── myappmodels/myapp/models/v1/
│   │   └── document.proto
│   └── myappapi/myapp/api/v1/
│       └── document.proto
├── pkg/
│   ├── pb/                      # Generated (make protogen)
│   ├── controller/
│   │   └── document/
│   │       ├── driver.go
│   │       └── controller/
│   │           └── document.go
│   ├── api/
│   │   └── document/
│   │       └── rpc.go
│   └── services/
│       └── pub-api/
│           ├── main.go
│           ├── wire.go
│           └── server.go
├── docs/
│   ├── design/                  # Architecture decisions
│   └── demos/                   # Runnable walkthroughs
├── web/                         # Frontend
│   ├── src/
│   └── gen/                     # Generated TS from protos
├── Makefile
├── go.mod
└── buf.yaml
```

## Service Directory Structure

```
pkg/
├── services/                   # Deployables
│   ├── pub-api/                # Public gRPC-web gateway
│   │   ├── main.go
│   │   ├── wire.go
│   │   └── server.go
│   ├── be-conductor/           # Internal orchestration
│   └── be-db-stream/           # DynamoDB Streams processor
│
├── api/                        # RPC implementations (thin)
│   └── document/
│       ├── rpc.go              # Service struct + Provider
│       ├── rpc_get.go
│       └── rpc_create.go
│
├── controller/                 # Business logic (thick)
│   └── document/
│       ├── driver.go           # Interface + Provider
│       └── controller/
│           ├── document.go
│           └── search.go
│
└── pb/                         # Generated (DO NOT EDIT)
```

## Standard Makefile Targets

All subprojects MUST implement:

| Target | Command |
|--------|---------|
| `build` | `go build ./...` |
| `test` | `go test ./...` |
| `bench` | `go test -bench=. -benchmem ./...` |
| `fmt` | `go fmt ./...` |
| `lint` | `go vet ./...` |
| `clean` | `go clean ./...` |
| `tidy` | `go mod tidy` |
| `generate` | `go generate ./...` |

## Root Makefile Pattern

Delegates to subprojects:

```makefile
SUBPROJECTS := 01-foo 02-bar 03-baz

.PHONY: build test tidy generate

build:
	@for dir in $(SUBPROJECTS); do \
		$(MAKE) -C $$dir build || exit 1; \
	done

test:
	@for dir in $(SUBPROJECTS); do \
		$(MAKE) -C $$dir test || exit 1; \
	done

# Shorthand: make 01-build, make 02-test
01-build 01-test:
	$(MAKE) -C 01-foo $(subst 01-,,$@)
```

## Protogen Targets

```makefile
API_PORT?=8080
WEB_PORT?=3000

.PHONY: protogen protogen/ts wiregen worldgen

protogen:
	buf generate

protogen/ts:
	buf generate --template buf.gen.ts.yaml

wiregen:
	go generate ./pkg/services/...

mockgen:
	go generate ./pkg/controller/...

worldgen: protogen wiregen mockgen

build/api: protogen
	go build -o build/api ./pkg/services/api
```

## Port Management

Kill stale processes before starting:

```makefile
.PHONY: kill/api
kill/api:
	@pid=$$(lsof -i :$(API_PORT) -sTCP:LISTEN 2>/dev/null | awk '/myapp-api/ {print $$2}'); \
	if [ -n "$$pid" ]; then \
		kill $$pid 2>/dev/null || true; \
		sleep 0.5; \
	fi

.PHONY: run/api
run/api: build/api kill/api
	PORT=$(API_PORT) ./build/api
```

## Adding New Entity

1. Define model proto (`protos/myappmodels/.../entity.proto`)
2. Define API proto (`protos/myappapi/.../entity.proto`)
3. `make protogen`
4. Create controller (`pkg/controller/entity/`)
5. Create RPC handlers (`pkg/api/entity/`)
6. Add to Wire config (`pkg/services/pub-api/wire.go`)
7. `make wiregen`
8. Add tests

## Adding New Service

1. Create directory (`pkg/services/my-service/`)
2. Define wire.go with provider sets
3. Define main.go with InitializeServer
4. Add Makefile target
5. Configure deployment
