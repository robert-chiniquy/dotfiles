---
name: rust-style
description: >-
  House Rust style: error handling, panics, output, module docs, comment
  density, test naming and layout, and the carrier-exposure rule for trait
  design. Load before writing or reviewing Rust in a repo the user owns.
  Triggers: writing Rust, reviewing a Rust diff, adding a crate or module,
  designing a Rust API, "does this match our Rust style".
---

# Rust style

Derived from the owner's pre-agentic Rust corpus (15 repos, 83k lines, 2021 to
2024) plus explicit decisions on the points where that corpus disagreed with
itself. Applies to repos the owner created. Contributions to other people's
repos follow that repo's conventions.

## Common Mistakes

Read this section first. Everything below it is elaboration.

1. **`unwrap` or `expect` in production code.** Out. A refusal is information.
   In tests they are fine.
2. **`Box<dyn Error>`.** Out, including as a test return type, which is where
   most of them accumulate. It erases the type to get past a boundary.
3. **`Result<_, String>`.** Out. Identity belongs in the term, not in a
   rendering of it.
4. **`println!` / `eprintln!` / `dbg!`.** Out. Diagnostics go through `tracing`.
   Exceptions are case by case, not by category. A CLI writing its actual output
   to stdout is not a diagnostic and is fine.
5. **A new module or crate with no `//!` header.** Every module and crate gets
   one, saying what the thing is for and what invariant it holds.
6. **A comment that restates the code.** Comments carry domain semantics: why
   this criterion, what this distinction means, what breaks if it changes. The
   mechanics are already on the screen.
7. **A `test_` prefix on a test name.** Redundant with `#[test]`. Name the test
   for the failure it catches.
8. **A type with bespoke methods where a std trait would do.** See
   [Expose the carrier](#expose-the-carrier). This is the most common and most
   expensive one.
9. **Encoding a solo-project habit as house style.** Missing CI, ungated test
   modules, and informal naming are what an exploratory repo looks like, not a
   preference. Do not propagate them.

## Errors

`thiserror` for library crates, `anyhow` for binary and CLI crates. Split by
crate role, not by author preference.

Errors carry identity as an enum variant. Document each variant with the
condition that produces it, not a restatement of its name. When a variant exists
because two failure causes are genuinely indistinguishable at that layer, say so
in the doc comment; that is exactly the fact a caller cannot recover on their
own.

`impl From<X> for MyError` is the propagation seam that makes `?` work. Prefer it
to a match that rewraps by hand.

## Panics

No `unwrap` or `expect` outside tests. Where a conversion is statically
infallible, prefer a form that shows why (a fixed-size array, a `const` bound)
over an `unwrap` with a comment promising it cannot fail.

`panic!` is for a violated internal invariant, not for an input the program
could receive.

## Output

`tracing`, not `println!`. Log messages, never values: a serialized struct in a
log line is how secrets escape, and it is noise even when it is not a hazard.

A CLI's real output is not a diagnostic. Writing results to stdout is the
program working, and it belongs on stdout, not in a log.

## Documentation and comments

Every module and crate gets a `//!` header. Doc comments on private items are
encouraged where the item carries a non-obvious lifetime, invariant, or
ordering.

Comment density should be high, and the target is domain semantics. The
distinctions worth writing down are the ones a reader cannot derive: why these
two states are different, what the criterion selects, which failure this guards.

Do not narrate the authoring process. No "an earlier version did X", no "first
attempt", no ruled-out hypotheses. State what is true.

Commented-out code does not get committed.

## Tests

Name the test for the failure it catches:
`open_fails_without_install_binding_sidecar`, `unlock_rejects_empty_and_short`,
`second_acquire_fails_while_held`. No `test_` prefix.

Before writing a test, state what bug it would catch. If there is no concrete
answer, the test is ceremony.

Layout: a sibling `tests.rs` per module, declared `mod tests;` in the parent,
gated with a file-wide inner `#![cfg(test)]` as the file's first line. Not a
per-item outer attribute, and never ungated, which compiles the test module and
its helpers into every build.

Integration and contract tests go in the crate's `tests/` directory, named for
the contract they pin rather than generically.

Never weaken a test to make it pass. A test that once caught something and was
loosened is worse than no test.

## Modules, imports, visibility

This is load-bearing, not cosmetic.

`pub(crate)` is the default. Public surface is assembled deliberately with a
curated `pub use` block at `lib.rs`, naming what is exported. Not
`pub use module::*;` glob re-exports, which make the public surface an accident
of what happens to be `pub` inside.

Imports group std, then external, then crate-local, blank-line separated. There
is usually no `rustfmt.toml` enforcing this, so it is maintained by hand.

`mod.rs` for nested module directories is fine and does not need migrating.

`build.rs` is approved when a real build-time input exists.

## Expose the carrier

The rule that changes designs, not just diffs.

Implement the std traits that let your type participate in general machinery,
rather than exposing bespoke methods that only you can drive. `Iterator`,
`IntoIterator`, `FromIterator`, `Display`, `From` and `TryFrom`, `AsRef`,
`Deref` where genuinely a smart pointer, and the operator traits when the
operation is really that operation.

The failure mode looks like a type with `fn all_current(&self) -> Vec<Item>`,
where `impl Iterator` would have let the caller filter, chain, zip, take, and
collect into whatever they needed. The method answers one question. The trait
answers every question of that shape, including the ones you did not think of.

The asymmetry is the argument: **narrowing is available to the caller, widening
is not.** A caller holding a full sequence can filter it. A caller holding your
one pre-filtered answer cannot get the rest back. So every narrowing performed
on the caller's behalf is irreversible, and every narrowing declined costs
nothing.

Stated as a rule: **return the most general thing you can justify, and let the
caller narrow.**

Corollary: serialize at the boundary that needs the representation, not before
it. Converting to `String` or an integer tag early is the same collapse.

## Deliberately unspecified

- Derive order. Unordered is fine.
- Concurrency idiom. No house rule; decide per problem.
- `mod.rs` versus flat module files.
