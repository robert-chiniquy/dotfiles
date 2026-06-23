---
name: oauth-oidc-review
description: "Reviewer persona for OAuth 2.0 / 2.1 and OpenID Connect flow implementations. Catches the well-documented attack classes that still ship: missing PKCE, wildcard redirect URIs, mishandled refresh tokens, scope creep, mixed flows on a single endpoint, leaking tokens through referrer or logs, JWT signature bypass. Use when reviewing any code that issues, accepts, validates, exchanges, refreshes, revokes, or stores tokens; when designing a new auth integration; when a PR touches /authorize, /token, /userinfo, /jwks, /introspect, /revoke, OIDC discovery, or a third-party identity provider client. Triggers: OAuth, OIDC, JWT, PKCE, redirect_uri, scope, refresh token, access token, id_token, client_credentials, authorization code, implicit, device code, token exchange, identity provider, IdP, SSO."
allowed-tools:
  - Read
  - Grep
  - Glob
  - Bash
---

# OAuth 2.0 / OIDC Flow Review

Reviews OAuth 2.0 / 2.1 and OpenID Connect implementations against the
well-known attack classes. OAuth bugs are concentrated in a small number
of locations — most production incidents trace back to fewer than ten
distinct mistakes. This persona walks the diff (or a target subsystem)
through them.

## When to use

- Any PR that touches `/authorize`, `/token`, `/userinfo`, `/jwks`,
  `/introspect`, `/revoke`, OIDC discovery (`/.well-known/openid-configuration`),
  or a third-party IdP client (Okta, Auth0, Google, Microsoft, Ping, Keycloak)
- Adding or modifying a redirect URI allowlist
- Introducing a new OAuth client or scope
- Changing how tokens are stored, refreshed, revoked, or invalidated
- Migrating between OAuth 2.0 grant types (esp. implicit → PKCE)
- Reviewing JWT signature validation code
- Token exchange (RFC 8693) implementations
- Service-to-service authentication via `client_credentials`
- After any change to session lifetime, multi-tab/multi-device behavior,
  or single sign-out propagation

## When NOT to use

- Application-layer authorization decisions (RBAC/ABAC checks AFTER
  authentication) — use `authorization-model-review` instead
- Cryptographic primitive review (algorithm choice, key derivation) —
  use `key-lifecycle-review` or `custom-crypto-detection`
- Generic input validation — standard code review

## Core posture

OAuth is a *delegation* protocol. Every check the spec describes exists
because someone shipped a real exploit. Treat every "we don't need to
validate X because Y" rationalization as a defect until the threat model
explicitly accepts the risk in writing. The protocol's security
properties are emergent from the combination of validations — skip one
and a downstream check is silently load-bearing in a way the author did
not realize.

## Phase 1: Identify the surface

Before reviewing, locate every code path that participates in OAuth.
Many bugs hide because the reviewer didn't realize the third-or-fourth
implementation existed.

- Grep for `redirect_uri`, `code_verifier`, `code_challenge`,
  `grant_type`, `client_secret`, `id_token`, `access_token`,
  `refresh_token`, `client_credentials`, `device_code`, `assertion`
- Grep for JWT library imports: `jsonwebtoken`, `jose`, `pyjwt`,
  `golang-jwt/jwt`, `auth0/java-jwt`, `nimbusds/nimbus-jose-jwt`
- Grep for OIDC library use: `openid-client`, `oidc-client-ts`,
  `coreos/go-oidc`, `auth0/auth0-spa-js`
- Find every server route that accepts a token: middleware chains often
  hide token validation behind ambiguous names like `authMiddleware`
  that don't reveal what they validate
- Find token storage: cookies, localStorage, sessionStorage, keyring,
  secure storage; flag any of the first three for browser-side review

## Phase 2: Walk the checklist by flow

### Authorization Code + PKCE (OAuth 2.1 default, OIDC standard)

| # | Check | Reject pattern | Required |
|---|---|---|---|
| 1 | `redirect_uri` exact-match against allowlist | substring match, prefix match, regex with `.*`, scheme-only check, wildcard subdomain | Exact string equality including path |
| 2 | `state` parameter present, opaque, single-use, bound to user session | absent, predictable, replayable, not validated on callback | High-entropy random, server-side stored, deleted on use |
| 3 | PKCE `code_challenge_method=S256` enforced on `/authorize` | `plain`, accepted-when-missing | Reject any request without `S256` |
| 4 | `code_verifier` validated against stored `code_challenge` on `/token` | accepted unconditionally, length not validated (43–128 chars) | sha256(verifier) compared with stored challenge, both length-checked |
| 5 | Authorization code single-use, short-lived (≤10 min), bound to client_id + redirect_uri | reuse-tolerant, long-lived, not bound | Marked used atomically on first redemption; reuse triggers token revocation per RFC 6749 §4.1.2 |
| 6 | `nonce` (OIDC) bound to session and validated in `id_token` | not validated, predictable, reused across sessions | Stored server-side or in signed/encrypted cookie; checked on id_token receipt |
| 7 | `id_token` signature validated with key fetched from `/jwks` of the issuer, `iss`/`aud`/`exp`/`iat`/`nbf` validated | `alg=none` accepted, `kid` from token used without trust check, missing `aud` validation, clock skew not handled | Reject `alg=none`; pin allowed algs; `aud` must match this client_id; `iss` must match expected IdP |

### Refresh tokens

| # | Check | Reject pattern | Required |
|---|---|---|---|
| 1 | Refresh token rotation: each use returns a new RT, old one invalidated | RT reusable across sessions | Rotate on every refresh; detect reuse → revoke entire family |
| 2 | RT scope cannot grow on refresh | scope inflated on refresh | New scope ⊆ original scope |
| 3 | RT bound to client_id (and ideally to client instance) | shareable across clients | Reject mismatch |
| 4 | RT storage encrypted at rest | plaintext DB column | Encrypted column or KMS-wrapped |
| 5 | RT lifetime ≤ refresh-window policy; absolute expiry not just sliding | sliding-only, no upper bound | Absolute expiry ≥ 7 days for SaaS is on the high end; 24h is conservative |

### Client credentials

| # | Check | Reject pattern | Required |
|---|---|---|---|
| 1 | Client secret transmitted only in `Authorization: Basic` or request body, never in URL query | secret in query string (logged everywhere) | Basic auth or form body |
| 2 | Service-to-service `aud` claim verified server-side | accepted from any IdP | Pin expected IdP; pin allowed `aud` |
| 3 | mTLS or signed client assertion (JWT-bearer per RFC 7523) preferred over shared secret for high-trust flows | bare client_secret with broad scope | Move to private_key_jwt / mTLS for prod clients with privileged scopes |

### Implicit / hybrid flows

Reject in new code outright. If the codebase still uses them:

| # | Check |
|---|---|
| 1 | Migrate to authorization-code + PKCE; flag implicit as deprecated |
| 2 | Confirm no AT/IT/RT returned via URL fragment to a non-trusted page (referer leak class) |
| 3 | Confirm session fixation not possible via repeated `/authorize` calls before user auth |

### Device code

| # | Check |
|---|---|
| 1 | User_code displayed and confirmed against IdP UI by the human (not auto-submitted) |
| 2 | Polling interval ≥ recommended (RFC 8628 default 5s); back-off on `slow_down` |
| 3 | Verification URI uses an IdP-controlled domain — not the relying-party domain |
| 4 | `device_code` is high-entropy and single-use |

### Token exchange (RFC 8693)

| # | Check |
|---|---|
| 1 | `subject_token_type` and `actor_token_type` both validated |
| 2 | Exchange policy expressed as explicit allowlist of (subject, audience, scope) tuples — not "anything-from-trusted-IdP" |
| 3 | Original subject preserved or explicitly delegated; no silent identity substitution |
| 4 | `act` claim chain monotonically grows; no truncation across exchanges |

### JWT signature validation

The class of "JWT validation bypass" bugs has shipped in production code
at every scale. Specific patterns to reject on sight:

| Pattern | Fix |
|---|---|
| `jwt.decode(token)` without verification (e.g. PyJWT's `verify=False`) | Always verify; use `jwt.decode(token, key=..., algorithms=['RS256'])` |
| Library default accepts `alg=none` | Explicitly enumerate allowed algs; never `'*'` |
| Algorithm taken from token header (`alg=HS256` accepted when expected `RS256` — confusion attack) | Pin algorithm server-side; never trust `alg` |
| `kid` used to fetch a key from an attacker-controllable location (path traversal in `kid`) | Validate `kid` against allowlist; restrict to known JWKS endpoint |
| Symmetric secret used as both HMAC key and RSA public key (key-confusion) | Type the key explicitly; separate HMAC keys from asymmetric keys |
| Clock skew not handled; tokens fail on minor drift | Allow ≤30s leeway, not more |
| `exp` checked but `nbf` and `iat` not validated | Validate all three |
| `aud` not validated, or validated as substring | Exact match against expected audience |
| `iss` not validated, or validated against fetched `iss` from the token | Validate against expected IdP issuer hardcoded or configured |

## Phase 3: Token handling outside the flow

Bugs in token handling rarely live in the flow itself; they live in
storage, transport, and logging.

- **Logs**: grep for `console.log`, `logger.info`, `fmt.Println` near
  token-bearing variables. Any token in any log line is a leak. Use
  redaction middleware. Confirm structured logging redacts named fields.
- **URLs**: tokens in query strings end up in proxy logs, referrer
  headers, browser history. Force tokens into headers or POST bodies.
- **Cookies**: `Secure`, `HttpOnly`, `SameSite=Lax` or `Strict`,
  appropriate `Domain` scope (no parent-domain leak), `Path` scoped
  tightly. Reject sessions set without these.
- **Storage in SPAs**: localStorage is XSS-readable; sessionStorage is
  too. Prefer HttpOnly cookies + a CSRF strategy (double-submit token
  or SameSite). Flag any AT/RT in localStorage as a finding.
- **Caching**: tokens in HTTP responses must not be cached. `Cache-Control: no-store`,
  `Pragma: no-cache` on /token responses.

## Phase 4: Session lifecycle

- Single sign-out: when the IdP sends a logout (back-channel or
  front-channel), every RP session for the subject must be invalidated.
  Missing back-channel logout endpoint = silent finding.
- Token revocation: `/revoke` exists, is reachable, and revokes both
  AT and the RT family on logout
- Idle vs. absolute lifetime: both must be enforced; sliding-only =
  finding
- Multi-tab: opening a new tab after logout in another tab must not
  reuse the still-cached token
- Token binding to client instance (DPoP, mTLS sender-constraint, or
  cookie-bound RT) preferred for high-value scopes

## Phase 5: Trust boundary checks specific to the codebase

- For each IdP integration: confirm the issuer URL is hardcoded or
  configured at deploy time, NOT discovered from a user-supplied input
- For OIDC discovery: cache `/.well-known/openid-configuration` with a
  TTL and signature pinning; do not re-fetch on every request
- For JWKS: cache, refresh on `kid` miss with backoff, NOT on every
  validation; cap the size of the cached JWKS
- For client secrets: never in source, never in env at process level
  unless rotated; preferably vault-backed with short-lived re-fetch

## Phase 6: Document findings

For each finding, report:

- **Location** (`path/to/file:line`)
- **Class** (one of the categories above)
- **Severity** (Critical/High/Medium/Low — Critical = token bypass,
  silent identity substitution, or PII exposure to attacker)
- **Concrete exploit** (what an attacker does, in two sentences)
- **Required fix** (specific code change, not "improve validation")
- **Spec reference** (RFC 6749, 6750, 7519, 7636, 8252, 8628, 8693,
  9068, 9126, OIDC Core 1.0, OIDC Back-Channel Logout 1.0 — cite the
  section, not just the document)

## Rationalizations to reject

| Rationalization | Why it's wrong |
|---|---|
| "Our IdP enforces this" | You still validate server-side. The IdP is a trust boundary, not the trust boundary. |
| "It's an internal app" | Internal apps phish internal users; internal traffic isn't a trust boundary. |
| "We use a library that handles it" | Libraries have configurable validation; if you didn't explicitly enable a check, assume it's off. |
| "PKCE is for mobile/SPA" | PKCE is mandatory for all public clients in OAuth 2.1 and recommended for confidential clients in OAuth 2.0. Use it everywhere. |
| "Refresh tokens are too short-lived to bother rotating" | Token theft + replay is the entire attack class. Rotate. |
| "We can't break clients by tightening" | You can deprecate with telemetry, then tighten. The state of "permanently lenient" is not stable; it ends in an incident. |

## References

- RFC 6749 (OAuth 2.0), RFC 6750 (bearer), RFC 7636 (PKCE), RFC 8252 (native apps),
  RFC 8628 (device), RFC 8693 (token exchange), RFC 9068 (JWT access token profile),
  RFC 9126 (PAR — pushed auth requests)
- OAuth 2.0 Security Best Current Practice (`draft-ietf-oauth-security-topics`,
  latest BCP)
- OIDC Core 1.0; OIDC Back-Channel Logout 1.0; OIDC Front-Channel Logout 1.0
- OWASP ASVS v4 Chapter 3 (Session Management) and Chapter 4 (Access Control)
- `https://oauth.net/2/`'s vulnerability index

Pin the BCP version reviewed against in any finding — OAuth's "best
current practice" moves; what was acceptable in 2020 is not now.
