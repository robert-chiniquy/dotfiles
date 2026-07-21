---
name: sharp-edges
description: "Identifies error-prone APIs, dangerous configurations, and footgun designs that enable security mistakes. Use when reviewing API designs, configuration schemas, cryptographic library ergonomics, or evaluating whether code follows 'secure by default' and 'pit of success' principles. Triggers: footgun, misuse-resistant, secure defaults, API usability, dangerous configuration."
allowed-tools:
  - Read
  - Grep
  - Glob
---

# Sharp Edges Analysis

Find designs where the easy path leads to insecurity. Secure usage must be the path of least resistance; if correct use requires reading docs carefully or remembering special rules, the API has failed. Not for implementation bugs or business-logic flaws — this is design review.

Reject these rationalizations; they don't excuse a footgun:
- "It's documented" — docs affect severity, never excuse the design
- "Advanced users need flexibility" — most advanced usage is copy-paste; hide primitives behind safe high-level APIs
- "Nobody would do that" — assume maximum developer confusion
- "It's just a config option / developer's responsibility" — config is code; validate it, reject dangerous combinations
- "Backwards compatibility" — insecure defaults can't be grandfathered; deprecate loudly

## Taxonomy

### 1. Algorithm/mode selection
Parameters named `algorithm`, `mode`, `cipher`, `hash_type`; enums/strings selecting primitives. Canonical: JWT — attacker-controlled header selects `"alg": "none"`, or RS256→HS256 confusion turns the RSA public key into the HMAC secret. Root cause: untrusted input controls a security decision. Also: `hash($algorithm, $password)` accepts `"crc32"`; `password_hash($p, PASSWORD_DEFAULT)` offers no choice — good.

### 2. Dangerous defaults and magic values
Probe every security parameter with `0`, `""`, `null`, `[]`, `-1`:
- `lifetime=0` — accept-all or expire-immediately?
- `session_timeout: -1` — never expire?
- empty key/password that bypasses the check
- boolean defaults that disable security

Is the default the most secure option? Can any single value disable security entirely?

### 3. Primitive vs. semantic types
Same raw type (`bytes`, `string`, `[]byte`) for keys, nonces, ciphertexts, signatures — parameters swappable with no type error. Libsodium vs Halite: `sodium_crypto_box($msg, $nonce, $keypair)` lets you swap nonce/keypair or reuse nonces; `Crypto::seal($msg, new EncryptionPublicKey($key))` makes the wrong key a type error.

The comparison lookalike:
```go
if hmac == expected { }          // timing attack
if hmac.Equal(mac, expected) { } // constant-time
```
Same types, different security properties.

### 4. Configuration cliffs
- boolean flags that disable security entirely
- unvalidated strings: `verify_ssl: fasle` — typo silently truthy?
- dangerous combinations accepted silently: `auth_required: true` + `bypass_auth_for_health_checks: true` + `health_check_path: "/"`
- environment variables that override security settings
- constructor params with good defaults but no validation — `$hashAlgo = 'sha256'` still accepts `md5`; a default is not a validator. See [config-patterns.md](references/config-patterns.md#unvalidated-constructor-parameters).

### 5. Silent failures
- verify functions that return bool where sibling APIs throw — return value silently ignored
- `if not key: return True` — missing key skips verification
- default values substituted on parse errors
- verification that "succeeds" on malformed input

### 6. Stringly-typed security
Permissions/roles/scopes as comma-joined strings (`permissions += ",admin"`) instead of enums/sets; SQL, commands, and URLs built by concatenation.

## Adversaries

Evaluate each developer choice point against three:
- **Scoundrel** — controls config: can they disable security, downgrade algorithms, inject values?
- **Lazy developer** — copy-pastes the first example found: is it secure? Do error messages steer toward safe usage?
- **Confused developer** — can they swap parameters or pick the wrong key/mode by accident? Are failures loud?

Validate each finding: write the minimal misuse and confirm it creates a real vulnerability.

## Severity

| Severity | Criteria |
|----------|----------|
| Critical | Default or obvious usage is insecure (`verify: false` default; empty password accepted) |
| High | Easy misconfiguration breaks security (`algorithm` accepts "none") |
| Medium | Unusual but possible misconfiguration (negative timeout means never-expire) |
| Low | Requires deliberate misuse |

## References

By category:
- Cryptographic APIs: [references/crypto-apis.md](references/crypto-apis.md)
- Configuration patterns: [references/config-patterns.md](references/config-patterns.md)
- Authentication/session: [references/auth-patterns.md](references/auth-patterns.md)
- Case studies (OpenSSL, GMP, etc.): [references/case-studies.md](references/case-studies.md)

By language (general footguns, not crypto-specific):

| Language | Guide |
|----------|-------|
| C/C++ | [references/lang-c.md](references/lang-c.md) |
| Go | [references/lang-go.md](references/lang-go.md) |
| Rust | [references/lang-rust.md](references/lang-rust.md) |
| Swift | [references/lang-swift.md](references/lang-swift.md) |
| Java | [references/lang-java.md](references/lang-java.md) |
| Kotlin | [references/lang-kotlin.md](references/lang-kotlin.md) |
| C# | [references/lang-csharp.md](references/lang-csharp.md) |
| PHP | [references/lang-php.md](references/lang-php.md) |
| JavaScript/TypeScript | [references/lang-javascript.md](references/lang-javascript.md) |
| Python | [references/lang-python.md](references/lang-python.md) |
| Ruby | [references/lang-ruby.md](references/lang-ruby.md) |

Combined quick reference: [references/language-specific.md](references/language-specific.md)
