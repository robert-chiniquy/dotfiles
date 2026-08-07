---
name: c1-dev-stack-in-squire
description: >-
  Stand up a full c1 dev stack inside a Squire env — process-compose, postgres,
  envoy, pub-api, pub-auth, be-* services — wired so an external client can
  drive c1's gRPC surface end to end with TLS + OAuth2 client_credentials.
  Use when testing a Latchkey or other c1 client against a real (not stubbed)
  c1 backend, or when reproducing c1 server-side behavior locally.
  Triggers on: c1 dev env, squire c1 stack, pc/up, dev-util mint-test-client,
  test against c1, c1 OAuth client_credentials, run c1 integration tests in
  squire, repro buildkite integration test, TEST_LOCAL_EXEC, api_no_uplift.
---

# Standing up a c1 dev stack in a Squire env

Runbook — follow as a script; skipped steps make the stack flap.


## Common Mistakes

1. **Using Tilt in squire c1 env** — runtime is squire-envmgr / process-compose, not Tilt.
2. **Assuming services are up without `env_status` / health** — verify before client tests.
3. **Minting clients without ensure-tenant** — order: ensure → ensure-tenant → mint-test-client.
4. **Long-lived processes over plain ssh without setsid** — they die with the session.

## When to use

- Driving the Latchkey CLI (or any c1 client) end-to-end against a real c1
  pub-api over TLS with a real OAuth-minted Bearer.
- Reproducing pub-api / be-session / be-innkeeper behavior locally.
- Producing a self-contained env handed off by SSH-forwarding envoy 2443.

## Prerequisites

- `squire` CLI authenticated to the gateway (`squire login` if needed).
- An entry in `/etc/hosts` mapping `127.0.0.1 <tenant-subdomain>.<installation-domain>`
  (one-time). The Host label is **only** for HTTP/gRPC routing via
  `tenants.SplitDomain`. It is **not** the multipass/latchkey `--tenant` value
  (that is the tenant **id** from mint-test-client / whoami). Read both from
  mint output: `tenant_domain=…` vs `tenant_id=…`.
- The default squire image does **not** ship with c1 cloned, despite what the
  generic squire-env-management skill claims. Clone it manually.

## Step 1 — create the env

```bash
squire new c1-dev --no-open
# wait until: squire env | grep c1-dev | awk '{print $4}' == "running"
```

Avoid `--prompt` / `--open` if driving the env from your laptop rather than
the in-env OpenCode agent.

## Step 2 — clone c1 with the envmgr `git_token` MCP tool

The squire credential helper handles `https://github.com/...` URLs after the
initial clone, but bootstrapping needs a real token from the env's MCP
gateway at `localhost:9877`:

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

Clone with `--depth 50` (full clone is slow over ~3M files):

```bash
TOK=ghs_...
squire ssh <env> -- "git clone --depth 50 https://x-access-token:$TOK@github.com/ductone/c1 /data/squire/src/c1
git -C /data/squire/src/c1 config user.email squire@conductorone.com
git -C /data/squire/src/c1 config user.name 'Squire Agent'
git -C /data/squire/src/c1 remote set-url origin https://github.com/ductone/c1.git"
```

The `remote set-url` matters: the token expires in ~30 min, so don't bake it
into the remote — the credential helper handles future `git push`.

## Step 3 — pre-fix two known config bugs

Both are env-image quirks, not c1 bugs. Fix before `pc/up` so services don't
burn `max_restarts` and get marked `Skipped`.

### Postgres unix socket lock

Postgres tries `/run/postgresql/.s.PGSQL.5432.lock`, root-owned in the squire
image. Point it at `/tmp`:

```bash
squire ssh <env> -- "mkdir -p /tmp/pg-socket
sed -i '/-c port=5432/a\\      -c unix_socket_directories=/tmp/pg-socket' \
  /data/squire/src/c1/dev/process-compose/process-compose.yaml"
```

### Innkeeper Zoho client id / secret can't be empty

`gen-env.sh` writes empty strings for the Zoho Manage Engine OAuth provider,
but runtime config validation requires `min_len=3` (the other providers get
placeholder `abc1234`; Zoho doesn't), so innkeeper crashloops. Run after
`make pc/init`:

```bash
squire ssh <env> -- "sed -i \
  's/^INNKEEPER_ZOHOMANAGEENGINEOAUTHPROVIDER_CLIENT_ID=\$/INNKEEPER_ZOHOMANAGEENGINEOAUTHPROVIDER_CLIENT_ID=abc1234/;
   s/^INNKEEPER_ZOHOMANAGEENGINEOAUTHPROVIDER_CLIENT_SECRET=\$/INNKEEPER_ZOHOMANAGEENGINEOAUTHPROVIDER_CLIENT_SECRET=abc1234/' \
  /data/squire/src/c1/.dev/env/be-innkeeper.env"
```

## Step 4 — build + bring up

```bash
# Cold build of all 21 binaries — about 11 minutes on small flavor.
# SQUIRE_ENV_ID must be set in the shell (squire injects it) so Makefile
# GO_TAGS=squire lands in every binary; without it be-ratelimit dies on
# Redis TLS (see "If be-ratelimit crashloops").
squire ssh <env> -- "nohup nix develop /data/squire/src/c1#localdev \
  --command bash -c 'export GOOS=linux GOARCH=arm64 && \
    make -C /data/squire/src/c1 pc/build && \
    make -C /data/squire/src/c1 pc/init' > /tmp/pc-build.log 2>&1 &"

# Once that's done (poll for a sentinel file, e.g. '&& touch /tmp/pc-build.done'
# appended to the build command), run pc/up via a launcher script. TWO traps:
#   1. process-compose.yaml uses $PWD-relative paths ($PWD/.dev/...), so the
#      working directory MUST be the repo root — launched from $HOME, every
#      service resolves /home/squire/.dev and postgres/dynamodb/temporal fail
#      with missing dirs/jars.
#   2. `make pc/up` does NOT pass -t=false, so it dies on a non-interactive
#      SSH with `terminal entry not found: term not set`. Use raw
#      process-compose with -t=false.
# A launcher script satisfies both (write locally + scp if your harness blocks
# inline `cd`):
cat > /tmp/pc-launch.sh <<'EOF'
#!/bin/bash
cd /data/squire/src/c1 || exit 1
export GOOS=linux GOARCH=arm64
exec nix develop /data/squire/src/c1#localdev --command \
  process-compose up -t=false -f /data/squire/src/c1/dev/process-compose/process-compose.yaml
EOF
scp /tmp/pc-launch.sh <env>.squire:/tmp/pc-launch.sh
squire ssh <env> -- "chmod +x /tmp/pc-launch.sh; setsid -f /tmp/pc-launch.sh > /tmp/pc-up.log 2>&1"
```

If you have to kill and relaunch process-compose, kill the ORPHANS too:
killing the supervisor leaves postgres holding `postmaster.pid` + port 5432
(and valkey its port), so the next instance's copies crashloop to `Skipped`
while everything else flaps. Find PIDs via `ss -tlnp` on 5432/6379/8080/2443
and `kill -9` them before relaunching. (`pkill -f "postgres -D\|postgres:"`
does NOT work — `\|` is not alternation in pkill.) Alternatively, if an
orphaned postgres is healthy and serving, adopt it: stop the flapping pc
copies via `curl -X PATCH http://localhost:8080/process/stop/postgres` (and
valkey) and carry on.

Process-compose exposes a REST API on `localhost:8080`:

```bash
squire ssh <env> -- "curl -sf http://localhost:8080/processes | \
  jq -r '.data[] | \"\(.name): \(.status)\"' | sort"
```

Wait until `postgres / valkey / pub-api / pub-auth / be-session / be-ratelimit /
be-vault / be-innkeeper` are all `Running` and `ensure: Completed`. Bringup
takes 1-2 minutes. **Do not skip `be-ratelimit`** — client errors that mention
`127.0.0.1:6014` are almost always ratelimit, not session (session is **6015**).

### If `be-ratelimit` crashloops (dial `127.0.0.1:6014` / introspect 503)

Port map (dev process-compose): **be-ratelimit :6014**, **be-session :6015**.
pub-api uses `API_RATELIMIT_PORT=6014` and `API_SESSION_PORT=6015`. CreateVault,
introspect, and other gated RPCs fail with `GRPC_UNAVAILABLE` / connection
refused / TLS handshake errors citing **6014** when ratelimit is down — not
because session is mis-pointed.

Typical cause on a long-lived env: binary built **without** `-tags=squire`.
Valkey is TLS-only on `127.0.0.1:6379`; squire builds set Redis TLS `ServerName`
via `usquire.AdjustRedisTLS` (`pkg/usquire/squire.go`). Production/`!squire`
builds leave that a no-op (`noop.go`), and init dies with:

`RedisRateLimitCache: ping failed on init: tls: either ServerName or InsecureSkipVerify must be specified`

Makefile auto-sets `GO_TAGS=squire` when `SQUIRE_ENV_ID` is set. Rebuild and
restart:

```bash
squire ssh <env> -- 'cd /data/squire/src/c1 && \
  nix develop .#localdev --command make dev-be-ratelimit GO_TAGS=squire GO_BUILDVCS=false && \
  curl -sf -X POST http://localhost:8080/process/restart/be-ratelimit'
# expect Ready + listen *:6014 + https://127.0.0.1:4514/ready
```

If postgres is also `Restarting` (dirty shutdown / orphaned postmaster), recover
it first — ratelimit and most be-* services will not stay healthy while
postgres flaps. Clean stop orphans, then process-compose restart:

```bash
squire ssh <env> -- 'cd /data/squire/src/c1
# if a manual/orphan postgres holds 5432, stop it cleanly; then:
curl -sf -X POST http://localhost:8080/process/restart/postgres
pg_isready -h 127.0.0.1 -p 5432'
```

### If `be-innkeeper: Skipped`

Even after the Zoho fix, innkeeper can hit `max_restarts=30` during the early
postgres-flapping period and process-compose gives up. Start it manually:

```bash
squire ssh <env> -- "set -a; . /data/squire/src/c1/.dev/env/be-innkeeper.env; set +a;
nohup /data/squire/src/c1/build/linux_arm64/be-innkeeper/be-innkeeper \
  > /tmp/innkeeper.log 2>&1 &"
```

Then re-run `dev-util ensure` to populate `CrossTenantSettings` (innkeeper's
init creates this row on first start; without it, anything calling
`tenants.TenantDomain` returns `dynamo: no item found`):

```bash
squire ssh <env> -- "set -a; . /data/squire/src/c1/.dev/env/dev-shell.env; set +a;
/data/squire/src/c1/build/linux_arm64/dev-util/dev-util ensure"
```

### Step 4b — file gateway needs minio (Latchkey uploads)

The sealed-file gateway (`PUT /file/:fileToken` on pub-api — KeyPackage
publish, secret-value upload, protocol artifacts) writes to the tenant object
store, which `gen-env.sh` points at a REAL AWS bucket
(`API_TENANTOBJECTSTORES3_BUCKET_NAME=dev-c1-tenant-objects-us-west-2`) with
empty creds. Squire envs have no AWS credentials, so every upload 504s (hang)
or 500s. Run minio and point pub-api at it — the config has an `Endpoint`
field (proto field 5):

```bash
squire ssh <env> -- 'curl -sSL -o /tmp/minio https://dl.min.io/server/minio/release/linux-arm64/minio && chmod +x /tmp/minio
mkdir -p /tmp/minio-data/dev-c1-tenant-objects-us-west-2
MINIO_ROOT_USER=dummyaccesskey MINIO_ROOT_PASSWORD=dummysecret \
  nohup /tmp/minio server /tmp/minio-data --address 127.0.0.1:34567 > /tmp/minio.log 2>&1 &
sleep 3; curl -sf http://127.0.0.1:34567/minio/health/live && echo MINIO_LIVE'
# NOTE: use nohup-in-the-ssh-command, and VERIFY MINIO_LIVE prints — a
# setsid -f launch in a one-shot ssh has died silently here before.

squire ssh <env> -- "sed -i \
  's|^API_TENANTOBJECTSTORES3_ACCESS_KEY_ID=.*|API_TENANTOBJECTSTORES3_ACCESS_KEY_ID=dummyaccesskey|; \
   s|^API_TENANTOBJECTSTORES3_SECRET_ACCESS_KEY=.*|API_TENANTOBJECTSTORES3_SECRET_ACCESS_KEY=dummysecret|; \
   /^API_TENANTOBJECTSTORES3_OBJECT_PREFIX=/a\\API_TENANTOBJECTSTORES3_ENDPOINT=http://127.0.0.1:34567' \
  /data/squire/src/c1/.dev/env/pub-api.env
curl -sf -X POST http://localhost:8080/process/restart/pub-api"
```

## Step 5 — mint a client_credentials pair

`dev-util mint-test-client` creates a user in the target tenant, promotes
them to `SystemOwnerRoleId`, and mints a personal OAuth2 client.

```bash
# TENANT_DOMAIN = innkeeper short domain from ensure / innkeeper (NOT tenant id)
squire ssh <env> -- "set -a; . /data/squire/src/c1/.dev/env/dev-shell.env; set +a;
/data/squire/src/c1/build/linux_arm64/dev-util/dev-util mint-test-client \
  --tenant-domain=\${TENANT_DOMAIN:?set from ensure/tenant_domain} --log_level=error" \
  2>&1 | grep -E '^(client_|user_|tenant_)'
```

Output is grep-able (save **all** four — they are different identifiers):

```
client_id=<cute-name>@<tenant_domain>.<installation_domain>/pcc
client_secret=secret-token:conductorone.com:v1:…
user_id=<user id>
tenant_id=<tenant id>          # multipass --tenant / device register --tenant
tenant_domain=<short domain>   # authorize --tenant-domain ONLY
```

**Multi-principal tests: pass `--user-email`.** The user is keyed by email
(default `test-cli@dev.local`), NOT by `--display-name` — re-running the cmd
without `--user-email` mints a new client for the SAME user, which silently
defeats any two-principal flow (share-to-self). For a second principal:
`--user-email=test-cli-b@dev.local --display-name=test-cli-b`.

The client_id encodes the tenant Host (tenant_domain + installation domain).
If the env has a different `INNKEEPER_INSTALLATION_DOMAIN` (squire public
hosts use the `--` collapsed form), the client_id host portion changes and
the laptop `/etc/hosts` entry must match **that** Host, not a guessed label.

### Step 5b — stuck value grant after share (dogfood)

With Latchkey FF grant-activated placement, ShareSecret grants **metadata**
now and files a **value** grant ticket. If that ticket never reaches
PROVISION_COMPLETE (App-owner approval with no current approver is common in
dev), the grantee can accept/join MLS and still fail OpenSecret
(`UserHasVaultValueAccess` false → remapped `NOT_FOUND`).

Bypass for dogfood (product path remains ticket auto-provision):

```bash
squire ssh <env> -- "set -a; . /data/squire/src/c1/.dev/env/dev-shell.env; set +a;
/data/squire/src/c1/build/linux_arm64/dev-util/dev-util complete-value-grant \
  --tenant-domain=\${TENANT_DOMAIN} \
  --vault-boundary-id=vault-… \
  --user-id=<grantee-bare-user-id> \
  --log_level=warn"
# expect: value_grant_completed=true post_value_access=true
# optional: --owner-user-id=… (default: first ListVaultOwnerUserIDs)
```

Rebuild dev-util after pulling the command (branch or main once merged):
`go build -o build/linux_arm64/dev-util/dev-util ./cmd/dev-util` under
`nix develop #localdev` with dev-shell.env sourced.

## Step 6 — drive a client from your laptop

```bash
# TENANT_HOST = <tenant_domain>.<installation_domain> from mint client_id
# (the part between @ and /pcc). Never use this string as multipass --tenant.
TENANT_HOST="<tenant_domain>.<installation_domain>"

# (a) tunnel envoy 2443 — squire's own `tunnel` mangles TLS bytes; use ssh -L
# Prefer local :2443 so gRPC :authority matches the default installation port.
ssh -fN -L 2443:127.0.0.1:2443 <env>.squire

# (b) /etc/hosts (one-time, requires sudo)
echo "127.0.0.1 ${TENANT_HOST}" | sudo tee -a /etc/hosts

# (c) pull the dev CA fresh — it's regenerated by certgen on each pc/init
scp <env>.squire:/data/squire/src/c1/.dev/pki/service-ca.crt /tmp/c1-dev-ca.pem
```

Then drive the client. For Latchkey:

```bash
latchkey \
  --c1-url "https://${TENANT_HOST}:2443" \
  --tls-trust-cert /tmp/c1-dev-ca.pem \
  --tls-server-name localhost \
  --client-id "$CLIENT_ID" \
  --client-secret "$CLIENT_SECRET" \
  vault list
```

Why these flags:
- URL host is the **routing** Host so pub-auth `tenants.SplitDomain` finds the
  tenant. It is **not** the multipass `--tenant` argument (use `tenant_id=`).
- `--tls-server-name=localhost` because the dev cert SAN is `localhost` plus
  internal-service DNS names — not the tenant Host. Validate against
  `localhost` while the URL Host stays the routing label.
- Token URL is `{c1-url}/auth/v1/token` (pub-auth). Keep **pub-api and
  pub-auth build tags aligned** (`GO_TAGS=squire` or not): squire builds
  expect the `--` Host separator; untagged builds expect `.`. Mixing tags
  yields `invalid domain` / `invalid installation domain` on one surface.

## Smoke test (30s) — is this env still healthy?

Run when picking up a paused/older env or when something looks off mid-test,
before spending 15 min re-bringing-up.

```bash
ENV=<env-name>           # e.g. lk-mint-client
CLIENT_ID="..."          # cached from mint-test-client
CLIENT_SECRET="..."

# (1) Inside the env — pc states + critical service health.
squire ssh "$ENV" -- '
  cd /data/squire/src/c1
  pc/list 2>/dev/null | grep -E "envoy|pub-api|pub-auth|be-session|be-ratelimit|be-innkeeper|postgres|valkey" \
    | awk "{ printf \"%-20s %s\n\", \$1, \$2 }"
  echo "---"
  curl -ksf https://localhost:2443/healthz/ready && echo "envoy: OK" || echo "envoy: FAIL"
  ss -lntp 2>/dev/null | grep -E ":6014|:6015" || true
'

# (2) From the laptop — OAuth round-trip against the SSH-forwarded envoy.
#     Returns the access_token if pub-auth + dev CA + tunnel all work.
#     TENANT_HOST from mint client_id (see Step 5).
curl -sf --cacert /tmp/c1-dev-ca.pem \
  --resolve "${TENANT_HOST}:2443:127.0.0.1" \
  -d grant_type=client_credentials \
  -d client_id="$CLIENT_ID" \
  -d client_secret="$CLIENT_SECRET" \
  "https://${TENANT_HOST}:2443/auth/v1/token" \
  | jq -r '.access_token // .error_description // .error' | head -c 80; echo

# (3) Trivial gRPC roundtrip via the CLI. Empty list = stack is
#     healthy and your principal has Latchkey perms.
latchkey \
  --c1-url "https://${TENANT_HOST}:2443" \
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
  has flapped. Open `pc/attach`, restart the failing service, and consult the
  Verification chain table for root causes (postgres unix-socket perms,
  innkeeper Zoho env, etc.).
- **(1) be-ratelimit not Running / nothing on :6014**: rebuild with
  `GO_TAGS=squire` (see "If be-ratelimit crashloops" above). Do not chase
  multipass/session knobs first.
- **(2) returns `error` / `error_description`**: pub-auth is up but rejecting
  the credentials. Re-mint with `dev-util mint-test-client` and update
  CLIENT_ID/CLIENT_SECRET.
- **(2) curl exits non-zero**: SSH tunnel is dead or `/etc/hosts` lost the
  routing Host mapping. Re-run the laptop setup one-liners with the Host
  from the current mint client_id.
- **(3) succeeds with `{"list":[]}` but you expected vaults**: principal
  mints but lacks Latchkey perms — re-check the SystemOwner ServiceRoles +
  tenant Latchkey FF (Verification table).
- **(3) fails with `policy_denied (PermissionDenied: ...)`**: same as the
  previous bullet; you reached pub-api but the role/FF chain is broken.

Use `latchkey auth claims` (no extra round-trip) to verify the
principal/tenant the CLI is scoped to before driving any device-register or
per-tenant flow.

## Verification chain — what you should see at each step

| Symptom | Meaning |
|---|---|
| `transport: error sending request` | Stale CA cert. SCP `/data/squire/src/c1/.dev/pki/service-ca.crt` fresh. |
| `Invalid input domain: 'localhost:…'` | Forgot the /etc/hosts entry; URL Host must be `<tenant_domain>.<installation>` (routing Host), not localhost. |
| `Invalid input domain` with a dotted Host while pub-api is `-tags=squire` | Squire builds use `--` Host separator; untagged builds use `.`. Rebuild pub-api/pub-auth with the **same** `GO_TAGS` or switch Host form. |
| multipass `DEVICE_KEY_TENANT_MISMATCH` after `--tenant <label>` | You passed `tenant_domain` / Host label. Use mint `tenant_id=` (or whoami `tenant`). |
| `dynamo: no item found` (mint-test-client) | be-innkeeper never came up; CrossTenantSettings missing. Restart innkeeper + re-run ensure. |
| `not_found (5)` from `/auth/v1/token` | Client_id/secret don't match a row in postgres. Re-run mint-test-client. |
| `oauth2 invalid_client` (CLI) | Same as above; CLI maps OAuth `invalid_client` to `Unauthenticated`. |
| `policy_denied (PermissionDenied: ...)` | Auth chain works — user just lacks permissions for the specific RPC. `SystemOwnerRoleId`'s `ServiceRoles` list is a hand-rolled allowlist in `pkg/builtin_roles/builtin_roles.go::GetSystemOwner` — newer services aren't in it by default (e.g. Latchkey). Add `latchkey_v1.LatchkeyServiceOwnerRole` (or whichever new service-role) to the slice and rebuild + restart pub-api **and** be-session (be-session builds the passport). The persisted role record in dynamo is overlayed by `builtin_roles.ApplyBuiltinAttributes` on every read, so rebuilding the binaries is enough — no DB migration needed. |
| `unauthenticated` | Bearer token invalid or expired (default lifetime is 30 min). Re-run with fresh creds. |
| Client `GRPC_UNAVAILABLE` / dial **`127.0.0.1:6014`** (CreateVault, introspect 503) | **be-ratelimit** down, not session. Session is **6015**. Rebuild `be-ratelimit` with `GO_TAGS=squire` (Redis TLS ServerName); fix postgres if it is also flapping. See "If be-ratelimit crashloops". |
| be-ratelimit log: `RedisRateLimitCache: ping failed` + `ServerName or InsecureSkipVerify` | Binary missing `-tags=squire`. `make dev-be-ratelimit GO_TAGS=squire` and restart. |

## GO_TAGS consistency (pub-api / pub-auth Host separator)

**Never rebuild only one of `pub-api` / `pub-auth` with a different `GO_TAGS`
than the rest of the front-door pair.**

`usquire.SubdomainSeparator()` is compile-time:

| Build | Separator | Routing Host form |
|---|---|---|
| `-tags=squire` | `--` | `<tenant>--<installation>` |
| untagged (`!squire`) | `.` | `<tenant>.<installation>` |

If pub-api is squire-tagged and pub-auth is not (or the reverse):

- Token exchange can succeed while gRPC returns `invalid domain` / `Unauthenticated`
- Or token returns `invalid installation domain` while gRPC would accept the other form

Dogfood incident (2026-08): rebuilt **only** pub-api with `GO_TAGS=squire` for a
diagnostic patch; produce path died on Host form until pub-api was rebuilt to
**match** live pub-auth tags. When patching a single binary: either
`make pc/restart/pub-api` (inherits Makefile `GO_TAGS ?= squire` when
`SQUIRE_ENV_ID` is set — rebuild the **pair** if you need untagged Hosts), or
explicitly pass the same `GO_TAGS` both services already run with.

Also: `be-ratelimit` **must** be squire-tagged for Redis TLS (`AdjustRedisTLS`).
That is independent of the pub Host form — ratelimit stays squire; the
constraint above is specifically **pub-api ↔ pub-auth** Host parsing.

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
  partial work is salvageable — check `git log` and `git ls-remote origin`
  from the env; if a branch is pushed, drive the rest from outside.
- Each squire env's `INNKEEPER_INSTALLATION_DOMAIN` is set per-env. Check
  `.dev/env/be-innkeeper.env` before composing routing Hosts. Never use that
  Host (or the innkeeper short domain) as multipass `--tenant`.

## Running c1 integration tests in a Squire env (no docker)

Different goal from the running stack above. The `tests/...` integration
suites (e.g. `tests/api_no_uplift`, run in CI as the buildkite
`go-...-testapinouplift-api-no-uplift` shard) don't need the process-compose
services — the suite starts the c1 services **in-process**. They only need
the data backends: postgres, dynamodb-local, temporal, valkey, and an S3
endpoint.

CI runs them with `TEST_TEST_CONTAINER=true`, which spins those up as
**docker** containers (`ci/integration.sh`). **Squire envs have no docker
daemon** — neither the base image nor the c1 image — so the testcontainer
path is dead. Use `TEST_LOCAL_EXEC=true` (`pkg/utest/integration.go` →
`newLocalResourceClient`), which runs every backend as a native binary on
PATH. The c1 nix `localdev` devshell already provides postgres, temporal,
valkey, and java; three things it does **not** set up:

1. **DynamoDBLocal.jar** — `newLocalResourceClient.allocateDynamoDB` looks only
   in `/home/dynamodblocal` or `/usr/local/dynamodblocal` (both root-owned).
   The squire user has passwordless sudo:
   ```bash
   sudo mkdir -p /usr/local/dynamodblocal && sudo chown "$(id -un)" /usr/local/dynamodblocal
   curl -sSL https://d1ni2b6xgvw0s0.cloudfront.net/v2.x/dynamodb_local_latest.tar.gz \
     | tar xz -C /usr/local/dynamodblocal
   ```
2. **An S3 endpoint on `127.0.0.1:34567`** — `allocateS3` *connects* to it (creds
   `dummyaccesskey` / `dummysecret`) but never starts it, and nothing in
   process-compose does either. Run minio (arm64 env):
   ```bash
   curl -sSL -o /tmp/minio https://dl.min.io/server/minio/release/linux-arm64/minio && chmod +x /tmp/minio
   MINIO_ROOT_USER=dummyaccesskey MINIO_ROOT_PASSWORD=dummysecret \
     setsid -f /tmp/minio server /tmp/minio-data --address 127.0.0.1:34567
   ```
3. **A UTF-8 locale** — pgtest's `initdb` inherits the shell locale, which is
   unset on a fresh env. With `LC_ALL=C` the DB comes up SQL_ASCII and every
   query dies with `simple protocol queries must be run with client_encoding=UTF8`
   (the config sets `PreferSimpleProtocol: true`). Use `C.UTF-8` (present as
   `C.utf8` in `locale -a`), not `C`.

Then run the suite (use `go -C` since the block-cd hook rejects `cd`; the
first compile is slow, the binary caches after):
```bash
nix develop /data/squire/src/c1#localdev --command bash -c '
  export TEST_LOCAL_EXEC=true LC_ALL=C.UTF-8 LANG=C.UTF-8 LC_CTYPE=C.UTF-8
  go -C /data/squire/src/c1 test -vet=off -count=1 -v \
    ./tests/api_no_uplift/... -run TestAPINoUplift -timeout 25m
'
```
The buildkite shard name maps directly: `TEST_CASE="TestAPINoUplift|api_no_uplift"`
→ `-run "TestAPINoUplift" ./tests/api_no_uplift/...` (split on the last `|`,
see `ci/integration.sh`). Launch it detached (`setsid -f ... > log; touch done`)
and poll the log — `squire ssh` sessions drop on long holds.

**Caveat — this does not match CI's postgres.** local-exec uses the devshell's
native postgres (currently 18.3); buildkite's testcontainer pins ECR
`postgres:2` (an older major). A test that passes here can still fail in
buildkite (and vice-versa) when behavior is postgres-version-dependent —
e.g. partitioned-table schema handling. A green local-exec run rules out
"broken on modern pg / general staleness" but does **not** clear a buildkite
failure. Reproducing a version-specific failure needs docker + the pinned
image (`TEST_TEST_CONTAINER=true` with `TEST_TEST_CONTAINER_POSTGRES_IMAGE`
set), i.e. a docker host, not squire.

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

## Before finishing

- [ ] Env healthy and required services up?
- [ ] Client credentials minted correctly?
- [ ] No Tilt-based instructions left in the runbook for this env?

