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

SSRF is the bug where a server fetches a URL the user provided and the
server has access the user does not. Confused deputy generalizes: any
time a privileged actor performs an action on behalf of an unprivileged
one, the unprivileged actor can usually borrow part of the privilege.

## When to use

- Any code path that takes a URL/host/IP from user input and connects
  to it server-side
- Webhook receivers and outbound webhook senders (both sides have SSRF
  surface)
- HTTP proxies, "fetch this URL and screenshot it", URL preview
  generators, file-from-URL uploaders, OAuth back-channel logout
  receivers
- Server-side templating that accepts URLs (`<img src="…">` rendered
  server-side)
- PDF/HTML-to-image renderers
- Anywhere `requests.get(user_url)`, `http.Get(user_url)`, `fetch(...)`
  appears in a privileged context
- Any feature that uses the service's IAM role / credentials to do
  something at the user's request

## When NOT to use

- Pure outbound traffic to fixed endpoints — no user-supplied URL
- Client-side fetches with user credentials only (different threat
  model)
- Pre-fetch authorization decisions — `authorization-model-review`

## Core posture

Treat any user-supplied URL or host as a hostile pointer into your
internal network. Allowlist destinations explicitly. Deny by default.
DNS resolves at fetch-time, not at validation-time — re-resolve
after policy decisions, or validate-and-pin.

## Review checklist

### URL validation

- URL parsed with a real parser (`url.Parse`, `URL`, `urllib.parse`),
  not regex
- Scheme allowlisted (typically `https` only; flag `file://`, `gopher://`,
  `ftp://`, `data:`, `blob:`, custom schemes)
- Authority restricted: no user-info (`user:pass@`), no IPv6 zone
  identifier weirdness
- Host part: not a literal IP if the policy requires DNS-based
  validation; not a DNS name pointing into RFC 1918 / link-local /
  cloud-metadata ranges
- Path canonicalization considered (path traversal in URL paths can
  hit internal endpoints)

### IP / DNS pitfalls

- IPv4 in any of: 10/8, 172.16/12, 192.168/16, 127/8, 169.254/16,
  100.64/10 (CGNAT), 0.0.0.0/8, 224.0.0.0/4 (multicast),
  255.255.255.255 (broadcast)
- IPv6 link-local (fe80::/10), unique-local (fc00::/7), loopback (::1)
- IPv4-mapped IPv6 (`::ffff:a.b.c.d`) — must validate the v4 portion
- Cloud metadata services:
  - AWS: `169.254.169.254`
  - GCP: `metadata.google.internal`, `169.254.169.254`
  - Azure: `169.254.169.254`
  - DigitalOcean / Oracle / Alibaba: documented endpoints exist
- IMDSv1 reachable from SSRF = instance credential theft. Enforce
  IMDSv2 (token-required) at the infrastructure layer; do not rely on
  the application to refuse.

### DNS rebinding

- A fetch that resolves once for validation and once for connection
  is vulnerable to DNS rebinding (TTL-bombed DNS flips between calls)
- Mitigations: pin the resolved IP after validation and connect to that
  IP with `Host:` header; or perform validation+connect atomically; or
  outsource fetching to a hardened forward proxy that does its own
  validation

### Redirects

- Auto-following redirects must re-apply the validation policy at every
  hop, not just on the originally-supplied URL
- Default-disable redirect-following on URL fetches from user input;
  if redirects must be followed, set a low max-depth and validate every
  intermediate target
- Cross-protocol redirects (`https://` → `file://`) blocked

### Port restrictions

- Fetches restricted to 80/443 (or app-specific allowed ports)
- Common-mistake ports to deny: 22 (SSH), 25/465/587 (SMTP), 3306
  (MySQL), 5432 (Postgres), 6379 (Redis), 9200 (Elasticsearch), 11211
  (Memcached), 27017 (Mongo)
- Any internal service port (gRPC servers, metrics endpoints, admin
  consoles)

### Egress controls

- A hardened forward proxy (squid / smokescreen / shopify/goproxy)
  upstream of the fetch is the most reliable defense
- Per-environment egress policy: prod allowlist vs dev allowlist
- Network egress at the platform layer (security group, NetworkPolicy
  in k8s, NSG in Azure) — application-level allowlisting is the second
  line, not the first

### Response handling

- Response size capped before reading into memory
- Response not echoed verbatim to the requester (leaks internal
  responses)
- Status codes not surfaced verbatim (a careful attacker derives
  internal topology from response codes/timings)
- Content-Type checked if the requester expected a specific type

## Confused deputy generalizations

Beyond SSRF, the same shape appears in:

- File system access: user supplies a path, service reads it with the
  service's filesystem permissions
- Database queries: user supplies a table/column name, service queries
  with full DB privileges
- Cloud APIs: user supplies a bucket name, service accesses with the
  IAM role of the service
- Internal RPC: user supplies a service identifier, gateway forwards
  with the gateway's mTLS identity
- Sub-process execution: user supplies a command argument, service
  invokes with the service's OS privileges

For each, ask: "the action runs with whose authority, and is that
authority strictly necessary for this user-initiated action?" If the
authority exceeds the user's, justify the gap.

## Rationalizations to reject

| Rationalization | Why it's wrong |
|---|---|
| "We validate the URL is HTTPS" | Doesn't cover host-side issues |
| "We block 169.254.169.254" | DNS rebinding bypasses; IPv6 sidesteps; cloud metadata endpoint may not be the only sensitive one |
| "We only fetch images" | An attacker-controlled HTTPS server serves whatever it wants — `Content-Type` from the response is attacker-set |
| "The cloud platform enforces IMDSv2" | Verify it; don't assume it |

## Output format

For each finding: location, class (URL parse / DNS / IP allowlist /
redirect / egress / confused-deputy / IMDS), exploit narrative, fix
(specific library / proxy / policy), severity.

## References

- OWASP SSRF Prevention Cheat Sheet
- "Cracking the Lens" (James Kettle / PortSwigger) on SSRF in
  webhooks / preview generators
- CVE-2019-2725, CVE-2019-5418, Capital One 2019 (AWS IMDSv1 → S3)
- AWS docs on IMDSv2 enforcement; GCP metadata flavor headers
- smokescreen (Stripe), Hardened HTTP egress proxies

## Status

**v0.1 draft** — checklist covers the standard SSRF surface and the
core confused-deputy generalizations. Expansion: per-cloud
metadata-service specifics; ingress-side SSRF in serverless platforms;
in-browser fetch policies for the SPA side.
