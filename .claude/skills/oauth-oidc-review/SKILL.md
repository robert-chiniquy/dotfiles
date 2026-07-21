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

Walk the diff (or target subsystem) through the attack classes below. Treat every "we don't need to validate X because Y" as a defect until the threat model accepts the risk in writing.

Out of scope: post-authentication authorization (`authorization-model-review`); cryptographic primitive review (`key-lifecycle-review`, `custom-crypto-detection`).

## Surface enumeration

Locate every OAuth code path before reviewing — bugs hide in the third-or-fourth implementation the reviewer never found. Middleware with ambiguous names (`authMiddleware`) hides what is actually validated; enumerate every route that accepts a token. Find all token storage (cookies, localStorage, sessionStorage, keyring); browser-side storage gets the SPA checks below.

## Authorization code + PKCE (OAuth 2.1 default, OIDC standard)

| Check | Require / Reject |
|---|---|
| `redirect_uri` | Exact string equality against allowlist, including path. Reject substring, prefix, regex, wildcard-subdomain, or scheme-only matching. |
| `state` | High-entropy, server-side stored, bound to user session, single-use (deleted on use), validated on callback. |
| PKCE method | `code_challenge_method=S256` enforced on `/authorize`; reject `plain` and reject when missing. |
| `code_verifier` | sha256(verifier) compared to stored challenge on `/token`; length 43–128 enforced; never accepted unconditionally. |
| Authorization code | Single-use (marked used atomically), lifetime ≤10 min, bound to client_id + redirect_uri. Reuse triggers revocation of tokens issued from it (RFC 6749 §4.1.2). |
| `nonce` (OIDC) | Bound to session (server-side or signed/encrypted cookie), validated on `id_token` receipt, not reused across sessions. |
| `id_token` | Signature validated with key from the issuer's JWKS; reject `alg=none`; pin allowed algs; `aud` must equal this client_id; `iss` must match the expected IdP; validate `exp`/`iat`/`nbf` with skew handling. |

## Refresh tokens

| Check | Require |
|---|---|
| Rotation | New RT on every refresh, old one invalidated; reuse detection revokes the entire family. |
| Scope | Refreshed scope ⊆ original scope — never grows. |
| Binding | Bound to client_id (ideally to client instance); reject mismatch. |
| Storage | Encrypted at rest (encrypted column or KMS-wrapped), not plaintext. |
| Lifetime | Absolute expiry enforced, not sliding-only. 7 days is the high end for SaaS; 24h is conservative. |

## Client credentials

- Secret only in `Authorization: Basic` or request body — never in the query string (logged everywhere).
- Verify the `aud` claim server-side; pin the expected IdP and allowed `aud`.
- Prefer private_key_jwt (RFC 7523) or mTLS over shared secret for clients with privileged scopes.

## Implicit / hybrid

Reject in new code outright. Legacy: flag as deprecated, migrate to code + PKCE; confirm no AT/IT/RT returned via URL fragment to a non-trusted page (referer leak); confirm no session fixation via repeated `/authorize` before user auth.

## Device code (RFC 8628)

- `user_code` displayed and confirmed against the IdP UI by the human — never auto-submitted.
- Polling interval ≥ 5s default; back off on `slow_down`.
- Verification URI on an IdP-controlled domain, not the relying party's.
- `device_code` high-entropy, single-use.

## Token exchange (RFC 8693)

- Validate both `subject_token_type` and `actor_token_type`.
- Exchange policy is an explicit allowlist of (subject, audience, scope) tuples — never "anything from a trusted IdP".
- Original subject preserved or explicitly delegated; no silent identity substitution.
- `act` claim chain grows monotonically; no truncation across exchanges.

## JWT validation bypass patterns

Reject on sight:

| Pattern | Fix |
|---|---|
| Decode without verification (bare `jwt.decode(token)`, PyJWT `verify=False`) | Always verify with pinned key and algs. |
| Library default accepts `alg=none` | Explicitly enumerate allowed algs; never wildcard. |
| Algorithm taken from token header (`HS256` accepted where `RS256` expected — confusion attack) | Pin algorithm server-side; never trust `alg`. |
| `kid` used to fetch a key from an attacker-controllable location (path traversal in `kid`) | Allowlist `kid`; keys only from the known JWKS endpoint. |
| Same secret used as HMAC key and RSA public key (key confusion) | Type keys explicitly; separate HMAC from asymmetric keys. |
| No clock-skew handling | Allow ≤30s leeway, not more. |
| Only `exp` checked | Validate `exp`, `nbf`, and `iat`. |
| `aud` not validated, or substring-matched | Exact match against expected audience. |
| `iss` not validated, or taken from the token itself | Match against the configured IdP issuer. |

## Token handling outside the flow

Most token bugs live in storage, transport, and logging, not the flow.

- Any token in any log line is a leak; confirm structured logging redacts token-bearing fields.
- No tokens in URLs (proxy logs, referrer headers, browser history) — headers or POST bodies only.
- Cookies: `Secure`, `HttpOnly`, `SameSite=Lax` or `Strict`, `Domain` scoped with no parent-domain leak, tight `Path`.
- SPA: any AT/RT in localStorage or sessionStorage is a finding (XSS-readable). HttpOnly cookies + a CSRF strategy instead.
- `/token` responses: `Cache-Control: no-store`, `Pragma: no-cache`.

## Session lifecycle

- IdP logout (back- or front-channel) must invalidate every RP session for the subject; a missing back-channel logout endpoint is a finding.
- `/revoke` exists and is reachable; logout revokes the AT and the RT family.
- Enforce both idle and absolute lifetime; sliding-only is a finding.
- A tab opened after logout in another tab must not reuse a still-cached token.
- Sender-constrained tokens (DPoP, mTLS, cookie-bound RT) preferred for high-value scopes.

## Codebase trust boundaries

- Issuer URL hardcoded or configured at deploy time — never discovered from user-supplied input.
- Cache `/.well-known/openid-configuration` with a TTL and signature pinning; do not re-fetch per request.
- JWKS: cache; refresh on `kid` miss with backoff, not on every validation; cap the cached size.
- Client secrets never in source; vault-backed with rotation.

## Rationalizations to reject

| Claim | Counter |
|---|---|
| "Our IdP enforces this" | Validate server-side anyway; the IdP is a trust boundary, not the only one. |
| "The library handles it" | Validation is configurable; a check not explicitly enabled is off. |
| "PKCE is for mobile/SPA" | OAuth 2.1 mandates PKCE for all public clients; the 2.0 BCP recommends it for confidential clients too. |
| "It's an internal app" | Internal traffic is not a trust boundary; internal users get phished. |
| "Tightening would break clients" | Deprecate with telemetry, then tighten; "permanently lenient" ends in an incident. |

## Findings format

Per finding: location (`file:line`), attack class, severity (Critical = token bypass, silent identity substitution, or PII exposure to attacker), a two-sentence concrete exploit, a specific fix (not "improve validation"), and a spec reference citing the section — RFC 6749, 6750, 7519, 7636, 8252, 8628, 8693, 9068, 9126 (PAR); OAuth Security BCP (`draft-ietf-oauth-security-topics`); OIDC Core 1.0; OIDC Back-/Front-Channel Logout. Pin the BCP version reviewed against — the BCP moves; what was acceptable in 2020 is not now.
