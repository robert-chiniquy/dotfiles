# Protogen Stack Overview

Proto-first architecture for Go backends with TypeScript frontends. Protocol Buffers define everything; code generation produces infrastructure.

## Core Components

1. **Protocol Buffers** - Single source of truth for APIs, models, DB schemas, validation
2. **Code generation** - Go/TS types, validators, gRPC stubs, DB mappers
3. **Three-layer architecture** - RPC (thin) -> Controller (thick) -> Database
4. **Tenant isolation** - WithPassport scoping prevents cross-tenant access
5. **Compile-time DI** - Wire generates type-safe initialization

## Philosophy

Make the right thing easy, the wrong thing impossible to compile.

## API Design Bias

CRUD + List + Search patterns. Resources have lifecycles (Create, Get, Update, Delete). Collections have List/Search. Avoid ad-hoc RPC verbs - model as sub-resources with lifecycles instead.

## When to Use

- Go backend services
- Type safety across language boundaries
- Multi-tenant SaaS
- Schema evolution matters
- Compile-time DI and testability needed

## When NOT to Use

- Simple CRUD API (overhead not justified)
- Highly dynamic schema (proto requires structure)
- Team won't learn proto
- Rapid prototyping with hourly type changes
- Pure event-driven architecture (though protos work for events)

## Alternatives

| Need | Use |
|------|-----|
| Simpler APIs | REST + JSON Schema |
| Client-driven queries | GraphQL |
| Internal tools | Pure DynamoDB SDK |
| Rapid MVP | Firebase/Supabase |

## Related Skills

- `proto-schema.md` - Proto organization, annotations, codegen
- `proto-architecture.md` - Three layers, Driver pattern, WithPassport
- `proto-database.md` - DynamoDB, Postgres, SQLite patterns
- `proto-patterns.md` - GetOrCreate, Mutate, Wire DI
- `proto-project.md` - Directory structure, Makefile
- `proto-testing.md` - Unit and integration tests
- `proto-frontend.md` - Transport, WebSocket, Proto-Driven UI
- `proto-pitfalls.md` - Common mistakes
