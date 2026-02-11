# Protogen Database Patterns

DynamoDB as primary store, Postgres for queries, SQLite for local/testing.

## DynamoDB (Primary Store)

Single-table design with composite keys. 8-way sharding for high-throughput entities.

```protobuf
message Document {
  option (dynamo.v1.msg).key = {
    pk_fields: ["tenant_id"]  # Automatically sharded 8 ways
    sk_fields: ["id"]
  };

  string tenant_id = 1;
  string id = 2;
}
```

Generated methods:

```go
func (d *Document) PartitionKey() string  // Returns: "tenant#<id>#<shard>"
func (d *Document) SortKey() string       // Returns: "doc#<id>"
```

## Postgres/XPGDB (Query Store)

Projected from DynamoDB for complex queries, full-text search, joins.

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

Projection pipeline: DynamoDB Streams -> be-db-stream -> Postgres

## SQLite (Local/Embedded/Testing)

Preferred for local filesystem databases.

Use cases:
- Local development
- Embedded databases in CLI tools
- Test fixtures
- Single-file data stores
- Read-heavy workloads

Why SQLite:
- Zero config, single-file
- Excellent read performance
- Built-in FTS5
- Portable, reliable

When NOT SQLite:
- High write concurrency -> Postgres
- Distributed systems -> DynamoDB/Postgres
- Production multi-tenant -> DynamoDB + Postgres

Testing pattern:

```go
func TestWithSQLite(t *testing.T) {
    db, _ := sql.Open("sqlite3", ":memory:")
    // Run tests with in-memory SQLite
}
```

## Schema Migration (DynamoDB)

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
