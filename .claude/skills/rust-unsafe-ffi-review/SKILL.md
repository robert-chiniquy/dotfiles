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

Also in scope: `extern "system"`, `#[link]`, `#[no_mangle]`, `#[repr(packed)]`,
`#[repr(transparent)]`, `slice::from_raw_parts`, `ptr::read`/`write`/`copy_nonoverlapping`,
`unsafe impl Send`/`Sync`, build scripts compiling native code.
Not in scope: safe-code concurrency (use `concurrency-truths`); the C/C++ side
of the FFI (pair with TOB's `c-review`).

Every `unsafe` block or fn owes: a `# Safety` comment stating the invariants,
evidence they hold at every call site, and a test under Miri or ASan. Missing
any of these → verdict is "expand the safety comment or remove the `unsafe`."

## Soundness checklist

### Aliasing & lifetimes

- `&mut T` coexisting with any other reference to overlapping memory = UB
- A raw pointer derived from `&mut T` does not extend the borrow's scope
- `&'static` synthesized via transmute / from_raw: verify the memory really
  lives forever
- Writing through a raw pointer into non-`UnsafeCell` memory reached via
  `&T` = UB; `UnsafeCell` is the only sanctioned interior-mutability primitive

### Raw pointers

- `ptr.add`/`offset`: in-bounds of the same allocation, at most one-past-the-end
- `ptr.read`/`write`: non-null, aligned, initialized (read) / valid storage (write)
- `copy_nonoverlapping`: non-overlap must actually hold
- `Box::from_raw(p)`: `p` from a prior `Box::into_raw` of the same `T`;
  never a foreign allocator
- `Vec::from_raw_parts`: same `T`, same allocator, `len <= cap`

### MaybeUninit

- `uninit().assume_init()` without writing first = UB even if `T` has only
  valid bit patterns
- Drop on partially-initialized memory — suppress with `ManuallyDrop`
- `[MaybeUninit<T>; N]` → `[T; N]` transmute: every element initialized

### Panic safety

- A panic mid-`unsafe` can leave invariants broken (Vec `len` past
  initialized prefix, etc.)
- `ptr::write`, not `=`, into uninitialized destinations — assignment runs
  `Drop` on garbage
- `catch_unwind` before crossing into `extern "C"`; unwinding into C is UB

### Send / Sync

- Common violation: wrapping a `*mut T` and blanket-impl `Send`/`Sync`
  without verifying the C side is thread-safe

### transmute

- Size equality is compiler-checked; bit-pattern validity is NOT:
  `u32 → bool` (most values UB), `u8 → NonZeroU8` (zero UB),
  `&'a T → &'static T` (UB if borrow doesn't outlive use),
  `*const T → &T` (must be valid and aligned)

### Pin

- Projecting `Pin<&mut Outer>` → `Pin<&mut Field>` is unsafe unless
  `Field: Unpin` or Outer's `Drop` is structurally compatible
- No moving out of `Pin<&mut T>` unless `T: Unpin`

## FFI checklist

### Signatures & layout

- `c_int` not `i32`, `*mut c_char` not `&str`, unless platform-confirmed
- `extern "C"` vs `extern "system"` calling convention matters on Windows
- `repr(C)` on every crossing struct; verify `mem::size_of`/`align_of`
  against C `sizeof`/`offsetof`
- Enums crossing FFI need explicit `repr(C)`/`repr(u32)` etc. — default Rust
  enum repr is not compatible with C `enum`

### Ownership & lifetimes

- Allocated where, freed where — documented; never Rust-alloc + C `free`
  (or vice versa)
- `CString::into_raw` → freed via your exported free function, NOT C `free()`
- If C stashes a pointer (callback registration etc.): bridge via `'static`
  (`Box::leak`) or a documented teardown protocol

### Callbacks from C

- Must not unwind: wrap in `catch_unwind`, convert panic to a C error
- If C may invoke from any thread, touched Rust state must be `Sync`
- No non-`'static` captures in a callback C stashes

### bindgen / cxx / build.rs

- Review the generated bindings, not just the wrapper; bindgen sometimes
  emits `*mut T` where the header means `*const T` — spot-check the C header
- cxx: review the .rs and .h sides together; trait bounds carry safety
  invariants
- build.rs: don't derive `-l` link flags from user-controllable input

## Tooling

Mandatory for any diff containing `unsafe`:

- `cargo miri test` on the affected crate. Caveat: FFI is largely opaque to
  Miri and some UB classes aren't modeled — passing Miri is not a proof
- Cross-target run to surface baked-in alignment assumptions:
  `cargo +nightly miri test --target x86_64-apple-darwin` (from ARM)
- ASan: `RUSTFLAGS="-Zsanitizer=address" cargo +nightly test --target x86_64-apple-darwin`
- FFI diffs: pair with TOB's `c-review` run against the C side

RUSTSEC advisory database is a good source of real-world unsafe/FFI bug
patterns.

## Output format

Per finding: location (file:line, plus the block's safety comment if any);
soundness class (one of the checklist sections above); concrete UB scenario
(what input or interleaving triggers it); required fix. Severity: soundness
bugs default to Critical for shipping crates, High for internal-only.
