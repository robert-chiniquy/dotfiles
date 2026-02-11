# Protogen Architecture

Three-layer architecture with tenant isolation via WithPassport.

## Three Layers

```
RPC Layer (pkg/api/<domain>/)      <- Thin: transport only
  - Parse requests
  - Extract passport from context
  - Call controllers
  - Format responses
           |
           v
Controller Layer (pkg/controller/<domain>/)  <- Thick: business logic
  - Reusable across services
  - WithPassport scoping
  - GetOrCreate, Mutate patterns
  - DB access via drivers
           |
           v
Database Layer
  - DynamoDB (primary, key-value)
  - Postgres/XPGDB (queries, joins)
  - Automatic tenant isolation
```

## Driver Interface Pattern

Every controller exposes Driver + Controller interfaces:

```go
// pkg/controller/<domain>/driver.go

type Driver interface {
    WithPassport(p *identity.Passport) Controller
}

type Controller interface {
    GetDocument(ctx context.Context, id string) (*Document, error)
    CreateDocument(ctx context.Context, doc *Document) (*Document, error)
    MutateDocument(ctx context.Context, id string, fn func(*Document) error) (*Document, error)
}

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

Benefits:
- DI friendly
- Easy to mock
- Compile-time interface enforcement
- Cascading passport scoping

## Tenant Isolation (WithPassport)

All operations scoped to tenant via passport:

```go
// RPC handler
func (s *Service) GetDocument(ctx context.Context, req *GetDocumentRequest) (*GetDocumentResponse, error) {
    // 1. Extract passport (authn middleware validated)
    p, err := passport.Get(ctx)
    if err != nil {
        return nil, err
    }

    // 2. Scope controller
    controller := s.DocumentDriver.WithPassport(p)

    // 3. All operations inherit tenant context
    doc, err := controller.GetDocument(ctx, req.DocumentId)
    if err != nil {
        return nil, err
    }

    return &GetDocumentResponse{Document: doc}, nil
}
```

Controller automatically uses passport:

```go
func (c *controller) GetDocument(ctx context.Context, id string) (*Document, error) {
    return db.Get(ctx, c.driver.DB, &Document{
        TenantId: c.passport.TenantId,  // Automatic tenant filter
        Id:       id,
    })
}
```

Security: Cross-tenant access requires explicitly creating differently-scoped controller (auditable, intentional).

## Controller Boilerplate

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

## Service Prefixes

- `pub-*` - Public-facing (stricter validation, authn/authz)
- `be-*` - Backend/internal services
- No prefix - Shared libraries/controllers
