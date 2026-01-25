# Protogen Controller Patterns

Core patterns: GetOrCreate, Mutate, Wire DI.

## GetOrCreate (Atomic Singleton)

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

Returns: (item, wasCreated, error)

## Mutate (Optimistic Locking)

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

Pattern: read-modify-write with version check. Retries on conflict.

## Wire Dependency Injection

Providers define dependencies:

```go
// pkg/controller/document/controller/document.go
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

Wire config aggregates providers:

```go
// pkg/services/pub-api/wire.go
//go:build wireinject

var ControllerSet = wire.NewSet(
    documentController.Provider,
    userController.Provider,
)

var ServiceSet = wire.NewSet(
    rpcDocument.Provider,
    rpcUser.Provider,
)

func InitializeServer(ctx context.Context) (*Server, error) {
    wire.Build(
        ControllerSet,
        ServiceSet,
        DatabaseSet,
    )
    return nil, nil
}
```

Generated `wire_gen.go`:
- Topologically sorted initialization
- Compile-time dependency checking
- No reflection, fast startup

## Validation Interceptor

Proto validation called in gRPC interceptor:

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

## Authorization Annotations

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
