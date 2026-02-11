# Protogen Pitfalls

Common mistakes and how to avoid them.

## 1. Changing Proto Field Numbers

Field numbers are wire format identifiers. Changing them breaks stored data.

```protobuf
// WRONG - breaks stored data
message Document {
  string id = 1;
  string title = 2;  // Was 3, now 2 - CORRUPTS DATA
}

// CORRECT
message Document {
  string id = 1;
  string title = 3;
  reserved 2;  // Mark removed numbers
}
```

## 2. Forgetting Protogen

After editing ANY .proto file:

```bash
make protogen
```

If changes don't appear:
1. Check buf.gen.yaml has correct plugins
2. Verify proto import paths
3. Run `buf lint` for syntax errors

## 3. Not Using WithPassport

```go
// WRONG - no tenant scoping, allows cross-tenant access
func (s *Service) Bad(ctx context.Context, req *Request) (*Response, error) {
    return s.DocumentDriver.GetDocument(ctx, req.DocumentId)
}

// CORRECT - scoped to tenant
func (s *Service) Good(ctx context.Context, req *Request) (*Response, error) {
    p, _ := passport.Get(ctx)
    controller := s.DocumentDriver.WithPassport(p)
    return controller.GetDocument(ctx, req.DocumentId)
}
```

## 4. Editing Generated Files

These files are overwritten on regeneration:
- `pkg/pb/*.pb.go`
- `pkg/pb/*.pb.validate.go`
- `pkg/pb/*.pb.dynamo.go`
- `frontend/pbts/*.ts`

Fix: Change `.proto` files, not generated code.

## 5. Skipping Validation on Public APIs

```protobuf
// WRONG - no validation
message CreateDocumentRequest {
  string title = 1;  // Could be empty, 10MB, anything
}

// CORRECT
message CreateDocumentRequest {
  string title = 1 [(validate.rules).string = {
    min_len: 1,
    max_len: 255
  }];
}
```

## 6. Using buf with BSR Dependencies

Network-dependent builds break in CI/air-gapped environments.

```bash
# Vendor dependencies locally
mkdir -p protos/vendor
git clone --depth 1 https://github.com/googleapis/googleapis protos/vendor/googleapis
git clone --depth 1 https://github.com/bufbuild/protovalidate protos/vendor/protovalidate
```

Use direct protoc calls instead of buf generate for production builds.

## 7. Mismatched Frontend Package Versions

`@bufbuild/protobuf` and plugin versions must match major version.

```yaml
# buf.gen.ts.yaml
plugins:
  - remote: buf.build/bufbuild/es:v1.10.0  # v1.x plugin
```

```json
// package.json
"@bufbuild/protobuf": "^1.10.0"  // Must be v1.x
```

v1 plugins with v2 runtime = type errors.

## 8. Ad-hoc RPC Methods Instead of Resources

```protobuf
// WRONG - verb-based RPC
rpc ActivateUser(ActivateUserRequest) returns (ActivateUserResponse);
rpc DeactivateUser(DeactivateUserRequest) returns (DeactivateUserResponse);

// CORRECT - resource with lifecycle
message User {
  UserStatus status = 5;  // ACTIVE, INACTIVE
}
rpc UpdateUser(UpdateUserRequest) returns (UpdateUserResponse);
```

Model state changes as resource updates, not separate verbs.
