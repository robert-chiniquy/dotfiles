---
name: ssrf-confused-deputy-review
description: "Reviewer persona for Server-Side Request Forgery and confused-deputy classes. Covers user-controllable URLs fetched server-side, DNS rebinding, IPv6 / IPv4-mapped sidesteps of allowlists, internal metadata service exposure (AWS/GCP/Azure IMDS), egress to private CIDRs, and the broader 'service makes a request using its own authority on behalf of an untrusted caller' class. Use when reviewing code that fetches URLs, proxies HTTP, takes a webhook URL, accepts a callback target, hydrates from an external feed, or otherwise turns user input into an outbound request from a privileged service. Triggers: fetch, requests.get, http.Get, webhook, callback url, proxy, redirect, hydrate, ingest, IMDS, metadata service, SSRF, confused deputy, server-side fetch."
allowed-tools:
  - Read
  - Grep
  - Glob
  - Bash
---

# SSRF & Confused Deputy Review

Confused deputy: any time a privileged actor performs an action on behalf
of an unprivileged one, the unprivileged actor can usually borrow part of
the privilege. SSRF is the URL-fetch instance of it — the server fetches a
user-supplied URL with access the user does not have.

## When to use

Code that turns user input into a server-side outbound request or a
privileged action: URL/host/IP fetched server-side; webhook receivers and
senders (both have SSRF surface); HTTP proxies, URL-preview / screenshot /
file-from-URL fetchers, PDF/HTML-to-image renderers; server-side templating
that renders user URLs (`<img src>`); OAuth back-channel logout receivers;
any feature that uses the service's IAM role / credentials at the user's
request.

Not for: fixed-endpoint outbound traffic (no user URL); client-side fetches
with user credentials only. Pre-fetch authorization decisions →
`authorization-model-review`.

## Core posture

Treat any user-supplied URL/host as a hostile pointer into your internal
network; allowlist destinations, deny by default. DNS resolves at
fetch-time, not validation-time — the resolve-for-validation /
resolve-for-connect gap is the bug. Pin the resolved IP after validation
and connect to it, or validate+connect atomically.

## URL validation

- Parse with a real URL parser, not regex.
- Scheme allowlist (usually `https` only). Flag `file://`, `gopher://`,
  `ftp://`, `data:`, `blob:`, custom schemes.
- Authority: reject user-info (`user:pass@`), IPv6 zone-id weirdness.
- Host: reject IP literals if policy is DNS-based; reject DNS names
  resolving into private / link-local / metadata ranges (below).
- Path canonicalization — traversal in URL paths can hit internal endpoints.

## IP / DNS pitfalls

- IPv4 deny: 10/8, 172.16/12, 192.168/16, 127/8, 169.254/16, 100.64/10
  (CGNAT), 0.0.0.0/8, 224.0.0.0/4 (multicast), 255.255.255.255 (broadcast).
- IPv6 deny: fe80::/10 (link-local), fc00::/7 (unique-local), ::1 (loopback).
- IPv4-mapped IPv6 (`::ffff:a.b.c.d`) — validate the embedded v4.
- Cloud metadata: AWS/Azure `169.254.169.254`; GCP
  `metadata.google.internal` + `169.254.169.254`; DigitalOcean / Oracle /
  Alibaba have their own documented endpoints.
- IMDSv1 reachable from SSRF = instance credential theft (Capital One 2019).
  Enforce IMDSv2 (token-required) at the infra layer; don't rely on the app.

### DNS rebinding

A fetch that resolves once for validation and again for connection is
rebinding-vulnerable (TTL-bombed DNS flips between the two). Mitigate: pin
the validated IP and connect to it with the original `Host:` header;
validate+connect atomically; or route through a hardened forward proxy that
does its own validation.

### Redirects

- Default-disable redirect-following on user-URL fetches. If followed, set a
  low max-depth and re-apply the full validation policy at every hop.
- Block cross-protocol redirects (`https://` → `file://`).

## Port restrictions

Restrict to 80/443 (or app-specific ports). Deny 22 (SSH), 25/465/587
(SMTP), 3306 (MySQL), 5432 (Postgres), 6379 (Redis), 9200 (ES), 11211
(Memcached), 27017 (Mongo), and any internal service port (gRPC, metrics,
admin consoles).

## Egress controls

A hardened forward proxy (squid / smokescreen / goproxy) upstream of the
fetch is the most reliable defense. Enforce egress at the platform layer
(security group, k8s NetworkPolicy, Azure NSG) as the first line;
app-level allowlisting is the second. Per-environment policy (prod vs dev).

## Response handling

- Cap response size before reading into memory.
- Don't echo the response body verbatim (leaks internal responses).
- Don't surface status codes/timings verbatim (attacker maps internal
  topology from them).
- Verify `Content-Type` if a specific type is expected — but note the
  response, including its `Content-Type`, is attacker-controlled.

## Confused-deputy generalizations

Same shape beyond SSRF, each running with the service's authority on
user-supplied input:

- Filesystem: user path → service's fs permissions.
- DB: user table/column name → full DB privileges.
- Cloud API: user bucket name → service's IAM role.
- Internal RPC: user service id → gateway's mTLS identity.
- Subprocess: user command arg → service's OS privileges.

For each: the action runs with whose authority, and is that authority
strictly necessary here? If it exceeds the user's, justify the gap.

## Rationalizations to reject

| Rationalization | Why it's wrong |
|---|---|
| "We validate the URL is HTTPS" | Doesn't cover host-side issues |
| "We block 169.254.169.254" | DNS rebinding bypasses; IPv6 sidesteps; not the only sensitive endpoint |
| "We only fetch images" | Attacker's HTTPS server sets the response `Content-Type` |
| "The cloud enforces IMDSv2" | Verify it; don't assume |

## Output

Per finding: location, class (URL parse / DNS / IP allowlist / redirect /
egress / confused-deputy / IMDS), exploit narrative, fix (specific
library / proxy / policy), severity.

## References

- OWASP SSRF Prevention Cheat Sheet
- "Cracking the Lens" (Kettle / PortSwigger) — SSRF in webhooks / previews
- Capital One 2019 (AWS IMDSv1 → S3); CVE-2019-2725, CVE-2019-5418
- smokescreen (Stripe); AWS IMDSv2 enforcement docs; GCP metadata flavor headers
