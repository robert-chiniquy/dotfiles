# Protogen Testing Patterns

Unit tests for controllers, integration tests for RPC.

## Controller Unit Tests

Mock the database, test business logic:

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

## RPC Integration Tests

Use DynamoDB Local or SQLite for realistic tests:

```go
func TestDocumentService_GetDocument(t *testing.T) {
    db := setupTestDB(t)
    defer db.Shutdown()

    // Seed test data
    seedDocument(t, db, &Document{
        TenantId: "tenant-123",
        Id:       "doc-456",
        Title:    "Test Doc",
    })

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

## SQLite for Tests

In-memory SQLite for fast, isolated tests:

```go
func TestWithSQLite(t *testing.T) {
    db, _ := sql.Open("sqlite3", ":memory:")
    // Tests run against in-memory DB
    // No cleanup needed - gone when test ends
}
```

## Test Tenant Isolation

Verify cross-tenant access fails:

```go
func TestCrossTenantAccessDenied(t *testing.T) {
    db := setupTestDB(t)

    // Create doc in tenant-A
    seedDocument(t, db, &Document{
        TenantId: "tenant-A",
        Id:       "doc-123",
    })

    // Try to access from tenant-B
    ctx := passport.SetContext(context.Background(), &identity.Passport{
        TenantId: "tenant-B",
    })

    svc := &DocumentService{DocumentDriver: realDocumentController(db)}
    _, err := svc.GetDocument(ctx, &GetDocumentRequest{DocumentId: "doc-123"})

    require.Error(t, err)  // Should fail - wrong tenant
}
```

## Mock Generation

Generate mocks from interfaces:

```go
//go:generate mockgen -source=driver.go -destination=mock_driver.go -package=document
```

Run with `make mockgen`.
