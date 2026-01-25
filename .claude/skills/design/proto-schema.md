# Protogen Schema & Codegen

Proto organization, annotations, and code generation pipeline.

## Directory Organization

```
protos/
├── <app>models/         # Storage models (wire-compatible, DynamoDB/Postgres)
├── <app>api/            # Public APIs (external clients, strict compatibility)
├── <app>backend/        # Internal service contracts (service-to-service)
└── <app>runtime/        # Non-persisted objects (config, identity)
```

## Proto Annotations

Annotations drive code generation for DB schemas, validation, authorization:

```protobuf
message Document {
  // DynamoDB key schema
  option (dynamo.v1.msg).key = {
    pk_fields: ["tenant_id"]
    sk_fields: ["id"]
  };

  // Postgres index
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

## Generated Artifacts

| File | Content |
|------|---------|
| `*.pb.go` | Go message types |
| `*.pb.validate.go` | Input validation |
| `*.pb.dynamo.go` | DynamoDB key methods |
| `*.pb.ts` | TypeScript types |
| `*.pb.apigw.go` | HTTP/JSON gateway routes |

## Two-Stage Generation

```bash
# Stage 1: Proto -> Code
make protogen
# Generates: Go/TS types, validators, gRPC stubs, DB mappers

# Stage 2: App-Specific
make appgen
# Generates: Service registries, OpenAPI specs, feature flags
```

## Tools

| Tool | Purpose |
|------|---------|
| `protoc` | Proto compilation (direct, no SaaS) |
| `protoc-gen-go` | Go code |
| `protoc-gen-go-grpc` | gRPC stubs |
| `protoc-gen-validate` | Validation rules |
| `protoc-gen-dynamo` | DynamoDB annotations |
| `protoc-gen-pgdb` | Postgres annotations |
| `protoc-gen-apigw` | HTTP gateway |
| `protoc-gen-authz` | Authorization annotations |

## Vendoring Dependencies

Avoid network-dependent builds. Vendor proto dependencies locally:

```bash
mkdir -p protos/vendor
git clone --depth 1 https://github.com/googleapis/googleapis protos/vendor/googleapis
git clone --depth 1 https://github.com/bufbuild/protovalidate protos/vendor/protovalidate
```

Makefile with vendored deps:

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

## buf.yaml (Local Only)

If using buf for linting (not generation):

```yaml
version: v2
modules:
  - path: protos
lint:
  use:
    - STANDARD
```

## buf.gen.yaml

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

## TypeScript Generation (buf.gen.ts.yaml)

```yaml
version: v2
plugins:
  - remote: buf.build/bufbuild/es:v1.10.0
    out: web/gen
    opt:
      - target=ts
      - import_extension=none  # Required for Next.js/bundlers

  - remote: buf.build/connectrpc/es:v1.6.1
    out: web/gen
    opt:
      - target=ts
      - import_extension=none
```

Match npm package versions to plugin versions (v1.x with v1.x).

## Import Aliases Convention

```go
// Models
import mduser "myapp.com/pkg/pb/myapp/models/user/v1"
import mddoc "myapp.com/pkg/pb/myapp/models/document/v1"

// APIs
import pbuser "myapp.com/pkg/pb/myapp/api/user/v1"
import pbdoc "myapp.com/pkg/pb/myapp/api/document/v1"
```
