---
name: key-lifecycle-review
description: "Reviewer persona for the full lifecycle of cryptographic keys and high-value secrets: generation, storage, distribution, rotation, revocation, and destruction. Trail of Bits' `zeroize-audit` covers the destruction half; this skill covers the other four phases plus closes the loop with destruction. Use when reviewing key management code, secret stores, KMS integrations, rotation logic, key derivation, RNG usage, or any system that issues, holds, or revokes long-lived credentials. Triggers: key generation, key rotation, KMS, HSM, secret store, vault, key derivation, KDF, master key, DEK, KEK, rotation, revocation, RNG, entropy, random, secrets management."
allowed-tools:
  - Read
  - Grep
  - Glob
  - Bash
---

# Key & Secret Lifecycle Review

Reviews the full lifecycle of cryptographic keys and high-value
secrets: generate → store → distribute → use → rotate → revoke →
destroy. Most key-management incidents are not "crypto broke" — they
are "the key escaped its lifecycle stage."

## When to use

- Reviewing code that generates, stores, or distributes keys/secrets
- KMS / HSM / Vault integration
- Adding or changing a rotation schedule
- Designing a key derivation hierarchy (master key → KEK → DEK)
- Reviewing a secrets-management migration
- Auditing how environment variables, config files, and runtime memory
  hold key material

## When NOT to use

- Algorithm/primitive selection (use `custom-crypto-detection` for
  "did they roll their own", and `sharp-edges` for footgun APIs)
- Token-specific flows (OAuth refresh, session tokens) — use
  `oauth-oidc-review`
- TLS configuration — separate pass; references TLS keys but doesn't
  manage them

## Core posture

A key has a start of life and an end of life. Most production code
gets the start right (generation) and forgets the end (destruction,
revocation). The middle (rotation) is uniformly weakest.

## Phase 1: Generation

- RNG source identified: CSPRNG (`/dev/urandom`, `getrandom(2)`,
  `crypto.randomBytes`, `rand::rngs::OsRng`, Java `SecureRandom`)
- NOT: `Math.random`, `rand()`, language default seeded RNGs, time-based
- Key length matches algorithm and threat model
- Entropy sufficient at the moment of generation (early-boot entropy
  problem on embedded / containers / hypervisors)
- Generation log does NOT include the key material itself

## Phase 2: Storage

- Where on disk / in memory does the key live?
- File permissions: 0400 / owner-only, on a non-shared filesystem
- Memory: zeroized after use (defer to `zeroize-audit` for the
  destruction-edge check)
- Database storage: encrypted with a KEK; KEK in a different trust zone
- Environment variables: documented limitation (process-readable,
  inherited by children, visible in `/proc/*/environ`); used only for
  bootstrap keys that wrap further material
- Backup files (`.bak`, `.tmp`, `.swp`, editor swap, log files):
  searched for stray copies

## Phase 3: Distribution

- Keys transit only over authenticated + encrypted channels
- Bootstrap key for the first hop: how is it injected, and where did
  it come from?
- mTLS, Vault sidecar, IAM-assume-role-with-WebIdentity, GCP Workload
  Identity, K8s service account tokens — each is a distribution
  mechanism with its own bootstrap problem

## Phase 4: Use (rate, scope, sender constraint)

- Key bound to a purpose: signing keys never decrypt, encryption keys
  never sign, MAC keys never used as RSA private keys (key-confusion)
- Use rate metered (anomaly detection on key use spikes)
- Sender-constrained where possible (DPoP, mTLS, IP allowlist for
  service-to-service)

## Phase 5: Rotation

The most-fragile phase.

- Rotation cadence defined and enforced (cron, KMS auto-rotate, manual
  drill)
- **Overlap window**: old key valid during the transition; both
  signatures verifiable until the overlap expires
- New key publishable BEFORE old key invalid (so verifiers can fetch
  it ahead of first encounter)
- Rotation is non-blocking: a rotation in flight doesn't lock callers
  out
- Rotation failure rolls back cleanly; partial-rotation state is
  detected and remediated
- The rotation procedure is *tested in a drill* on at least one
  staging environment per quarter
- Emergency rotation runbook exists and is rehearsed

## Phase 6: Revocation

- Revocation list / CRL / OCSP / KMS disable mechanism is reachable
  and timely
- Revocation propagates within a documented SLA (e.g., ≤5 min for
  high-privilege keys)
- Verifiers check revocation; lazy-cache without check window is a bug

## Phase 7: Destruction

- Defer to `zeroize-audit` for in-memory wiping
- On-disk: shred / KMS-managed delete with audit trail
- Backups: retention policy explicitly covers key material; key
  material backed up only as encrypted (key-encrypting-key never in
  the same backup)

## Cross-cutting

### Hierarchies

- Master key → KEK → DEK depth limited and audited
- Each layer's rotation is independent
- Compromise of one layer's key is bounded (DEK compromise → re-wrap
  affected records, not full re-encrypt of everything)

### Multi-region / multi-tenant

- Per-region keys: a region-specific key doesn't cross the region
  boundary even in DR scenarios without explicit policy
- Per-tenant keys: tenant deletion invalidates the tenant's keys
  before the tenant data is deleted

### Build / deploy / boot

- Secrets injected at deploy, never baked into images
- CI/CD secrets: scope-pinned, short-lived, auditable
- Boot-time secrets unsealed via TPM / KMS / Vault, not from
  unencrypted disk

## Common findings

| Finding | Class |
|---|---|
| Key generated with `Math.random` / non-CSPRNG | Phase 1 |
| Private key in source repo / .env file in image | Phase 2 |
| Rotation cron exists but the read-old-and-new code path was never written | Phase 5 |
| Revocation list exists but nothing checks it | Phase 6 |
| Key cleared from process memory but written to swap | Phase 7 |
| Key derivation uses constant salt | Phase 1 |
| KEK and DEK live in the same KMS keyring with same access policy | Hierarchy |

## Rationalizations to reject

| Rationalization | Why it's wrong |
|---|---|
| "KMS handles all of this" | KMS handles storage and use-control. You handle generation requests, rotation schedule, revocation propagation, and destruction policy. |
| "We rotate annually" | Annual rotation without a drill is a hope, not a plan |
| "It's an internal key" | Internal keys protect external data and are stolen by internal incidents |
| "We zeroize it" | Defer the validation; confirm with `zeroize-audit` |

## References

- NIST SP 800-57 Parts 1–3 (Key Management)
- NIST SP 800-90A/B/C (RNG / DRBG)
- IETF RFC 5280 (X.509 / CRL), RFC 6960 (OCSP)
- OWASP Cryptographic Storage Cheat Sheet
- TOB's `zeroize-audit` and `constant-time-analysis` plugins

## Status

**v0.1 draft** — covers all seven lifecycle phases at outline depth.
Expansion: per-KMS-provider checklists (AWS KMS, GCP KMS, HashiCorp
Vault, Azure Key Vault); language-specific patterns for in-memory
key handling.
