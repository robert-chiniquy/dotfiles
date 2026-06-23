---
name: rust-unsafe-ffi-review
description: "Reviewer persona for Rust `unsafe` blocks and FFI (foreign function interface) boundaries. Catches the well-documented soundness violations: aliasing rule breaches, lifetime extension into 'static, raw-pointer arithmetic past bounds, `repr` mismatches with C, panic-across-FFI undefined behavior, transmute footguns, unsoundness from `Send`/`Sync` blanket impls, unwind-across-FFI, drop-on-uninitialized. Use when reviewing any PR that touches `unsafe { ... }`, `extern \"C\"`, `#[repr(C)]`, `Box::from_raw`, `Vec::from_raw_parts`, `mem::transmute`, `Pin`, `ManuallyDrop`, `MaybeUninit`, raw-pointer ops, or build.rs that bindgen / cc-rs / cxx integration. Triggers: unsafe rust, FFI, extern C, raw pointer, transmute, Box::from_raw, repr(C), bindgen, cxx, Send, Sync, aliasing, undefined behavior, miri."
allowed-tools:
  - Read
  - Grep
  - Glob
  - Bash
---

# Rust `unsafe` and FFI Review

The Rust compiler's safety guarantees apply only outside `unsafe`
blocks. Inside an `unsafe` block, the author has asserted that the
code upholds Rust's safety invariants. This skill verifies that
assertion.

The well-documented finding: ~72% of crates on crates.io depend
(transitively) on at least one `unsafe`-FFI binding. Most production
Rust memory-safety CVEs are in those bindings.

## When to use

- Diff touches any `unsafe { ... }` block
- New or changed FFI: `extern "C"`, `extern "system"`, `#[link]`,
  `bindgen`, `cxx`, `cc`, build scripts that compile native code
- `#[repr(C)]`, `#[repr(packed)]`, `#[repr(transparent)]` on
  structs/enums crossing FFI
- Raw pointers: `*const T`, `*mut T`, `Box::from_raw`,
  `Vec::from_raw_parts`, `slice::from_raw_parts`, `ptr::read`,
  `ptr::write`, `ptr::copy_nonoverlapping`
- `mem::transmute`, `mem::transmute_copy`
- `Pin<&mut T>` projections
- `MaybeUninit<T>`, `ManuallyDrop<T>`
- `unsafe impl Send` / `unsafe impl Sync` on a type
- `#[no_mangle]` exports

## When NOT to use

- Pure safe Rust — standard code review
- Async/concurrency in safe code (use `concurrency-truths`)
- C/C++ code on the other side of the FFI — use TOB's `c-review`
  and treat them as paired reviews

## Core posture

`unsafe` is a contract. The author owes the reviewer:

1. A documented safety comment on every `unsafe` block (or function),
   stating the invariants the caller must maintain (`# Safety` doc)
2. A demonstration that those invariants hold at every call site
3. A test that exercises the unsafe code under Miri, ASan, or both

If any of these is missing, the review verdict is "expand or remove
the `unsafe`."

## Soundness checklist

### Aliasing & lifetimes

- Mutable reference uniqueness: `&mut T` to a location while any other
  `&T` or `&mut T` to that location (or any overlap) exists = UB
- Raw pointer derived from `&mut T` does not extend the lifetime; the
  borrow checker still applies to the reference's scope
- `&'static` lifetimes synthesized via transmute / from_raw — verify
  the underlying memory really does live forever
- `Cell` / `RefCell` / `UnsafeCell` boundaries: only `UnsafeCell` is
  the canonical interior-mutability primitive; reaching into `&T` and
  writing through a raw pointer to non-`UnsafeCell` memory is UB

### Raw pointer hygiene

- `ptr.add(n)` / `ptr.offset(n)`: in-bounds of the same allocation,
  not past one-past-the-end
- `ptr.read()` / `ptr.write()`: pointer non-null, aligned to `T`'s
  alignment, points to initialized memory (for read) or valid storage
  (for write)
- `ptr.copy_nonoverlapping()`: non-overlap must hold
- `Box::from_raw(p)`: `p` must come from a previous `Box::into_raw` of
  the same `T` (or compatible); never a foreign allocator
- `Vec::from_raw_parts(p, len, cap)`: `p` must come from a Vec of the
  same `T` with the same allocator; `len <= cap`

### `MaybeUninit` discipline

- `MaybeUninit<T>::uninit().assume_init()` without writing first = UB
  even if `T` has only valid bit patterns
- Drop on partially-initialized memory — use `ManuallyDrop` to suppress
- Element-wise init of a `[MaybeUninit<T>; N]` then transmute to
  `[T; N]`: every element must be initialized

### Panic safety across `unsafe`

- A panic during an `unsafe` block can leave invariants broken (vec
  with `len` past initialized prefix; etc.)
- Use `ptr::write` instead of `=` assignment when the destination is
  uninitialized to avoid running `Drop` on garbage
- Catch unwinding at FFI boundaries: `catch_unwind` before crossing
  into `extern "C"` — unwinding into C is UB

### `Send` / `Sync` blanket impls

- `unsafe impl Send for T` requires: any thread can drop/move this
  without aliasing-with-original-thread concerns
- `unsafe impl Sync for T` requires: `&T` can be shared across threads
  with no data race
- Common violation: wrapping a `*mut T` and blanket-impl Send/Sync
  without considering whether the C side is thread-safe

### `transmute`

- Source and destination sizes equal at compile time (checked by
  compiler)
- Source's bit pattern is valid for the destination (NOT checked):
  - `u32 -> bool`: most values UB
  - `u8 -> NonZeroU8`: zero is UB
  - `&'a T -> &'static T`: lifetime extension is UB if the borrow
    doesn't actually outlive the use
  - `*const T -> &T`: T must be valid and aligned at *const T
- Prefer `mem::transmute_copy`, `ptr::read`, `From`/`TryFrom`, or
  `as` casts where they suffice; `transmute` is the last resort

### `Pin` projections

- Projecting `Pin<&mut Outer>` to `Pin<&mut Field>` is unsafe unless
  Field is `Unpin` or Outer's `Drop` impl is structurally compatible
- `Pin<&mut T>` does not let you move T out unless `T: Unpin`

## FFI-specific checklist

### `extern "C"` declaration consistency

- Argument and return types: layout-compatible with the C signature
  (e.g., `c_int` not `i32` unless platform-confirmed; `*mut c_char`
  not `&str`)
- Calling convention matches (`extern "C"` vs `extern "system"` on
  Windows)
- `repr(C)` on every struct that crosses the boundary
- Field alignment / padding matches C — verify with `mem::size_of` and
  `mem::align_of` against C's `sizeof`/`offsetof`
- Enums: explicit `repr(C)` or `repr(u32)` etc.; default Rust enum
  repr is not compatible with C `enum`

### Pointer ownership across FFI

- Allocated where, freed where, documented explicitly
- Don't: alloc with Rust's allocator, free with C's `free` (or vice
  versa)
- Use opaque pointers + a documented allocator for the boundary
- For strings: `CString::into_raw` → C frees via your-exported
  free-string function, NOT C `free()`

### Lifetime across FFI

- `&T` passed to C must outlive the call AND any pointer C stashes
- If C stashes the pointer (callback registration etc.), bridge via a
  `'static` (e.g., `Box::leak`) or a documented teardown protocol

### Callbacks from C to Rust

- The callback must not unwind: wrap in `catch_unwind` and convert
  panic to a C-friendly error
- Thread safety: if C might invoke from any thread, the Rust state
  it touches must be `Sync`
- Don't capture non-`'static` references in a callback that C stashes

### `bindgen` / `cxx` output

- Generated bindings: review the generated code, not just the wrapper
- bindgen sometimes generates unsound signatures (e.g., `*mut T` for
  what should be `*const T`); spot-check against the C header
- For `cxx`: review the .rs and .h sides together; trait bounds
  carry safety invariants

### Build script (`build.rs`)

- External tools (cmake, autoconf, custom scripts) run with the
  developer's privileges; review for what they do
- Output linking flags: avoid `-l<random>` from user-controllable input

## Tooling

Mandatory for diff containing `unsafe`:

- `cargo miri test` on the affected crate at least once
- `cargo +nightly miri test --target x86_64-apple-darwin` for ARM-on-x86
  to surface alignment assumptions baked into platform-specific
  unsafe blocks
- Address Sanitizer build: `RUSTFLAGS="-Zsanitizer=address" cargo +nightly test --target x86_64-apple-darwin`
- For FFI: paired with TOB's `c-review` plugin run against the C side

## Rationalizations to reject

| Rationalization | Why it's wrong |
|---|---|
| "It compiles" | UB compiles |
| "Miri passes" | Miri catches a lot, not all (FFI is largely opaque to Miri; some UB classes Miri doesn't model) |
| "We use it through a safe wrapper" | The wrapper is the contract; if the wrapper's invariants can be violated by safe callers, the `unsafe impl` is wrong |
| "It's just `transmute` for the byte representation" | Bit-pattern validity at the target type still required |
| "We need this for perf" | Most "perf needs unsafe" cases are wrong; benchmark with safe alternatives first |

## Output format

For each finding:

- Location (file:line, function name, the `unsafe` block's safety
  comment if one exists)
- Soundness class (aliasing / lifetime / raw-pointer / MaybeUninit /
  panic-safety / Send-Sync / transmute / Pin / FFI-signature /
  FFI-ownership / FFI-callback)
- Concrete UB scenario (one paragraph: what input or interleaving
  triggers it)
- Required fix (specific code change; remove the `unsafe`, expand the
  safety comment with the missing invariant, or change the data
  structure)
- Severity: soundness bugs default to Critical for shipping crates;
  High for internal-only

## References

- The Rustonomicon (`https://doc.rust-lang.org/nomicon/`) — read this
  before reviewing
- Rust UCG (Unsafe Code Guidelines) discussion / repo
- `cargo miri` book
- `RUSTSEC` advisory database — read it for patterns
- "Unsafe code review checklist" by Yoshua Wuyts (web)
- `cxx` and `bindgen` docs for FFI patterns

## Status

**v0.1 draft** — covers the canonical soundness violations and FFI
boundary classes. Expansion: per-platform alignment / endian
specifics; specific patterns for occult's native interop;
nightly-only features (e.g., custom allocators, strict provenance).
