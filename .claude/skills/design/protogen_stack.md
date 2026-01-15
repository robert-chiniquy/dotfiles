# Design Skill: Protogen Stack Architecture

## Purpose

Define the architectural patterns and conventions for building Go backend applications using the "Protogen Stack" - a proto-first, code-generation-driven approach to building gRPC services with pluggable persistence.

## When to Apply

- Designing new backend services in Go
- Building APIs that need strong type safety across language boundaries
- Creating multi-tenant SaaS platforms
- Architecting systems where schema evolution matters
- Projects requiring compile-time DI and testability

---

## What is the Protogen Stack?

The Protogen Stack is an architectural pattern where:

1. **Protocol Buffers define everything** - APIs, data models, database schemas, validation rules
2. **Code generation produces infrastructure** - Go/TypeScript code, HTTP gateways, validators, DB mappers
3. **Three-layer architecture separates concerns** - RPC → Controller → Database
4. **Tenant isolation is structural** - WithPassport scoping prevents cross-tenant access
5. **Compile-time dependency injection** - Wire generates type-safe initialization

**Core Philosophy:** Make the right thing easy, and the wrong thing impossible to compile.

**API Design Bias:** All API design should follow RAII principles and map closely to CRUD + List + Search patterns. Resources have clear lifecycles (Create, Get, Update, Delete), collections have List/Search operations. Avoid ad-hoc RPC methods when a resource-oriented design would work. If something feels like a "verb" RPC, ask whether it should be modeled as a sub-resource with its own lifecycle instead.

---

## Stack Components

### 1. Protocol Buffers (Single Source of Truth)

**Directory Organization:**

```
protos/
├── <app>models/         # Storage models (wire-compatible, DynamoDB/Postgres)
├── <app>api/            # Public APIs (external clients, strict compatibility)
├── <app>backend/        # Internal service contracts (service-to-service)
└── <app>runtime/        # Non-persisted objects (config, identity)
```

**Proto Annotations Drive Code Generation:**

```protobuf
// Database schema from proto
message Document {
  option (dynamo.v1.msg).key = {
    pk_fields: ["tenant_id"]
    sk_fields: ["id"]
  };
  
  option (pgdb.v1.msg).indexes = {
    name: "created_at_idx"
    method: INDEX_METHOD_BTREE
    columns: ["tenant_id", "created_at"]
  };
  
  string tenant_id = 1;
  string id = 2;
  string title = 3 [(validate.rules).string = {min_len: 1, max_len: 255}];
  google.protobuf.Timestamp created_at = 4;
}
```

**Generated Artifacts:**
- `*.pb.go` - Go message types
- `*.pb.validate.go` - Input validation
- `*.pb.dynamo.go` - DynamoDB key methods
- `*.pb.ts` - TypeScript types for frontend
- `*.pb.apigw.go` - HTTP/JSON gateway routes

### 2. Code Generation Pipeline

**Two-Stage Generation:**

```bash
# Stage 1: Proto → Code
make protogen
# Generates: Go/TS types, validators, gRPC stubs, DB mappers

# Stage 2: App-Specific
make appgen  # or <app>gen
# Generates: Service registries, OpenAPI specs, feature flags, constants
```

**Tools Used:**
- `protoc` - Proto compilation (direct, no SaaS dependency)
- `protoc-gen-go` / `protoc-gen-go-grpc` - Go code
- `protoc-gen-validate` - Validation rules
- `protoc-gen-dynamo` / `protoc-gen-pgdb` - Database annotations
- `protoc-gen-apigw` - HTTP gateway (gRPC-Gateway style)
- `protoc-gen-authz` - Authorization annotations

**Vendor Dependencies - No SaaS Tooling:**

Proto tooling should work offline. Avoid tools that require network connectivity for basic operations.

**Why vendor:**
- `buf` (Buf Schema Registry) requires internet for dependency resolution and has rate limits
- Network failures shouldn't break builds
- Reproducible builds require pinned, local dependencies
- No surprise breakages from upstream changes

**How to vendor proto dependencies:**

```bash
# Create vendor directory
mkdir -p protos/vendor

# Download common dependencies once
# googleapis (for google.protobuf.*, google.api.*, etc.)
git clone --depth 1 https://github.com/googleapis/googleapis protos/vendor/googleapis

# protovalidate (for buf.validate.*)
git clone --depth 1 https://github.com/bufbuild/protovalidate protos/vendor/protovalidate
```

**Makefile with vendored deps:**

```makefile
PROTO_VENDOR := protos/vendor
PROTO_INCLUDES := -I protos -I $(PROTO_VENDOR)/googleapis -I $(PROTO_VENDOR)/protovalidate/proto/protovalidate

.PHONY: protogen
protogen:
	protoc $(PROTO_INCLUDES) \
		--go_out=pkg/pb --go_opt=paths=source_relative \
		--go-grpc_out=pkg/pb --go-grpc_opt=paths=source_relative \
		protos/**/*.proto
```

**If using buf for linting only** (optional, not required for generation):

```yaml
# buf.yaml - local-only config, no deps
version: v2
modules:
  - path: protos
lint:
  use:
    - STANDARD
```

**Migration from buf with BSR deps:**

1. Identify deps in `buf.yaml` (e.g., `buf.build/googleapis/googleapis`)
2. Clone source repos to `protos/vendor/`
3. Update import paths if needed (usually not - paths match)
4. Replace `buf generate` with direct `protoc` calls
5. Remove BSR deps from `buf.yaml`

### 3. Three-Layer Architecture

```
┌─────────────────────────────────────┐
│  RPC Layer (pkg/api/<domain>/)     │  ← Thin: transport concerns only
│  - Parse requests                   │
│  - Extract passport from context    │
│  - Call controllers                 │
│  - Format responses                 │
└──────────────┬──────────────────────┘
               │ Delegates to
               ▼
┌─────────────────────────────────────┐
│  Controller Layer                   │  ← Thick: business logic
│  (pkg/controller/<domain>/)         │
│  - Reusable across services         │
│  - WithPassport scoping             │
│  - GetOrCreate, Mutate patterns     │
│  - DB access via drivers            │
└──────────────┬──────────────────────┘
               │ Queries/writes
               ▼
┌─────────────────────────────────────┐
│  Database Layer                     │
│  - DynamoDB (primary, key-value)    │
│  - Postgres/XPGDB (queries, joins)  │
│  - Automatic tenant isolation       │
└─────────────────────────────────────┘
```

### 4. Driver Interface Pattern

**Every controller exposes a Driver interface:**

```go
// In pkg/controller/<domain>/driver.go
type Driver interface {
    WithPassport(p *identity.Passport) Controller
    // Optional: driver-level methods (no passport needed)
}

type Controller interface {
    GetDocument(ctx context.Context, id string) (*Document, error)
    CreateDocument(ctx context.Context, doc *Document) (*Document, error)
    MutateDocument(ctx context.Context, id string, fn func(*Document) error) (*Document, error)
}

// Implementation
type driver struct {
    Tracer trace.Tracer
    DB     db.DB
    Xpgdb  *xpgdb.Driver
}

func (d *driver) WithPassport(p *identity.Passport) Controller {
    return &controller{
        driver:   d,
        passport: p,
        pg:       d.Xpgdb.WithPassport(p),
    }
}
```

**Benefits:**
- Dependency injection friendly
- Easy to mock for tests
- Compile-time interface enforcement
- Cascading passport scoping

### 5. Tenant Isolation via WithPassport

**Pattern:** All business logic operations are scoped to a tenant via passport:

```go
// In RPC handler
func (s *Service) GetDocument(ctx context.Context, req *GetDocumentRequest) (*GetDocumentResponse, error) {
    // 1. Extract passport (authn middleware already validated)
    p, err := passport.Get(ctx)
    if err != nil {
        return nil, err
    }
    
    // 2. Scope controller with passport
    controller := s.DocumentDriver.WithPassport(p)
    
    // 3. All operations inherit tenant context
    doc, err := controller.GetDocument(ctx, req.DocumentId)
    if err != nil {
        return nil, err
    }
    
    return &GetDocumentResponse{Document: doc}, nil
}
```

**Controller implementation automatically uses passport:**

```go
func (c *controller) GetDocument(ctx context.Context, id string) (*Document, error) {
    // c.passport is set by WithPassport
    return db.Get(ctx, c.driver.DB, &Document{
        TenantId: c.passport.TenantId,  // Automatic tenant filter
        Id:       id,
    })
}
```

**Security guarantee:** Cross-tenant access requires explicitly creating a differently-scoped controller (auditable, intentional).

### 6. Database Patterns

#### DynamoDB (Primary Store)

**Key Design:**
- Single-table design with composite keys
- 8-way sharding for high-throughput entities
- `pk_fields` + `sk_fields` defined in proto annotations

```protobuf
message Document {
  option (dynamo.v1.msg).key = {
    pk_fields: ["tenant_id"]  // Automatically sharded 8 ways
    sk_fields: ["id"]
  };
  
  string tenant_id = 1;
  string id = 2;
}
```

**Generated methods:**
```go
func (d *Document) PartitionKey() string  // Returns: "tenant#<id>#<shard>"
func (d *Document) SortKey() string       // Returns: "doc#<id>"
```

#### Postgres/XPGDB (Query Store)

**Purpose:** Projected from DynamoDB for complex queries, full-text search, joins.

**Schema defined in proto:**
```protobuf
message Document {
  option (pgdb.v1.msg).indexes = {
    name: "created_at"
    method: INDEX_METHOD_BTREE
    columns: ["tenant_id", "created_at"]
  };
  
  option (pgdb.v1.msg).indexes = {
    name: "search_text"
    method: INDEX_METHOD_GIN
    columns: ["search_text"]
  };
  
  string title = 3 [(pgdb.v1.options).full_text_type = FULL_TEXT_TYPE_ENGLISH];
}
```

**Projection:** DynamoDB Streams → be-db-stream → Postgres

### 7. Core Controller Patterns

#### GetOrCreate (Atomic Singleton)

```go
func GetOrCreate[T any](
    ctx context.Context,
    tracer trace.Tracer,
    p *identity.Passport,
    db db.DB,
    item T,
) (T, bool, error) {
    // 1. Try GET
    existing, err := Get(ctx, tracer, p, db, item)
    if err == nil {
        return existing, false, nil
    }
    
    // 2. If not found, CREATE with conditional insert
    if errors.Is(err, db.ErrNotFound) {
        created, err := Create(ctx, tracer, p, db, item)
        if err != nil {
            // Race: another request created it
            if errors.Is(err, dynamo.ErrConditionalCheckFailed) {
                return Get(ctx, tracer, p, db, item)
            }
            return item, false, err
        }
        return created, true, nil
    }
    
    return item, false, err
}
```

#### Mutate (Optimistic Locking)

```go
func (c *controller) MutateDocument(
    ctx context.Context,
    id string,
    editFunc func(*Document) error,
) (*Document, error) {
    const maxRetries = 3
    
    for attempt := 0; attempt < maxRetries; attempt++ {
        // 1. Get current
        doc, err := c.GetDocument(ctx, id)
        if err != nil {
            return nil, err
        }
        
        originalUpdatedAt := doc.UpdatedAt
        
        // 2. Apply edit
        if err := editFunc(doc); err != nil {
            return nil, err
        }
        
        // 3. Conditional update
        doc.UpdatedAt = timestamppb.Now()
        err = c.driver.DB.Put(ctx, doc, db.WithCondition(
            "updated_at = ?", originalUpdatedAt,
        ))
        
        if err == nil {
            return doc, nil
        }
        
        if !errors.Is(err, dynamo.ErrConditionalCheckFailed) {
            return nil, err
        }
        // Retry on conflict
    }
    
    return nil, errors.New("max retries exceeded")
}
```

### 8. Dependency Injection (Wire)

**Wire providers define dependencies:**

```go
// In pkg/controller/document/controller/document.go
func Provider(
    tracer trace.Tracer,
    db db.DB,
    xpgdb *xpgdb.Driver,
) (*driver, error) {
    return &driver{
        Tracer: tracer,
        DB:     db,
        Xpgdb:  xpgdb,
    }, nil
}
```

**Wire config aggregates providers:**

```go
// In pkg/services/pub-api/wire.go
//go:build wireinject

var ControllerSet = wire.NewSet(
    documentController.Provider,
    userController.Provider,
    // ... all controllers
)

var ServiceSet = wire.NewSet(
    rpcDocument.Provider,
    rpcUser.Provider,
    // ... all RPC services
)

func InitializeServer(ctx context.Context) (*Server, error) {
    wire.Build(
        ControllerSet,
        ServiceSet,
        DatabaseSet,
        // ...
    )
    return nil, nil
}
```

**Generated code (`wire_gen.go`):**
- Topologically sorted initialization
- Compile-time dependency checking
- No reflection, fast startup

### 9. Service Organization

**Directory Structure:**

```
pkg/
├── services/                   # Service binaries (deployables)
│   ├── pub-api/                # Public gRPC-web gateway
│   │   ├── main.go
│   │   ├── wire.go             # DI config
│   │   └── server.go
│   ├── be-conductor/           # Internal orchestration (Temporal)
│   ├── be-db-stream/           # DynamoDB Streams processor
│   └── ...
│
├── api/                        # RPC implementations (thin)
│   ├── document/
│   │   ├── rpc.go              # Service struct + Provider
│   │   ├── rpc_get.go          # Individual method
│   │   └── rpc_create.go
│   └── ...
│
├── controller/                 # Business logic (thick, reusable)
│   ├── document/
│   │   ├── driver.go           # Interface + Provider
│   │   └── controller/
│   │       ├── document.go     # Implementation
│   │       ├── search.go
│   │       └── ...
│   └── ...
│
└── pb/                         # Generated proto code (DO NOT EDIT)
    ├── <app>models/
    ├── <app>api/
    └── ...
```

**Service Prefixes:**
- `pub-*` - Public-facing (stricter validation, authn/authz)
- `be-*` - Backend/internal services
- No prefix - Shared libraries/controllers

### 10. Validation and Security

**Proto-level validation (compile-time):**

```protobuf
message CreateDocumentRequest {
  string title = 1 [(validate.rules).string = {
    min_len: 1,
    max_len: 255
  }];
  
  string content = 2 [(validate.rules).string.max_len = 100000];
  
  repeated string tags = 3 [(validate.rules).repeated = {
    min_items: 0,
    max_items: 50
  }];
}
```

**Generated validator called in interceptor:**

```go
func ValidateInterceptor() grpc.UnaryServerInterceptor {
    return func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
        if v, ok := req.(interface{ Validate() error }); ok {
            if err := v.Validate(); err != nil {
                return nil, status.Error(codes.InvalidArgument, err.Error())
            }
        }
        return handler(ctx, req)
    }
}
```

**Authorization annotations:**

```protobuf
service DocumentService {
  rpc GetDocument(GetDocumentRequest) returns (GetDocumentResponse) {
    option (authz.v1.method).required_permissions = ["document:read"];
  }
  
  rpc DeleteDocument(DeleteDocumentRequest) returns (DeleteDocumentResponse) {
    option (authz.v1.method).required_permissions = ["document:delete"];
  }
}
```

---

## Minimal Viable Protogen Stack Application

### Directory Layout

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
│   ├── design/                  # Architecture decisions, design docs
│   │   ├── DESIGN.md            # High-level architecture
│   │   ├── TODO.md              # Tracked future work
│   │   └── <feature>.md         # Per-feature design docs
│   ├── demos/                   # Runnable walkthrough docs
│   │   └── DEMO_<feature>.md    # Step-by-step demos
│   └── api/                     # API documentation (if not generated)
│       └── README.md
├── web/                         # Frontend (if applicable)
│   ├── src/
│   └── gen/                     # Generated TS from protos
├── Makefile                     # protogen, wiregen, mockgen targets
├── go.mod
└── buf.yaml                     # buf configuration
```

**Documentation Conventions:**

| Directory | Purpose | Examples |
|-----------|---------|----------|
| `docs/design/` | Architecture decisions, design rationale, TODOs | `DESIGN.md`, `TODO.md`, `patch-updates.md` |
| `docs/demos/` | Runnable walkthroughs showing features in action | `DEMO_WEB.md`, `DEMO_LLM.md` |
| `docs/api/` | API reference (if not auto-generated from protos) | OpenAPI specs, endpoint docs |

**Naming:**
- Design docs: `<FEATURE>.md` or `<FEATURE>_<ASPECT>.md` (e.g., `LLM_LAYER_DESIGN.md`)
- Demos: `DEMO_<FEATURE>.md` (uppercase prefix for visibility)
- TODOs: `TODO.md` for general, `TODO_<FEATURE>.md` for feature-specific

### Makefile Targets

```makefile
.PHONY: protogen
protogen:
	buf generate

.PHONY: wiregen
wiregen:
	go generate ./pkg/services/...

.PHONY: mockgen
mockgen:
	go generate ./pkg/controller/...

.PHONY: worldgen
worldgen: protogen wiregen mockgen

.PHONY: build
build:
	go build -o build/pub-api ./pkg/services/pub-api
```

### buf.yaml

```yaml
version: v1
lint:
  use:
    - DEFAULT
breaking:
  use:
    - FILE
```

### buf.gen.yaml

```yaml
version: v1
plugins:
  - plugin: go
    out: pkg/pb
    opt: paths=source_relative
  - plugin: go-grpc
    out: pkg/pb
    opt: paths=source_relative
  - plugin: validate
    out: pkg/pb
    opt: paths=source_relative,lang=go
```

### buf.gen.ts.yaml (TypeScript/Frontend)

For web frontends using Connect-RPC or gRPC-Web, create a separate generation config:

```yaml
version: v2
plugins:
  # Protobuf ES v1.x (class-based messages)
  - remote: buf.build/bufbuild/es:v1.10.0
    out: web/gen
    opt:
      - target=ts
      - import_extension=none  # Required for Next.js/bundlers
  
  # Connect-RPC ES v1.x (service clients)
  - remote: buf.build/connectrpc/es:v1.6.1
    out: web/gen
    opt:
      - target=ts
      - import_extension=none
```

**Important:** Match `@bufbuild/protobuf` and `@connectrpc/connect` npm package versions to the plugin versions (v1.x with v1.x, v2.x with v2.x).

### Makefile Targets

```makefile
# Port configuration (override with: make run/api API_PORT=9090)
API_PORT?=8080
WEB_PORT?=3000

.PHONY: protogen
protogen:  ## Generate Go code from protos
	buf generate

.PHONY: protogen/ts
protogen/ts:  ## Generate TypeScript code from protos
	buf generate --template buf.gen.ts.yaml

.PHONY: wiregen
wiregen:  ## Generate Wire dependency injection code
	go generate ./pkg/services/...

.PHONY: build/api
build/api: protogen  ## Build the API server
	go build -o build/api ./pkg/services/api

# Kill any existing process on the configured port before starting
# Only kills processes matching our binary name (lsof truncates names)
# If port is in use by something else, warn and exit
.PHONY: kill/api
kill/api:
	@pid=$$(lsof -i :$(API_PORT) -sTCP:LISTEN 2>/dev/null | awk '/myapp-api/ {print $$2}'); \
	if [ -n "$$pid" ]; then \
		echo "Stopping existing process (pid $$pid)"; \
		kill $$pid 2>/dev/null || true; \
		sleep 0.5; \
	else \
		existing=$$(lsof -i :$(API_PORT) -sTCP:LISTEN 2>/dev/null | tail -n +2); \
		if [ -n "$$existing" ]; then \
			echo "Warning: Port $(API_PORT) in use by another process:"; \
			echo "$$existing"; \
			echo "Use a different port: make run/api API_PORT=9090"; \
			exit 1; \
		fi; \
	fi

.PHONY: run/api
run/api: build/api kill/api  ## Build and run the API server
	PORT=$(API_PORT) ./build/api

.PHONY: web/deps
web/deps:  ## Install web frontend dependencies
	cd web && npm install

.PHONY: web/dev
web/dev:  ## Run web frontend in dev mode
	cd web && PORT=$(WEB_PORT) npm run dev

.PHONY: web/build
web/build: protogen/ts  ## Build web frontend for production
	cd web && npm run build

.PHONY: clean
clean:  ## Clean generated files and build artifacts
	rm -rf pkg/pb build/ web/.next web/gen
```

---

## Evolution Patterns

### Adding a New Entity

1. **Define model proto** (`protos/myappmodels/myapp/models/v1/entity.proto`)
2. **Define API proto** (`protos/myappapi/myapp/api/v1/entity.proto`)
3. **Run `make protogen`**
4. **Create controller** (`pkg/controller/entity/`)
5. **Create RPC handlers** (`pkg/api/entity/`)
6. **Add to Wire config** (`pkg/services/pub-api/wire.go`)
7. **Run `make wiregen`**
8. **Add tests**

### Adding a New Service

1. **Create service directory** (`pkg/services/my-service/`)
2. **Define wire.go** with required provider sets
3. **Define main.go** with InitializeServer call
4. **Add Makefile target** for building
5. **Configure deployment** (Lambda, k8s, etc.)

### Schema Migration (DynamoDB)

**Safe changes:**
- Add optional fields
- Add GSI (backfill separately)
- Deprecate fields (leave in place)

**Unsafe changes (require versioning):**
- Change field types
- Remove fields
- Change key schema

**Migration strategy:**
1. Add new field with new name
2. Dual-write to both fields
3. Backfill old data (Temporal workflow)
4. Migrate readers to new field
5. Deprecate old field

---

## Testing Patterns

### Controller Unit Tests

```go
func TestCreateDocument(t *testing.T) {
    mockDB := &mockDB{}
    ctrl := &controller{
        driver: &driver{DB: mockDB},
        passport: &identity.Passport{
            TenantId: "tenant-123",
        },
    }
    
    doc := &Document{
        TenantId: "tenant-123",
        Id:       "doc-456",
        Title:    "Test",
    }
    
    mockDB.On("Put", mock.Anything, doc).Return(nil)
    
    result, err := ctrl.CreateDocument(context.Background(), doc)
    require.NoError(t, err)
    require.Equal(t, doc.Id, result.Id)
    
    mockDB.AssertExpectations(t)
}
```

### RPC Integration Tests

```go
func TestDocumentService_GetDocument(t *testing.T) {
    // Use DynamoDB Local for realistic tests
    db := setupTestDB(t)
    defer db.Shutdown()
    
    // Seed test data
    seedDocument(t, db, &Document{
        TenantId: "tenant-123",
        Id:       "doc-456",
        Title:    "Test Doc",
    })
    
    // Create service with real dependencies
    svc := &DocumentService{
        DocumentDriver: realDocumentController(db),
    }
    
    // Test with valid passport
    ctx := passport.SetContext(context.Background(), &identity.Passport{
        TenantId: "tenant-123",
    })
    
    resp, err := svc.GetDocument(ctx, &GetDocumentRequest{
        DocumentId: "doc-456",
    })
    
    require.NoError(t, err)
    require.Equal(t, "Test Doc", resp.Document.Title)
}
```

---

## Deployment Patterns

### Lambda (Serverless)

**Package:** Single binary with HTTP gateway routes

```go
func main() {
    server, err := InitializeServer(context.Background())
    if err != nil {
        log.Fatal(err)
    }
    
    // Adapt Gin router to Lambda
    lambda.Start(func(ctx context.Context, event events.APIGatewayV2HTTPRequest) (events.APIGatewayV2HTTPResponse, error) {
        return ginLambdaAdapter(server.Router, event)
    })
}
```

**API Gateway routes to Lambda function, gRPC-Gateway translates HTTP → gRPC internally**

### Kubernetes

**Deployment:** Container with gRPC + HTTP gateway

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pub-api
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: pub-api
        image: myapp/pub-api:latest
        ports:
        - containerPort: 8080  # gRPC
        - containerPort: 8081  # HTTP gateway
        env:
        - name: DYNAMODB_ENDPOINT
          value: "dynamodb.us-west-2.amazonaws.com"
```

---

## When NOT to Use Protogen Stack

**Avoid if:**
- Extremely simple CRUD API (overhead not justified)
- Schema is highly dynamic (proto requires structure)
- Team unfamiliar with proto and unwilling to learn
- Rapid prototyping where types change hourly
- Pure event-driven architecture (though protos work great for events too)

**Alternatives:**
- REST + JSON Schema for simpler APIs
- GraphQL for client-driven queries
- Pure DynamoDB SDK for internal tools
- Firebase/Supabase for rapid MVPs

---

## Key Benefits

### Type Safety
- Compile-time checks across Go/TypeScript/Python
- Impossible to send wrong types
- IDE autocomplete for all fields

### Schema as Code
- Database schemas in version control
- No manual SQL migrations
- Code and DB cannot drift

### Security by Design
- Tenant isolation structural (WithPassport)
- Cross-tenant access requires explicit override
- Validation errors at compile-time

### Developer Velocity
- Single command regenerates everything (`make worldgen`)
- Controllers reusable across services
- Tests use same interfaces as production

### Operational Simplicity
- DynamoDB handles scaling
- Postgres projections for complex queries
- Streams-based CDC for consistency

---

## Common Pitfalls

### 1. Changing Proto Field Numbers

**NEVER:**
```protobuf
message Document {
  string id = 1;
  string title = 2;  // Changed from 3 - BREAKS STORED DATA!
}
```

**ALWAYS:**
```protobuf
message Document {
  string id = 1;
  string title = 3;
  reserved 2;  // Mark removed numbers as reserved
}
```

### 2. Forgetting to Run Protogen

```bash
# After editing ANY .proto file:
make protogen

# If changes don't appear, check:
# 1. buf.gen.yaml has correct plugins
# 2. proto import paths are correct
# 3. No syntax errors in proto (buf lint)
```

### 3. Not Using WithPassport

```go
// ❌ WRONG - No tenant scoping!
func (s *Service) BadExample(ctx context.Context, req *Request) (*Response, error) {
    // This would allow cross-tenant access
    return s.DocumentDriver.GetDocument(ctx, req.DocumentId)
}

// ✅ CORRECT - Scoped to tenant
func (s *Service) GoodExample(ctx context.Context, req *Request) (*Response, error) {
    p, _ := passport.Get(ctx)
    controller := s.DocumentDriver.WithPassport(p)
    return controller.GetDocument(ctx, req.DocumentId)
}
```

### 4. Editing Generated Files

**Generated files will be overwritten:**
- `pkg/pb/*.pb.go`
- `pkg/pb/*.pb.validate.go`
- `pkg/pb/*.pb.dynamo.go`
- `frontend/pbts/*.ts`

**Fix:** Make changes in `.proto` files, not generated code.

### 5. Skipping Validation on Public APIs

```protobuf
// ❌ WRONG - No validation
message CreateDocumentRequest {
  string title = 1;  // Could be empty, too long, etc.
}

// ✅ CORRECT - Validation rules
message CreateDocumentRequest {
  string title = 1 [(validate.rules).string = {
    min_len: 1,
    max_len: 255
  }];
}
```

---

## Further Reading

### External Resources
- [Protocol Buffers Guide](https://developers.google.com/protocol-buffers)
- [gRPC-Gateway](https://github.com/grpc-ecosystem/grpc-gateway)
- [protoc-gen-validate](https://github.com/bufbuild/protoc-gen-validate)
- [Google Wire](https://github.com/google/wire)
- [DynamoDB Best Practices](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/best-practices.html)
- [Connect-RPC](https://connectrpc.com/) - Modern gRPC for browsers and servers

### Reference Implementations

These patterns are derived from production codebases. When applying to your project,
consult local implementation notes for specific lineage and adaptations.

---

## Quick Reference

### Essential Commands

```bash
# Regenerate everything
make worldgen

# Just proto
make protogen

# Just dependency injection
make wiregen

# Just mocks
make mockgen

# Lint protos
buf lint

# Check breaking changes
buf breaking --against '.git#branch=main'
```

### Import Aliases

```go
// Models
import mduser "myapp.com/pkg/pb/myapp/models/user/v1"
import mddoc "myapp.com/pkg/pb/myapp/models/document/v1"

// APIs
import pbuser "myapp.com/pkg/pb/myapp/api/user/v1"
import pbdoc "myapp.com/pkg/pb/myapp/api/document/v1"
```

### Controller Boilerplate

```go
// driver.go
type Driver interface {
    WithPassport(p *identity.Passport) Controller
}

type Controller interface {
    GetX(ctx context.Context, id string) (*X, error)
    CreateX(ctx context.Context, x *X) (*X, error)
}

func Provider(tracer trace.Tracer, db db.DB) (*driver, error) {
    return &driver{Tracer: tracer, DB: db}, nil
}

// x.go
type driver struct {
    Tracer trace.Tracer
    DB     db.DB
}

func (d *driver) WithPassport(p *identity.Passport) Controller {
    return &controller{driver: d, passport: p}
}

type controller struct {
    driver   *driver
    passport *identity.Passport
}
```

---

## Frontend Patterns

These patterns are common in production proto-first web applications.

### Technology Stack

| Component | Choice | Notes |
|-----------|--------|-------|
| Framework | Next.js | SSR, file-based routing, pages directory |
| Language | TypeScript | Strict mode |
| UI Library | Material-UI (MUI) | `sx` prop for styles, wrapper components |
| Proto Transport | `@protobuf-ts/grpcweb-transport` | Type-safe gRPC-Web |
| Runtime | `@protobuf-ts/runtime` | Proto manipulation |
| State | Redux Toolkit | Complex state; React hooks for simple |

### Key Frontend Rules

1. **Proto objects SHOULD NOT be converted to other objects** - Use generated types directly
2. Views are "dumb" (no side effects); containers are "smart" (make gRPC calls, access globals)
3. Use wrapper components over raw MUI components for consistency
4. All copy should be internationalized (react-intl)

### Browser Console Logging

Use a simple namespaced logger for debugging. Enable via URL param or localStorage.

```typescript
// lib/log.ts
type LogLevel = "debug" | "info" | "warn" | "error";

const LEVEL_STYLES: Record<LogLevel, string> = {
  debug: "color: #888",
  info: "color: #2196f3",
  warn: "color: #ff9800",
  error: "color: #f44336; font-weight: bold",
};

function getDebugConfig(): { enabled: boolean; namespaces: Set<string> | null } {
  if (typeof window === "undefined") return { enabled: false, namespaces: null };

  // Check URL param: ?debug=true or ?debug=useSession,CellView
  const urlDebug = new URLSearchParams(window.location.search).get("debug");
  if (urlDebug !== null) {
    if (urlDebug === "true" || urlDebug === "1" || urlDebug === "") {
      return { enabled: true, namespaces: null };
    }
    return { enabled: true, namespaces: new Set(urlDebug.split(",")) };
  }

  // Fall back to localStorage.debug
  const stored = localStorage.getItem("debug");
  if (!stored) return { enabled: false, namespaces: null };
  if (stored === "true") return { enabled: true, namespaces: null };
  return { enabled: true, namespaces: new Set(stored.split(",")) };
}

function shouldLog(level: LogLevel, namespace: string): boolean {
  if (level === "warn" || level === "error") return true;
  const config = getDebugConfig();
  if (!config.enabled) return false;
  if (config.namespaces && !config.namespaces.has(namespace)) return false;
  return true;
}

function formatMessage(level: LogLevel, namespace: string, args: unknown[]): void {
  if (!shouldLog(level, namespace)) return;
  const time = new Date().toISOString().slice(11, 23);
  const method = level === "debug" ? "log" : level;
  console[method](`%c[${time}] [${namespace}]`, LEVEL_STYLES[level], ...args);
}

export const log = {
  debug: (ns: string, ...args: unknown[]) => formatMessage("debug", ns, args),
  info: (ns: string, ...args: unknown[]) => formatMessage("info", ns, args),
  warn: (ns: string, ...args: unknown[]) => formatMessage("warn", ns, args),
  error: (ns: string, ...args: unknown[]) => formatMessage("error", ns, args),
  create: (ns: string) => ({
    debug: (...args: unknown[]) => formatMessage("debug", ns, args),
    info: (...args: unknown[]) => formatMessage("info", ns, args),
    warn: (...args: unknown[]) => formatMessage("warn", ns, args),
    error: (...args: unknown[]) => formatMessage("error", ns, args),
  }),
};
```

**Usage:**

```typescript
import { log } from "@/lib/log";

// Direct logging with namespace
log.debug("useSession", "Received event:", event);

// Create namespaced logger
const logger = log.create("CellView");
logger.debug("Rendering components:", components.length);
```

**Enable debug logging:**
- URL: `?debug=true` or `?debug=useSession,CellView`
- Console: `localStorage.setItem("debug", "true")`

### gRPC-Web Transport Setup

```typescript
// lib/transport.ts
import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

export const transport = new GrpcWebFetchTransport({
  baseUrl: window.origin,
  fetchInit: { credentials: 'include' },
});
```

### RPC Pattern with Status Handling

```typescript
// lib/rpc.ts
export enum Status {
  INIT,
  LOADING,
  OK,
  ERROR,
}

export type ActionStatus<T> =
  | { status: Status.INIT }
  | { status: Status.OK; item: T }
  | { status: Status.ERROR; error: RpcError }
  | { status: Status.LOADING };

// Unary call wrapper
export const unaryInvoke = <Req, Resp>(
  input: Req,
  invokeFn: (input: Req, options?: RpcOptions) => UnaryCall<Req, Resp>,
  dispatch: Dispatch<ActionStatus<Resp>>,
) => {
  const aborter = new AbortController();
  dispatch({ status: Status.LOADING });

  invokeFn(input, { timeout: 29000, abort: aborter.signal })
    .then(resp => dispatch({ status: Status.OK, item: resp.response }))
    .catch(err => dispatch({ status: Status.ERROR, error: err }));

  return () => aborter.abort();  // Cleanup function
};
```

### WebSocket Patterns

**Notification WebSocket (broadcast changes):**

```typescript
// Singleton pattern for WebSocket client
export class WebsocketNotifyClient {
  private static instance: WebsocketNotifyClient | undefined;
  private subscriptions: Record<string, Record<string, OnNotifyEventType[]>>;

  // Subscribe to proto type changes
  subscribe(tags: string[], key: string, onNotify: OnNotifyEventType): void;
  unsubscribe(tags: string[], key: string): void;
}

// Usage: Subscribe to entity changes
notifyClient.subscribe(
  [SessionRef.typeName],
  'my-component-key',
  (ref, eventType, lastUpdatedAt) => {
    if (eventType === ChangedRefEventType.UPDATED) {
      refetchData();
    }
  }
);
```

**Chat/Streaming WebSocket (bidirectional):**

```typescript
export class WebsocketChatClient {
  private socket: WebSocket | undefined;

  async connect(): Promise<void> {
    this.socket = new WebSocket(`wss://${location.host}/ws/chat`);

    this.socket.onmessage = async (event) => {
      // Binary proto or JSON
      if (event.data instanceof Blob) {
        const buffer = await event.data.arrayBuffer();
        const message = UIMessage.fromBinary(new Uint8Array(buffer));
        this.notifySubscribers(message);
      }
    };
  }

  subscribe(conversationId: string, callback: OnMessageCallback): Promise<void>;
  unsubscribe(conversationId: string, callback: OnMessageCallback): void;
}
```

### Proto-Driven UI Pattern

**Core Insight:** Protos don't just define wire formats - they define UI contracts. A proto message can fully describe what a UI component should render, what inputs it accepts, and how to validate those inputs.

This pattern appears throughout the stack:
- **FormService** stores form schemas that frontends render dynamically
- **Agent tools** emit proto payloads that map to specific UI components  
- **Validation rules** in protos (`validate.rules`) drive both server-side and client-side validation

**The Two-Way Contract:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Proto Schema                                                    │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ message SelectField {                                       ││
│  │   repeated SelectOption options = 1;                        ││
│  │   SelectType type = 2;  // DROPDOWN, RADIO, BUTTONS         ││
│  │ }                                                           ││
│  │                                                             ││
│  │ message SelectOption {                                      ││
│  │   string display_name = 1;                                  ││
│  │   string value = 2;                                         ││
│  │   string description = 3;  // For BUTTONS display           ││
│  │ }                                                           ││
│  └─────────────────────────────────────────────────────────────┘│
│                              │                                   │
│              ┌───────────────┴───────────────┐                   │
│              ▼                               ▼                   │
│  ┌─────────────────────┐         ┌─────────────────────┐        │
│  │ Backend (Go)        │         │ Frontend (TS)       │        │
│  │                     │         │                     │        │
│  │ // Emit via RPC     │         │ // Render based on  │        │
│  │ field.SelectField = │         │ // proto field type │        │
│  │   &SelectField{     │         │ switch (field.type) │        │
│  │     Type: RADIO,    │         │   case 'selectField'│        │
│  │     Options: [...], │         │     return <Select/>│        │
│  │   }                 │         │                     │        │
│  └─────────────────────┘         └─────────────────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

**Key Properties:**

1. **Schema as Contract** - The proto defines exactly what the UI can do. A `SelectField` with `type: RADIO` tells the frontend to render radio buttons, not a dropdown. No guessing.

2. **Validation Travels with Schema** - `validate.rules` annotations define constraints that apply at both ends:
   ```protobuf
   message StringField {
     string default_value = 1;
     optional validate.StringRules rules = 2;  // min_len, max_len, pattern, etc.
   }
   ```
   Frontend renders inline validation; backend enforces on submit.

3. **View Variants via Oneof** - Same underlying data type can render differently:
   ```protobuf
   message StringField {
     oneof view {
       TextField text_field = 100;      // Input box
       PasswordField password_field = 101;  // Masked input
       SelectField select_field = 102;  // Dropdown from options
     }
   }
   ```

4. **RPCs Serve Schemas** - FormSchemaService is the canonical example:
   - `Create` / `Get` / `List` / `Update` / `Delete` for form templates
   - Frontend calls `Get` to retrieve schema, renders UI from it
   - User submits form, backend validates against same schema

**When to Use This Pattern:**

- Agent-generated UI (clarifying questions, tool outputs)
- User-configurable forms (admin-defined workflows)
- Dynamic dashboards (components driven by data)
- Any case where UI structure isn't known at compile time

**Frontend Consumption:**

```typescript
// Route to renderer based on proto field.type.oneofKind
function renderField(field: Field): JSX.Element {
  switch (field.type.oneofKind) {
    case 'stringField':
      return renderStringField(field.type.stringField);
    case 'boolField':
      return <Checkbox {...props} />;
    case 'selectField':
      return <Select options={field.type.selectField.options} />;
  }
}

function renderStringField(sf: StringField): JSX.Element {
  switch (sf.view.oneofKind) {
    case 'textField':
      return <TextField multiline={sf.view.textField.multiline} />;
    case 'passwordField':
      return <PasswordField />;
    case 'selectField':
      return <Select type={sf.view.selectField.type} />;
  }
}
```

**This is the foundation for A2UI** - agent tools emit proto payloads that frontend components consume. The proto IS the component contract.

---

### Dynamic Form Schema Pattern

For agent-generated or user-defined forms, use a schema proto that can be stored and rendered dynamically:

```protobuf
// Atomic form field types (A2UI building blocks)
message Field {
  string name = 1;           // Unique within form
  string display_name = 2;   // Label
  string description = 3;    // Helper text

  oneof type {
    StringField string_field = 100;
    BoolField bool_field = 101;
    StringSliceField string_slice_field = 102;
    Int64Field int64_field = 103;
    FileField file_field = 104;
  }

  oneof provider_config {
    UserProviderConfig user_config = 200;    // User provides value
    AdminProviderConfig admin_config = 201;  // Admin-set default
    SharedProviderConfig shared_config = 202; // User can override default
  }
}

message StringField {
  string default_value = 1;
  optional validate.StringRules rules = 2;
  string placeholder = 3;

  oneof view {
    TextField text_field = 100;      // Single/multiline input
    PasswordField password_field = 101;
    SelectField select_field = 102;  // Dropdown/radio/buttons
    PickerField picker_field = 103;  // Entity picker
  }
}

// Form is a collection of fields with relationships
message Form {
  string id = 1;
  string display_name = 2;
  repeated Field fields = 3;
  repeated FieldRelationship field_relationships = 4;  // Cross-field validation
  repeated FieldGroup field_groups = 5;                // Visual grouping
}

// FormSchema wraps Form with metadata for CRUD
message FormSchema {
  string id = 1;
  string display_name = 2;
  Form form = 3;
  google.protobuf.Timestamp created_at = 4;
  google.protobuf.Timestamp updated_at = 5;
}
```

**FormSchemaService pattern:**
- CRUD service for form templates
- Forms can be created at runtime (agent-generated, user-defined)
- A2UI atomic units map to Field type variants
- Validation rules embedded in proto, enforced at render and submit

### Ref Type Pattern for WebSocket Notifications

Define `*Ref` types for each entity to enable targeted cache invalidation:

```typescript
// Map proto types to their Ref types for notification routing
const standardRefHandlers: Record<string, MessageType<AllowedRefs>> = {
  [UserRef.typeName]: UserRef,
  [SessionRef.typeName]: SessionRef,
  [FormSchemaRef.typeName]: FormSchemaRef,
  // ... all entity refs
};

// Convert between ref string and proto for subscription matching
export const getRefString = (ref: AllowedRefs): string => {
  return Object.entries(ref)
    .map((e) => `${e[0]}:${e[1]}`)
    .join('|');
};
```

---

**Total Sections: 11**  
**Verification: Based on production proto-first codebases**
