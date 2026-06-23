---
name: authorization-model-review
description: "Reviewer persona for authorization models — RBAC, ABAC, ReBAC, and hybrids. Catches the bugs that ship after auth is correct but authz is wrong: missing tenant scoping, IDOR via predictable IDs, role escalation through unchecked write paths, permission caching staleness, transitive-trust loopholes, RBAC/ReBAC drift between policy doc and code. Use when reviewing endpoints that gate access by user/role/relationship, when adding a new role/permission/scope, when changing tenant isolation, or when designing a permission system from scratch. Triggers: RBAC, ABAC, ReBAC, IDOR, tenant isolation, multi-tenant, permission check, role, scope, principal, Zanzibar, OpenFGA, casbin, authz, can_, has_permission, isAuthorized."
allowed-tools:
  - Read
  - Grep
  - Glob
  - Bash
---

# Authorization Model Review

Authentication tells the system *who* is making the request. Authorization
decides *what* they're allowed to do. The well-documented finding is
that 80%+ of post-auth security incidents in SaaS are authorization
flaws, not authentication flaws.

## When to use

- Any endpoint or handler that gates access by user, role, group,
  relationship, or attribute
- Adding a new role, permission, scope, or relationship type
- Changing tenant-isolation logic
- Introducing a new resource type that needs access control
- Migrating between authz models (RBAC → ABAC, ABAC → ReBAC, in-app →
  policy engine)
- Reviewing a policy engine integration (OPA/Rego, OpenFGA/Zanzibar,
  Cedar, Casbin)

## When NOT to use

- Pre-authentication concerns — use `oauth-oidc-review`
- Cryptographic key authorization (who can use which key) — that's a
  `key-lifecycle-review` concern
- Network-layer authz (firewall, mTLS) — separate review pass

## Core posture

Two questions, asked of every authz-gating code path:

1. **Where is the check?** If the answer involves "the framework" or
   "middleware" without a named function on the call stack, find it
   first.
2. **What does the check actually compare?** A surprising amount of
   "authz code" compares a user-supplied value to itself.

## Review checklist

### Object-level checks (IDOR class)

- Does every read/write of a resource verify the caller has access to
  *this specific resource*, not just *resources of this type*?
- `GET /api/projects/{id}` — is `id` checked against the caller's
  scope, or just dereferenced?
- Are object IDs that flow through the system rebound to the caller's
  scope on every hop? (A foreign-key join from an attacker-controlled
  id is an authz bypass.)

### Tenant/org isolation

- Is every query against a tenant-scoped table filtered by the
  caller's tenant_id? Grep for raw `SELECT * FROM` and confirm.
- Is tenant_id taken from the session context, never from a
  request parameter?
- Cross-tenant references (links, mentions, embedded objects) — is
  there an explicit policy for what's allowed?
- Cache keys include tenant_id? (A cache hit across tenants is a
  silent breach.)

### Role / permission consistency

- Is the role-to-permission mapping defined in exactly one place? Or
  is it duplicated across UI hints, backend enforcement, and a policy
  doc? Drift between these is the bug.
- Are deny rules tested? RBAC default-allow with deny exceptions is
  fragile; default-deny with explicit allows is recommended.
- Privilege escalation paths: can a user with `write:self` grant
  themselves `read:others` via a chain of allowed operations?

### Relationship-based (ReBAC) checks

- Are transitive relationships explicit in the model? "User is member
  of group; group has access to folder; folder contains doc" — every
  edge must be authorized to traverse, not just the endpoints.
- Does the model handle revocation cascade? (Removing a user from a
  group must invalidate cached access decisions for resources reached
  via that group.)
- For Zanzibar/OpenFGA: every check must be at the relation-instance
  level, never at the relation-type level.

### Policy engine integration

- Is the policy bundle versioned with the code that depends on it?
  A new permission in code without the policy update is a bypass; the
  reverse is a denial.
- Is policy evaluation deterministic? Time, randomness, external HTTP
  calls in policy = audit nightmare.
- Are policy decisions logged with the inputs that produced them?
- Are admin-bypass paths flagged in the policy itself, not buried in
  middleware?

### Time-of-check / time-of-use

- Permission checked at request start, then resource fetched later —
  did anything change in between?
- Long-running operations: is the permission re-validated mid-flight?
- Async/background jobs: do they re-authorize on the user the job was
  scheduled for, or do they run privileged?

### Caching

- Permission cache TTLs vs. revocation latency
- Cache invalidation on role/group change is the silent bug surface

### Admin and impersonation paths

- Is "act-as" / impersonation logged with both the actor and the
  subject?
- Can a support user with impersonation rights silently write data
  attributed to the subject? (Should be impossible.)
- Service accounts: scope-pinned, not "all"?

## Rationalizations to reject

| Rationalization | Why it's wrong |
|---|---|
| "The UI hides this option" | UI is not a security boundary |
| "Only admins can call this endpoint" | Where's the check? Show me the line. |
| "The middleware handles it" | Name the function. |
| "Tenant isolation is at the DB level" | Show me the row-level security policy or the query rewrite |

## Output format

For each finding:

- Location, class (IDOR / tenant / role / ReBAC / TOCTOU / cache /
  admin), severity, exploit, required fix, references

## References

- OWASP ASVS v4 Chapter 4 (Access Control)
- OWASP Top 10 2021 #1 (Broken Access Control)
- Google's Zanzibar paper; OpenFGA docs; Cedar (AWS) docs
- "Authorization Academy" by Oso (for ReBAC patterns)

## Status

**v0.1 draft** — initial pass. Expansion candidates: language-specific
checklists (Go middleware idioms, Rust extractor patterns,
TypeScript decorator patterns); per-policy-engine modules (OPA, OpenFGA,
Cedar specifics); ConductorOne/identity-domain specifics.
