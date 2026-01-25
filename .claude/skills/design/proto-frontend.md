# Protogen Frontend Patterns

TypeScript patterns for proto-first web applications.

## Stack

| Component | Choice |
|-----------|--------|
| Framework | Next.js (pages directory) |
| Language | TypeScript strict |
| UI | Material-UI (sx prop) |
| Proto Transport | @protobuf-ts/grpcweb-transport |
| Runtime | @protobuf-ts/runtime |
| State | Redux Toolkit (complex), React hooks (simple) |

## Key Rules

1. Proto objects used directly - don't convert to other types
2. Views are dumb (no side effects), containers are smart (gRPC calls)
3. Wrapper components over raw MUI
4. All copy internationalized (react-intl)

## gRPC-Web Transport

```typescript
// lib/transport.ts
import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

export const transport = new GrpcWebFetchTransport({
  baseUrl: window.origin,
  fetchInit: { credentials: 'include' },
});
```

## RPC Status Pattern

```typescript
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

  return () => aborter.abort();
};
```

## Browser Debug Logging

```typescript
// lib/log.ts
type LogLevel = "debug" | "info" | "warn" | "error";

function shouldLog(level: LogLevel, namespace: string): boolean {
  if (level === "warn" || level === "error") return true;
  const debug = new URLSearchParams(window.location.search).get("debug");
  if (!debug) return false;
  if (debug === "true") return true;
  return debug.split(",").includes(namespace);
}

export const log = {
  debug: (ns: string, ...args: unknown[]) => shouldLog("debug", ns) && console.log(`[${ns}]`, ...args),
  info: (ns: string, ...args: unknown[]) => shouldLog("info", ns) && console.info(`[${ns}]`, ...args),
  warn: (ns: string, ...args: unknown[]) => console.warn(`[${ns}]`, ...args),
  error: (ns: string, ...args: unknown[]) => console.error(`[${ns}]`, ...args),
};

// Usage: log.debug("useSession", "event:", event)
// Enable: ?debug=true or ?debug=useSession,CellView
```

## WebSocket Notification Pattern

Subscribe to entity changes:

```typescript
export class WebsocketNotifyClient {
  private subscriptions: Record<string, Record<string, OnNotifyEventType[]>>;

  subscribe(tags: string[], key: string, onNotify: OnNotifyEventType): void;
  unsubscribe(tags: string[], key: string): void;
}

// Usage
notifyClient.subscribe(
  [SessionRef.typeName],
  'my-component',
  (ref, eventType, lastUpdatedAt) => {
    if (eventType === ChangedRefEventType.UPDATED) {
      refetchData();
    }
  }
);
```

## Proto-Driven UI

Protos define UI contracts. A proto message describes what to render, inputs accepted, validation.

```
Proto Schema
    |
    +---> Backend emits field with type
    |
    +---> Frontend renders based on type
```

```protobuf
message Field {
  string name = 1;
  string display_name = 2;

  oneof type {
    StringField string_field = 100;
    BoolField bool_field = 101;
    SelectField select_field = 102;
  }
}

message StringField {
  string default_value = 1;
  optional validate.StringRules rules = 2;

  oneof view {
    TextField text_field = 100;
    PasswordField password_field = 101;
    SelectField select_field = 102;
  }
}
```

Frontend consumption:

```typescript
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

## When to Use Proto-Driven UI

- Agent-generated UI (clarifying questions, tool outputs)
- User-configurable forms
- Dynamic dashboards
- UI structure not known at compile time

## Ref Types for Cache Invalidation

```typescript
const standardRefHandlers: Record<string, MessageType<AllowedRefs>> = {
  [UserRef.typeName]: UserRef,
  [SessionRef.typeName]: SessionRef,
  [FormSchemaRef.typeName]: FormSchemaRef,
};

export const getRefString = (ref: AllowedRefs): string => {
  return Object.entries(ref)
    .map((e) => `${e[0]}:${e[1]}`)
    .join('|');
};
```
