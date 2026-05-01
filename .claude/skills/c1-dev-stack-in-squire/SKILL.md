---
name: c1-dev-stack-in-squire
description: >-
  Stand up a full c1 dev stack inside a Squire env — process-compose, postgres,
  envoy, pub-api, pub-auth, be-* services — wired so an external client can
  drive c1's gRPC surface end to end with TLS + OAuth2 client_credentials.
  Use when testing a Latchkey or other c1 client against a real (not stubbed)
  c1 backend, or when reproducing c1 server-side behavior locally.
  Triggers on: c1 dev env, squire c1 stack, pc/up, dev-util mint-test-client,
  test against c1, c1 OAuth client_credentials.
---

# Standing up a c1 dev stack in a Squire env

This is a runbook with all the friction points encoded. Treat it as a script — if you skip steps the stack flaps and you spend an hour debugging postgres unix sockets.

## When to use

- Driving the Latchkey CLI (or any c1 client) end-to-end against a real c1
  pub-api over TLS with a real OAuth-minted Bearer.
- Reproducing pub-api / be-session / be-innkeeper behavior locally.
- Producing a self-contained env you can hand to a teammate by SSH-forwarding
  envoy 2443.

## Prerequisites

- `squire` CLI authenticated to the gateway (`squire login` if needed).
- An entry in `/etc/hosts` mapping `127.0.0.1 c1dev.c1.ductone.com` (one-time;
  needed because c1's pub-auth resolves the tenant from the Host header and
  the dev tenant is `c1dev` on installation domain `c1.ductone.com`).
- The default squire image **does not** ship with c1 cloned, despite what the
  generic squire-env-management skill claims. We clone it manually.

## Step 1 — create the env

```bash
squire new c1-dev --no-open
# wait until: squire env | grep c1-dev | awk '{print $4}' == "running"
```

Avoid `--prompt` / `--open` if you're driving the env from your laptop rather
than the in-env OpenCode agent.

## Step 2 — clone c1 with the envmgr `git_token` MCP tool

The default-image squire credential helper handles `https://github.com/...`
URLs after the initial clone, but you need a real token to bootstrap. Pull one
from the env's MCP gateway at `localhost:9877`:

```bash
squire ssh <env> -- 'set -e
init() {
  curl -sf -i -X POST http://localhost:9877/mcp \
    -H "content-type: application/json" \
    -H "accept: application/json, text/event-stream" \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{},\"clientInfo\":{\"name\":\"probe\",\"version\":\"1\"}}}" \
  | grep -i ^mcp-session-id | tr -d "\r" | cut -d" " -f2
}
SID=$(init)
call() {
  curl -sf -X POST http://localhost:9877/mcp \
    -H "content-type: application/json" \
    -H "accept: application/json, text/event-stream" \
    -H "mcp-session-id: $SID" -d "$1"
}
call "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}" >/dev/null
call "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"git_token\",\"arguments\":{\"repo\":\"ductone/c1\"}}}" \
  | jq -r ".result.content[0].text" | jq -r ".token"
'
```

Then clone (depth 50 is plenty; full clone is slow over ~3M files):

```bash
TOK=ghs_...
squire ssh <env> -- "git clone --depth 50 https://x-access-token:$TOK@github.com/ductone/c1 /data/squire/src/c1
git -C /data/squire/src/c1 config user.email squire@conductorone.com
git -C /data/squire/src/c1 config user.name 'Squire Agent'
git -C /data/squire/src/c1 remote set-url origin https://github.com/ductone/c1.git"
```

Reset the remote URL so the squire credential helper handles future `git push`
(don't bake the short-lived token into the remote URL — it expires in ~30 min).

## Step 3 — pre-fix two known config bugs

Both are env-image quirks, not c1 bugs. Fix them before `pc/up` so services
don't burn `max_restarts` and get marked `Skipped`.

### Postgres unix socket lock

Postgres tries to `/run/postgresql/.s.PGSQL.5432.lock` which is root-owned in
the squire image. Patch process-compose.yaml to point it at `/tmp`:

```bash
squire ssh <env> -- "mkdir -p /tmp/pg-socket
sed -i '/-c port=5432/a\\      -c unix_socket_directories=/tmp/pg-socket' \
  /data/squire/src/c1/dev/process-compose/process-compose.yaml"
```

### Innkeeper Zoho client id / secret can't be empty

`gen-env.sh` writes empty strings for the Zoho Manage Engine OAuth provider,
but the runtime config validation requires `min_len=3`. The other OAuth
providers have placeholder `abc1234` strings — the Zoho ones don't, so
innkeeper crashloops on startup. Run after `make pc/init`:

```bash
squire ssh <env> -- "sed -i \
  's/^INNKEEPER_ZOHOMANAGEENGINEOAUTHPROVIDER_CLIENT_ID=\$/INNKEEPER_ZOHOMANAGEENGINEOAUTHPROVIDER_CLIENT_ID=abc1234/;
   s/^INNKEEPER_ZOHOMANAGEENGINEOAUTHPROVIDER_CLIENT_SECRET=\$/INNKEEPER_ZOHOMANAGEENGINEOAUTHPROVIDER_CLIENT_SECRET=abc1234/' \
  /data/squire/src/c1/.dev/env/be-innkeeper.env"
```

## Step 4 — build + bring up

```bash
# Cold build of all 21 binaries — about 11 minutes on small flavor.
squire ssh <env> -- "nohup nix develop /data/squire/src/c1#localdev \
  --command bash -c 'export GOOS=linux GOARCH=arm64 && \
    make -C /data/squire/src/c1 pc/build && \
    make -C /data/squire/src/c1 pc/init' > /tmp/pc-build.log 2>&1 &"

# Once that's done (poll with: pgrep -fa 'make pc/build'), run pc/up.
# Use -t=false because there's no TTY on a non-interactive SSH; default-TUI
# mode prints `terminal entry not found: term not set` and bails.
squire ssh <env> -- "nohup nix develop /data/squire/src/c1#localdev \
  --command bash -c 'export GOOS=linux GOARCH=arm64 && \
    process-compose up -t=false \
      -f /data/squire/src/c1/dev/process-compose/process-compose.yaml' \
    > /tmp/pc-up.log 2>&1 &"
```

Process-compose exposes a REST API on `localhost:8080`:

```bash
squire ssh <env> -- "curl -sf http://localhost:8080/processes | \
  jq -r '.data[] | \"\(.name): \(.status)\"' | sort"
```

Wait until `postgres / valkey / pub-api / pub-auth / be-session / be-vault / be-innkeeper` are
all `Running` and `ensure: Completed`. The whole bringup takes 1-2 minutes.

### If `be-innkeeper: Skipped`

If you see this even after the Zoho fix, innkeeper hit `max_restarts=30`
during the early postgres-flapping period and process-compose gave up. Start
it manually:

```bash
squire ssh <env> -- "set -a; . /data/squire/src/c1/.dev/env/be-innkeeper.env; set +a;
nohup /data/squire/src/c1/build/linux_arm64/be-innkeeper/be-innkeeper \
  > /tmp/innkeeper.log 2>&1 &"
```

Then re-run `dev-util ensure` to populate `CrossTenantSettings` (innkeeper's
init code creates this row on first start; without it, anything that calls
`tenants.TenantDomain` returns `dynamo: no item found`):

```bash
squire ssh <env> -- "set -a; . /data/squire/src/c1/.dev/env/dev-shell.env; set +a;
/data/squire/src/c1/build/linux_arm64/dev-util/dev-util ensure"
```

## Step 5 — mint a client_credentials pair

The `dev-util mint-test-client` cmd (PR #17295 / merged) creates a user in the
target tenant, promotes them to `SystemOwnerRoleId`, and mints a personal
OAuth2 client. Without this cmd you'd be doing direct postgres inserts.

```bash
squire ssh <env> -- "set -a; . /data/squire/src/c1/.dev/env/dev-shell.env; set +a;
/data/squire/src/c1/build/linux_arm64/dev-util/dev-util mint-test-client \
  --tenant-domain=c1dev --log_level=error" 2>&1 | grep -E '^(client_|user_|tenant_)'
```

Output is grep-able:

```
client_id=mellow-flatcar-10265@c1dev.c1.ductone.com/pcc
client_secret=secret-token:conductorone.com:v1:eyJrdHk...
user_id=3D5vAVJPtjmttwCTphpWsZ2uVav
tenant_id=3D5ijhr15puycSTgo0ol87hz4yE
tenant_domain=c1dev
```

The client_id encodes the tenant's installation domain (`c1.ductone.com`
in this default config). If your env has a different `INNKEEPER_INSTALLATION_DOMAIN`
(squire envs sometimes get squire-specific ones like
`envoy--<env-id>.us-west-2.squire.ductone.com`), the client_id will look
different and the laptop /etc/hosts entry won't apply.

## Step 6 — drive a client from your laptop

Three pieces of laptop setup:

```bash
# (a) tunnel envoy 2443 — squire's own `tunnel` mangles TLS bytes; use ssh -L
ssh -fN -L 12443:127.0.0.1:2443 <env>.squire

# (b) /etc/hosts (one-time, requires sudo)
echo "127.0.0.1 c1dev.c1.ductone.com" | sudo tee -a /etc/hosts

# (c) pull the dev CA fresh — it's regenerated by certgen on each pc/init
scp <env>.squire:/data/squire/src/c1/.dev/pki/service-ca.crt /tmp/c1-dev-ca.pem
```

Then drive the client. For Latchkey:

```bash
latchkey \
  --c1-url https://c1dev.c1.ductone.com:12443 \
  --tls-trust-cert /tmp/c1-dev-ca.pem \
  --tls-server-name localhost \
  --client-id "mellow-flatcar-10265@c1dev.c1.ductone.com/pcc" \
  --client-secret "secret-token:..." \
  vault list
```

Why these flags:
- URL host is the **tenant** subdomain so pub-auth's `tenants.SplitDomain`
  finds the c1dev tenant and pub-api's authn middleware accepts the request.
- `--tls-server-name=localhost` because the dev cert SAN is `localhost` plus
  internal-service DNS names — it doesn't include `c1dev.c1.ductone.com`. The
  override tells tonic + reqwest to validate against the `localhost` SAN
  while the URL host stays `c1dev.c1.ductone.com` for routing.
- The CLI exchanges client_credentials against
  `https://c1dev.c1.ductone.com:12443/auth/v1/token` (pub-auth, not the
  legacy `/auth/token`) on startup, then uses the access token as Bearer.

## Smoke test (30s) — is this env still healthy?

Run this when picking up a paused / older env, or when something looks
off mid-test, before spending 15 min re-bringing-up. Three layers:
process-compose is alive, OAuth still mints, gRPC still answers.

```bash
ENV=<env-name>           # e.g. lk-mint-client
CLIENT_ID="..."          # cached from mint-test-client
CLIENT_SECRET="..."

# (1) Inside the env — pc states + critical service health.
squire ssh "$ENV" -- '
  cd /data/squire/src/c1
  pc/list 2>/dev/null | grep -E "envoy|pub-api|pub-auth|be-session|be-innkeeper|postgres|valkey" \
    | awk "{ printf \"%-20s %s\n\", \$1, \$2 }"
  echo "---"
  curl -ksf https://localhost:2443/healthz/ready && echo "envoy: OK" || echo "envoy: FAIL"
'

# (2) From the laptop — OAuth round-trip against the SSH-forwarded
#     envoy. Returns the access_token if pub-auth + dev CA + tunnel
#     all work end to end.
curl -sf --cacert /tmp/c1-dev-ca.pem \
  --resolve c1dev.c1.ductone.com:12443:127.0.0.1 \
  -d grant_type=client_credentials \
  -d client_id="$CLIENT_ID" \
  -d client_secret="$CLIENT_SECRET" \
  https://c1dev.c1.ductone.com:12443/auth/v1/token \
  | jq -r '.access_token // .error_description // .error' | head -c 80; echo

# (3) Trivial gRPC roundtrip via the CLI. Empty list = stack is
#     healthy and your principal has Latchkey perms.
latchkey \
  --c1-url https://c1dev.c1.ductone.com:12443 \
  --tls-trust-cert /tmp/c1-dev-ca.pem \
  --tls-server-name localhost \
  --client-id "$CLIENT_ID" \
  --client-secret "$CLIENT_SECRET" \
  --format json-line \
  vault list
# Expected: {"list":[],"next_page_token":""}
```

Failure mapping:

- **(1) any of envoy/pub-api/pub-auth not in `Running`**: process-compose
  has flapped. Open `pc/attach`, restart the failing service, and
  consult the Verification chain table below for the usual root
  causes (postgres unix-socket perms, innkeeper Zoho env, etc.).
- **(2) returns `error` / `error_description`**: pub-auth path is up
  but rejecting the credentials. Re-mint with `dev-util mint-test-client`
  and update CLIENT_ID/CLIENT_SECRET.
- **(2) curl exits non-zero**: SSH tunnel is dead or `/etc/hosts` lost
  the `c1dev.c1.ductone.com` mapping. Re-run the laptop setup
  one-liners above.
- **(3) succeeds with `{"list":[]}` but you expected vaults**: your
  principal mints but lacks Latchkey perms — re-check the
  SystemOwner ServiceRoles + tenant Latchkey FF (Verification table).
- **(3) fails with `policy_denied (PermissionDenied: ...)`**: same as
  the previous bullet; you reached pub-api but the role/FF chain is
  broken.

Use `latchkey auth claims` (no extra round-trip) to verify the
principal/tenant the CLI is scoped to before driving any
device-register or per-tenant flow.

## Verification chain — what you should see at each step

| Symptom | Meaning |
|---|---|
| `transport: error sending request` | Stale CA cert. SCP `/data/squire/src/c1/.dev/pki/service-ca.crt` fresh. |
| `Invalid input domain: 'localhost:12443'` | Forgot the /etc/hosts entry; URL host needs to be the tenant subdomain. |
| `dynamo: no item found` (mint-test-client) | be-innkeeper never came up; CrossTenantSettings missing. Restart innkeeper + re-run ensure. |
| `not_found (5)` from `/auth/v1/token` | Client_id/secret don't match a row in postgres. Re-run mint-test-client. |
| `oauth2 invalid_client` (CLI) | Same as above; CLI maps OAuth `invalid_client` to `Unauthenticated`. |
| `policy_denied (PermissionDenied: ...)` | Auth chain works — user just lacks permissions for the specific RPC. `SystemOwnerRoleId`'s `ServiceRoles` list is a hand-rolled allowlist in `pkg/builtin_roles/builtin_roles.go::GetSystemOwner` — newer services aren't in it by default (e.g. Latchkey). Add `latchkey_v1.LatchkeyServiceOwnerRole` (or whichever new service-role) to the slice and rebuild + restart pub-api **and** be-session (be-session is what builds the passport). The persisted role record in dynamo is overlayed by `builtin_roles.ApplyBuiltinAttributes` on every read, so just rebuilding the binaries is enough — no DB migration needed. |
| `unauthenticated` | Bearer token invalid or expired (default lifetime is 30 min). Re-run with fresh creds. |

## Squire-env-specific caveats

- `squire tunnel` proxies as a websocket and corrupts TLS handshakes in both
  directions. **Always use `ssh -L`** for TLS-fronted services.
- The default OpenCode model whitelist on a fresh env may be `claude-opus-4-7`
  only. If you spawn an in-env OpenCode agent and set a different model in
  `prompt_async`, the call returns silently with `ProviderModelNotFoundError`
  and the agent looks frozen. Always `cat .config/opencode/opencode.json |
  jq '.provider.anthropic.whitelist'` first.
- OpenCode + opus-4-7 will sometimes hit the Anthropic API
  `assistant message prefill` 400 error mid-session and stop streaming. The
  partial work it did is salvageable — check `git log` and `git ls-remote
  origin` from the env; if a branch is pushed, drive the rest from outside.
- Each squire env's `INNKEEPER_INSTALLATION_DOMAIN` is set per-env. In
  cloud-routed envs it's a squire subdomain. In non-cloud-tested envs the
  default is `c1.ductone.com`. Always check `.dev/env/be-innkeeper.env`
  before composing tenant URLs.

## Cleanup

```bash
# stop the env (preserves state — restart with `start_env`)
squire env <env-id>  # selects
# or via MCP from inside another env: stop_env tool

# delete entirely
squire env delete <env-id>
```

State on EFS persists between stop/start; the dev CA + postgres data + minted
clients all survive.
