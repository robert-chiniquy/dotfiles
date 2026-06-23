---
name: custom-crypto-detection
description: "Reviewer persona for detecting hand-rolled cryptography. Distinct from `sharp-edges` (which catches footgun APIs) and `key-lifecycle-review` (which covers lifecycle hygiene): this skill catches the class where someone wrote their own MAC, KDF, AEAD, signature scheme, secret-comparison routine, RNG, or password hash. Almost all custom crypto is broken. Use when reviewing any code that does math on bytes, manipulates buffers in a 'crypto-shaped' way, or implements something whose docs reference a named primitive (HMAC, AES-GCM, Argon2, X25519). Triggers: hand-rolled crypto, custom MAC, custom hash, custom KDF, byte XOR, constant-time compare, derived key, password hashing, HKDF, encrypt_then_mac, mac_then_encrypt, AE, AEAD."
allowed-tools:
  - Read
  - Grep
  - Glob
---

# Custom Crypto Detection

Finds code that re-implements primitives that should have been called
from an established library. Custom crypto is almost always broken
even when written by people who know they shouldn't.

## When to use

- Diff touches files in `crypto/`, `auth/`, `security/`, `keys/`,
  `tokens/`, `password/`, `mac/`, `hash/`, `sign/`
- A new primitive name appears (MAC, HMAC, KDF, HKDF, PBKDF2, Argon2,
  scrypt, AEAD, AES-GCM, ChaCha20-Poly1305, X25519, Ed25519, ECDSA,
  RSA-PSS)
- XOR over byte buffers in non-trivial code (XOR is the canonical
  "this looks like crypto" signal)
- A function name contains `encrypt`, `decrypt`, `sign`, `verify`,
  `hash`, `mac`, `derive_key`, `random` and the implementation is
  more than a thin wrapper
- Comments mention "secure enough", "good enough for our purposes",
  "we couldn't use library X so we…"

## When NOT to use

- Calling well-known libraries (libsodium, BoringSSL, ring,
  RustCrypto, Tink) — those are out of scope; use `sharp-edges` if
  the API is used incorrectly
- Algorithm selection within a library — that's `key-lifecycle-review`
  or `sharp-edges`

## Core posture

If the diff contains custom crypto, the default verdict is "rewrite
to call a vetted library". Custom crypto is correct only when:

1. There is a documented threat model that the existing libraries
   demonstrably don't meet
2. The author is a cryptographer or has obtained a review from one
3. There are KAT (known-answer tests) and a fuzzing harness
4. The code is constant-time where needed and that's verified
5. There's a plan to replace with a vetted implementation when one
   becomes available

All four must hold simultaneously. If any is missing, the review
verdict is "this should not ship; use $LIBRARY".

## Common patterns to flag on sight

### Reimplemented HMAC

```python
def my_hmac(key, msg):
    return hashlib.sha256(key + msg).hexdigest()   # WRONG: length-extension
```

**Issue**: Length-extension attack. SHA-256 is a Merkle–Damgård hash;
appending `key+msg` and hashing is the classic vulnerable construction.

**Fix**: `hmac.new(key, msg, hashlib.sha256).hexdigest()`

### Naive secret comparison

```go
if userToken == storedToken { ... }     // WRONG: timing oracle
```

**Issue**: Byte-by-byte short-circuit allows network timing oracle to
recover the token.

**Fix**: `subtle.ConstantTimeCompare`, `crypto.timingSafeEqual`,
`hmac.compare_digest`, `subtle.ConstantTimeEq` (Rust)

### Roll-your-own KDF

```javascript
const key = sha256(password + salt + 'pepper');   // WRONG: not slow
```

**Issue**: Password hashing must be slow and memory-hard. SHA-256 is
neither.

**Fix**: Argon2id (preferred), scrypt, or bcrypt — never plain hash.

### Encrypt-then-XOR

```c
for (i = 0; i < len; i++) ciphertext[i] = plaintext[i] ^ key[i];
```

**Issue**: One-time-pad reuse the moment key is shorter than plaintext
or reused across messages. No integrity. No nonce. No authenticated
encryption.

**Fix**: AES-GCM or ChaCha20-Poly1305 from a vetted library.

### MAC-then-encrypt or encrypt-and-MAC

Modern AEAD modes do this correctly. Custom compositions are easy to
get wrong (chosen-ciphertext, padding oracles). Flag any non-AEAD
composition.

### Custom signature schemes / "encrypt with private key"

Encryption and signature are different operations even when the
math looks similar. RSA-encrypt-with-private-key is not a signature
scheme. ECDSA without nonce safety leaks the key.

### Custom random number generation

```python
def my_token(): return ''.join(chr(time.time_ns() % 256) for _ in range(32))
```

Any RNG that isn't a documented CSPRNG.

### Manual constant-time arithmetic

Even attempts at "constant-time" code can be miscompiled. If custom
crypto needs constant-time guarantees, the only valid answer is:

1. Use a vetted library that documents constant-time
2. Or pair the implementation with TOB's `constant-time-analysis`
   plugin and accept the cost

## Heuristic grep patterns

Run these as a first pass:

```
\bxor\b|\boplus\b
\bhash\(.*\+.*\)
\bencrypt.*xor
==.*token|==.*secret|==.*password
\.compare\(.*(token|secret|key|password|mac|hmac)
hashlib\.sha.*\+|md5\(.*\+
Math\.random|rand\(\)|random\.random
```

## Rationalizations to reject

| Rationalization | Reality |
|---|---|
| "It's just XOR / it's reversible by design" | XOR-based encryption is wrong unless it's a one-time pad with provably-fresh key material |
| "We don't have an HMAC library available" | Every modern stdlib has HMAC. If yours doesn't, your platform predates this concern. |
| "We control both sides" | Padding oracles, timing oracles, replay — these don't require attacker control of the protocol, just observation |
| "We've been running this in production for years without issue" | "Without observed issue." Crypto exploits are silent until used. |
| "It's only used for X" | Cryptographic primitives are misused when they're reused; the next maintainer extends the use case |

## Output format

For each finding:

- Location (file:line)
- Custom primitive identified (HMAC, KDF, AE, RNG, …)
- Concrete attack (one paragraph)
- Vetted replacement (specific function in a specific library)
- Severity: Custom crypto findings default to High; ship-blocking unless
  a documented threat-model justification exists

## References

- "Cryptography Engineering" (Ferguson, Schneier, Kohno) Ch. 1 for
  the philosophical posture
- libsodium / BoringSSL / RustCrypto / Tink — vetted-library catalog
- BSI Technical Guideline TR-02102; NIST SP 800-series for algorithm
  selection
- Latacora's cryptographic right-answers blog post

## Status

**v0.1 draft** — covers the most common rolled-your-own patterns.
Expansion: language-specific grep heuristics, FFI-wrapped C crypto,
WebCrypto subtleties, post-quantum primitive misuse.
