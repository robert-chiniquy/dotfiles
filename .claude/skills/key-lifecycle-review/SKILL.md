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

Review keys/secrets across generate → store → distribute → use → rotate → revoke → destroy.

## Scope split

- Primitive selection / rolled-own crypto → `custom-crypto-detection`; footgun APIs → `sharp-edges`
- OAuth/session token flows → `oauth-oidc-review`
- In-memory wiping → `zeroize-audit` (this skill covers the rest of destruction: disk, backups)
- TLS configuration: separate pass

## Generation

- CSPRNG only; flag seeded, time-based, or language-default RNGs
- Early-boot entropy may be insufficient at generation time on embedded/containers/hypervisors
- Key material must not appear in generation logs
- KDF salt must not be constant

## Storage

- Disk: 0400/owner-only, non-shared filesystem
- DB: encrypted with a KEK held in a different trust zone
- Env vars: process-readable, inherited by children, visible in `/proc/*/environ` — acceptable only for bootstrap keys that wrap further material
- Hunt stray copies: `.bak`/`.tmp`/editor swap files, logs, OS swap

## Distribution

- Authenticated + encrypted channels only
- Every mechanism (mTLS, Vault sidecar, AssumeRoleWithWebIdentity, GCP Workload Identity, K8s SA tokens) has its own bootstrap problem: trace where the first-hop credential comes from and how it is injected

## Use

- Purpose-bound: signing keys never decrypt, encryption keys never sign, MAC keys never used as RSA private keys (key-confusion)
- Use rate metered (anomaly detection on spikes)
- Sender-constrained where possible (DPoP, mTLS, IP allowlist for service-to-service)

## Rotation — the most fragile phase

- Overlap window: old key valid through the transition; both verifiable until overlap expires
- New key published BEFORE old key invalidated, so verifiers can fetch it ahead of first encounter
- Classic failure: rotation cron exists but the read-old-and-new code path was never written
- Non-blocking; partial-rotation state detected and remediated, failure rolls back cleanly
- Drilled in staging at least quarterly; emergency-rotation runbook exists and is rehearsed

## Revocation

- Mechanism (CRL/OCSP/KMS disable) reachable and timely; propagation within a documented SLA (e.g. ≤5 min for high-privilege keys)
- Verifiers must actually check — a revocation list nothing consults, or a lazy cache with no check window, is a finding

## Destruction

- Memory: defer to `zeroize-audit`; also check for key cleared from process memory but written to swap
- Disk: shred / KMS-managed delete with audit trail
- Backups: retention policy explicitly covers key material; backed up only encrypted, and the KEK never in the same backup as what it wraps

## Cross-cutting

- Hierarchy (master → KEK → DEK): depth limited, per-layer rotation independent, compromise bounded (DEK compromise → re-wrap affected records, not full re-encrypt)
- KEK and DEK in the same KMS keyring with the same access policy defeats the hierarchy
- Per-region keys never cross the region boundary, even in DR, without explicit policy
- Tenant deletion invalidates the tenant's keys before the tenant's data is deleted
- Secrets injected at deploy, never baked into images; CI/CD secrets scope-pinned, short-lived, auditable; boot secrets unsealed via TPM/KMS/Vault, not read from unencrypted disk

## Rationalizations to reject

| Claim | Reality |
|---|---|
| "KMS handles all of this" | KMS handles storage and use-control; generation requests, rotation schedule, revocation propagation, and destruction policy are still yours |
| "We rotate annually" | Rotation without a drill is a hope, not a plan |
| "It's an internal key" | Internal keys protect external data and get stolen in internal incidents |
