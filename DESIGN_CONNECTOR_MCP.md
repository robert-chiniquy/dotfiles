# Design: Connector Development MCP Tools

Extend the c1 MCP infrastructure to support local connector development workflows.

## Problem

Connector development involves a feedback loop:

1. Write connector code
2. Run sync locally
3. Inspect output (c1z file)
4. Debug issues
5. Repeat

Currently this loop requires:
- Manual CLI invocations (`baton-okta sync`, `baton-okta validate`)
- External tools to inspect c1z files
- Context switching between IDE and terminal
- No structured way for AI assistants to help with connector development

The c1 MCP infrastructure already provides tools for **production** connector investigation (`SupportConnectorService`). This design extends that infrastructure to support **local development**.

## Proposal

Add MCP tool support to the baton-sdk so that every connector built with the SDK can expose development tools via MCP stdio transport.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         c1 MCP (Hosted)                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ SupportConnector│  │   AppService    │  │  PolicyAgent    │ │
│  │    Service      │  │                 │  │    Service      │ │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘ │
│           │                    │                    │          │
│           └────────────────────┴────────────────────┘          │
│                              │                                  │
│                       SSE Transport                             │
│                       (mcp.conductorone.com)                    │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    Connector Dev MCP (Local)                    │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ ConnectorDev    │  │   C1ZInspector  │  │  SyncPreview    │ │
│  │    Service      │  │     Service     │  │    Service      │ │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘ │
│           │                    │                    │          │
│           └────────────────────┴────────────────────┘          │
│                              │                                  │
│                       stdio Transport                           │
│                       (baton-<connector> --mcp)                 │
└─────────────────────────────────────────────────────────────────┘

Both use:
  - protoc-gen-mcpgw for code generation
  - mcpgw.v1 proto annotations
  - Same JSON Schema format for tool inputs
```

### Integration Points

| Component | Location | Purpose |
|-----------|----------|---------|
| Proto annotations | `vendor_mcpgw/mcpgw/v1/mcpgw.proto` | Tool metadata (title, description, hints) |
| Code generator | `protoc-gen-mcpgw` | Generates `.pb.mcpgw.go` from protos |
| SDK runtime | `baton-sdk/pkg/mcp/` | MCP server implementation |
| CLI flag | `baton-sdk/pkg/cli/` | `--mcp serve` command |

## Tool Specifications

### ConnectorDevService

Introspect connector capabilities and configuration.

#### GetMetadata

Returns connector metadata, resource types, and capabilities.

```proto
rpc GetMetadata(GetMetadataRequest) returns (GetMetadataResponse) {
  option (mcpgw.v1.method) = {
    title: "GetConnectorMetadata"
    description: "Get connector capabilities, resource types, config schema, and supported operations. Use this first when working with a connector."
    read_only_hint: true
  };
}

message GetMetadataRequest {}

message GetMetadataResponse {
  // Connector identity
  string name = 1 [(mcpgw.v1.field) = {
    description: "Connector name (e.g., 'baton-okta')"
  }];
  string description = 2 [(mcpgw.v1.field) = {
    description: "Human-readable connector description"
  }];

  // Resource types this connector syncs
  repeated ResourceTypeInfo resource_types = 10 [(mcpgw.v1.field) = {
    description: "Resource types with their capabilities"
  }];

  // Configuration schema
  repeated ConfigField config_schema = 20 [(mcpgw.v1.field) = {
    description: "Required and optional configuration fields"
  }];

  // Connector capabilities
  repeated string capabilities = 30 [(mcpgw.v1.field) = {
    description: "Supported capabilities: sync, provision, targeted_sync, event_feed, ticketing, actions"
  }];
}

message ResourceTypeInfo {
  string id = 1 [(mcpgw.v1.field) = {
    description: "Resource type ID (e.g., 'user', 'group', 'role')"
  }];
  string display_name = 2;
  repeated string traits = 3 [(mcpgw.v1.field) = {
    description: "Resource traits: user, group, role, app"
  }];
  repeated string capabilities = 4 [(mcpgw.v1.field) = {
    description: "Per-resource capabilities: sync, provision, targeted_sync"
  }];
}

message ConfigField {
  string name = 1;
  string description = 2;
  string type = 3 [(mcpgw.v1.field) = {
    description: "Field type: string, bool, int, secret"
  }];
  bool required = 4;
  string default_value = 5;
  string env_var = 6 [(mcpgw.v1.field) = {
    description: "Environment variable name if applicable"
  }];
}
```

#### ValidateConfig

Validate connector configuration and credentials.

```proto
rpc ValidateConfig(ValidateConfigRequest) returns (ValidateConfigResponse) {
  option (mcpgw.v1.method) = {
    title: "ValidateConnectorConfig"
    description: "Validate connector configuration. Tests API connectivity, credential validity, and required permissions. Returns specific errors for each validation failure."
    read_only_hint: true
  };
}

message ValidateConfigRequest {}

message ValidateConfigResponse {
  bool valid = 1;
  repeated ValidationIssue issues = 2;
}

message ValidationIssue {
  ValidationSeverity severity = 1;
  string category = 2 [(mcpgw.v1.field) = {
    description: "Issue category: config, credentials, permissions, connectivity"
  }];
  string field = 3 [(mcpgw.v1.field) = {
    description: "Config field name if applicable"
  }];
  string message = 4;
  string suggestion = 5 [(mcpgw.v1.field) = {
    description: "Suggested fix"
  }];
}

enum ValidationSeverity {
  VALIDATION_SEVERITY_UNSPECIFIED = 0;
  VALIDATION_SEVERITY_ERROR = 1;
  VALIDATION_SEVERITY_WARNING = 2;
  VALIDATION_SEVERITY_INFO = 3;
}
```

### C1ZInspectorService

Query and analyze c1z sync output files.

#### ListResources

Query resources from a c1z file.

```proto
rpc ListResources(ListResourcesRequest) returns (ListResourcesResponse) {
  option (mcpgw.v1.method) = {
    title: "ListResources"
    description: "List resources from a c1z sync output file. Filter by resource type, search by display name. Returns resource details including traits and parent relationships."
    read_only_hint: true
  };
}

message ListResourcesRequest {
  string c1z_path = 1 [(mcpgw.v1.field) = {
    description: "Path to c1z file (default: ./sync-output.c1z)"
  }];

  string resource_type = 2 [(mcpgw.v1.field) = {
    description: "Filter by resource type ID (e.g., 'user', 'group')"
  }];

  string query = 3 [(mcpgw.v1.field) = {
    description: "Search query for display name (case-insensitive substring match)"
  }];

  int32 limit = 4 [(mcpgw.v1.field) = {
    description: "Maximum results (default 50, max 500)"
  }];

  string page_token = 5;
}

message ListResourcesResponse {
  repeated ResourceSummary resources = 1;
  string next_page_token = 2;
  int32 total_count = 3;
}

message ResourceSummary {
  string id = 1;
  string resource_type = 2;
  string display_name = 3;
  string description = 4;
  repeated string traits = 5;
  string parent_resource_id = 6;
  string parent_resource_type = 7;
  map<string, string> profile = 8 [(mcpgw.v1.field) = {
    description: "Resource profile attributes"
  }];
}
```

#### ListGrants

Query grants from a c1z file.

```proto
rpc ListGrants(ListGrantsRequest) returns (ListGrantsResponse) {
  option (mcpgw.v1.method) = {
    title: "ListGrants"
    description: "List grants from a c1z sync output. Filter by principal, entitlement, or resource. Shows who has access to what."
    read_only_hint: true
  };
}

message ListGrantsRequest {
  string c1z_path = 1;

  string principal_id = 2 [(mcpgw.v1.field) = {
    description: "Filter by principal (user/group) resource ID"
  }];

  string entitlement_id = 3 [(mcpgw.v1.field) = {
    description: "Filter by entitlement ID"
  }];

  string resource_id = 4 [(mcpgw.v1.field) = {
    description: "Filter by resource ID (returns grants to entitlements on this resource)"
  }];

  int32 limit = 5;
  string page_token = 6;
}

message ListGrantsResponse {
  repeated GrantSummary grants = 1;
  string next_page_token = 2;
  int32 total_count = 3;
}

message GrantSummary {
  string grant_id = 1;

  // Principal (who has access)
  string principal_id = 2;
  string principal_type = 3;
  string principal_display_name = 4;

  // Entitlement (what access)
  string entitlement_id = 5;
  string entitlement_display_name = 6;
  string entitlement_slug = 7;

  // Resource (on what)
  string resource_id = 8;
  string resource_type = 9;
  string resource_display_name = 10;
}
```

#### ListEntitlements

Query entitlements from a c1z file.

```proto
rpc ListEntitlements(ListEntitlementsRequest) returns (ListEntitlementsResponse) {
  option (mcpgw.v1.method) = {
    title: "ListEntitlements"
    description: "List entitlements from a c1z sync output. Filter by resource or search by name. Shows available permissions and roles."
    read_only_hint: true
  };
}

message ListEntitlementsRequest {
  string c1z_path = 1;

  string resource_id = 2 [(mcpgw.v1.field) = {
    description: "Filter by resource ID"
  }];

  string resource_type = 3 [(mcpgw.v1.field) = {
    description: "Filter by resource type"
  }];

  string query = 4 [(mcpgw.v1.field) = {
    description: "Search query for display name"
  }];

  int32 limit = 5;
  string page_token = 6;
}

message ListEntitlementsResponse {
  repeated EntitlementSummary entitlements = 1;
  string next_page_token = 2;
  int32 total_count = 3;
}

message EntitlementSummary {
  string id = 1;
  string display_name = 2;
  string description = 3;
  string slug = 4;
  string resource_id = 5;
  string resource_type = 6;
  string resource_display_name = 7;
  bool grantable = 8;
  bool revocable = 9;
  int32 grant_count = 10 [(mcpgw.v1.field) = {
    description: "Number of grants to this entitlement"
  }];
}
```

#### GetSyncStats

Get summary statistics from a c1z file.

```proto
rpc GetSyncStats(GetSyncStatsRequest) returns (GetSyncStatsResponse) {
  option (mcpgw.v1.method) = {
    title: "GetSyncStats"
    description: "Get summary statistics from a c1z sync output. Shows counts by resource type, total grants, and sync metadata."
    read_only_hint: true
  };
}

message GetSyncStatsRequest {
  string c1z_path = 1;
}

message GetSyncStatsResponse {
  // Sync metadata
  google.protobuf.Timestamp started_at = 1;
  google.protobuf.Timestamp completed_at = 2;
  int64 duration_seconds = 3;

  // Totals
  int64 total_resources = 10;
  int64 total_entitlements = 11;
  int64 total_grants = 12;

  // Per-resource-type breakdown
  repeated ResourceTypeStats resource_type_stats = 20;
}

message ResourceTypeStats {
  string resource_type = 1;
  string display_name = 2;
  int64 resource_count = 3;
  int64 entitlement_count = 4;
  int64 grant_count = 5;
}
```

### SyncService

Run and manage sync operations.

#### RunSync

Execute a sync and return results.

```proto
rpc RunSync(RunSyncRequest) returns (RunSyncResponse) {
  option (mcpgw.v1.method) = {
    title: "RunSync"
    description: "Run a full sync and write output to a c1z file. Returns sync statistics and any errors encountered."
  };
}

message RunSyncRequest {
  string output_path = 1 [(mcpgw.v1.field) = {
    description: "Path for c1z output (default: ./sync-output.c1z)"
  }];

  repeated string resource_types = 2 [(mcpgw.v1.field) = {
    description: "Limit sync to specific resource types (empty = all)"
  }];
}

message RunSyncResponse {
  string c1z_path = 1;
  GetSyncStatsResponse stats = 2;
  repeated SyncError errors = 3;
  repeated SyncWarning warnings = 4;
}

message SyncError {
  string resource_type = 1;
  string phase = 2 [(mcpgw.v1.field) = {
    description: "Sync phase: list, get, grants"
  }];
  string message = 3;
  string details = 4;
}

message SyncWarning {
  string resource_type = 1;
  string message = 2;
  int32 count = 3 [(mcpgw.v1.field) = {
    description: "Number of occurrences"
  }];
}
```

#### DiffSyncs

Compare two sync outputs.

```proto
rpc DiffSyncs(DiffSyncsRequest) returns (DiffSyncsResponse) {
  option (mcpgw.v1.method) = {
    title: "DiffSyncs"
    description: "Compare two c1z sync outputs. Shows added, removed, and changed resources, entitlements, and grants."
    read_only_hint: true
  };
}

message DiffSyncsRequest {
  string before_path = 1 [(mcpgw.v1.field) = {
    description: "Path to older c1z file"
    required: true
  }];

  string after_path = 2 [(mcpgw.v1.field) = {
    description: "Path to newer c1z file"
    required: true
  }];

  string resource_type = 3 [(mcpgw.v1.field) = {
    description: "Filter diff to specific resource type"
  }];
}

message DiffSyncsResponse {
  // Summary
  int32 resources_added = 1;
  int32 resources_removed = 2;
  int32 resources_changed = 3;
  int32 grants_added = 4;
  int32 grants_removed = 5;

  // Details (limited to first N of each category)
  repeated ResourceDiff resource_diffs = 10;
  repeated GrantDiff grant_diffs = 11;
}

message ResourceDiff {
  DiffType diff_type = 1;
  string resource_type = 2;
  string resource_id = 3;
  string display_name = 4;
  repeated FieldDiff field_changes = 5 [(mcpgw.v1.field) = {
    description: "Changed fields (for CHANGED type)"
  }];
}

message GrantDiff {
  DiffType diff_type = 1;
  GrantSummary grant = 2;
}

message FieldDiff {
  string field = 1;
  string before = 2;
  string after = 3;
}

enum DiffType {
  DIFF_TYPE_UNSPECIFIED = 0;
  DIFF_TYPE_ADDED = 1;
  DIFF_TYPE_REMOVED = 2;
  DIFF_TYPE_CHANGED = 3;
}
```

## SDK Changes

### New Package: `pkg/mcp/`

```
baton-sdk/pkg/mcp/
├── server.go           # MCP server implementation
├── connector_dev.go    # ConnectorDevService implementation
├── c1z_inspector.go    # C1ZInspectorService implementation
├── sync_service.go     # SyncService implementation
└── transport/
    └── stdio.go        # stdio transport for local MCP
```

### CLI Changes

Add `--mcp` flag to connector CLI:

```go
// pkg/cli/cli.go

type MCPOpts struct {
    Enabled bool
    // Future: could add socket path, HTTP port for alternative transports
}

func WithMCPServer() RunOption {
    return func(o *runOptions) {
        o.mcpEnabled = true
    }
}
```

Usage:
```bash
# Start MCP server (blocks, serves via stdio)
baton-okta --mcp serve

# Alternative: run sync with MCP server available
baton-okta --mcp sync
```

### Proto Changes

New proto file at `proto/c1/connector/mcp/v1/`:

```
baton-sdk/proto/
└── c1/
    └── connector/
        └── mcp/
            └── v1/
                ├── connector_dev.proto
                ├── c1z_inspector.proto
                └── sync.proto
```

### buf.gen.yaml Changes

```yaml
# Add mcpgw plugin
plugins:
  # ... existing plugins ...
  - local: protoc-gen-mcpgw
    out: pb
    opt:
      - lang=go
      - paths=source_relative
```

### Dependencies

Add to `go.mod`:
```
require (
    github.com/ductone/protoc-gen-mcpgw v0.1.0
)
```

## Usage Examples

### Claude Code Configuration

```json
// ~/.claude/mcp.json
{
  "mcpServers": {
    "baton-okta": {
      "command": "./baton-okta",
      "args": ["--mcp", "serve"],
      "env": {
        "BATON_DOMAIN": "${OKTA_DOMAIN}",
        "BATON_API_TOKEN": "${OKTA_TOKEN}"
      }
    }
  }
}
```

### Workflow: Investigating Sync Issues

```
User: "The Okta connector isn't syncing groups correctly"

Claude: [calls GetConnectorMetadata]
"The connector supports user, group, role, and app resource types.
Groups have sync and provision capabilities."

Claude: [calls RunSync with resource_types=["group"]]
"Sync completed. Found 142 groups, 89 entitlements, 1,247 grants."

Claude: [calls ListResources with resource_type="group", query="engineering"]
"Found 3 groups matching 'engineering':
- Engineering (12 members)
- Engineering-Leads (4 members)
- Engineering-Contractors (0 members)"

Claude: [calls ListGrants with resource_id="<engineering-group-id>"]
"Engineering group has 12 grants..."
```

### Workflow: Comparing Sync Outputs

```
User: "What changed since last week's sync?"

Claude: [calls DiffSyncs with before="sync-2025-01-23.c1z", after="sync-2025-01-30.c1z"]
"Changes detected:
- 3 users added
- 1 user removed
- 47 grants added
- 12 grants removed

Notable additions:
- User: jane.doe@example.com
- Grant: jane.doe -> Engineering group membership
..."
```

## Implementation Phases

### Phase 1: Foundation
- Add protoc-gen-mcpgw to baton-sdk
- Implement ConnectorDevService (GetMetadata, ValidateConfig)
- Add `--mcp serve` CLI command
- stdio transport

### Phase 2: C1Z Inspection
- Implement C1ZInspectorService (ListResources, ListGrants, ListEntitlements, GetSyncStats)
- c1z file reading utilities

### Phase 3: Sync Operations
- Implement SyncService (RunSync, DiffSyncs)
- Integration with existing sync infrastructure

### Phase 4: Integration
- Update connector template to include MCP by default
- Documentation
- Example configurations for Claude Code

## Security Considerations

1. **Local-only by default**: stdio transport means no network exposure
2. **Credentials in environment**: MCP server reads credentials from env, not passed through MCP
3. **Read-only hints**: Most tools marked `read_only_hint: true` for AI safety
4. **No credential exposure**: Tools never return raw credentials in responses

## Open Questions

1. **Connector selection**: Should there be a single MCP server that can target multiple connector binaries, or one MCP server per connector?

2. **C1Z location**: Should c1z path be a global config or per-tool parameter?

3. **Streaming**: Should RunSync stream progress updates, or return final result?

4. **Resource limits**: What are appropriate defaults for pagination limits?

## Related Work

- `c1/protos/c1mcp/` - Existing c1 MCP service definitions
- `protoc-gen-mcpgw` - Code generator at github.com/ductone/protoc-gen-mcpgw
- `SupportConnectorService` - Production connector investigation tools
