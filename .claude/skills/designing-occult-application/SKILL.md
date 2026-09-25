---
name: designing-occult-application
description: >
  Design a userspace pure Occult application: theory, middle functor
  (theory to components/actors), handle-threading G, last-hop backend.
  Use when writing application Occult, a new G or projection analog,
  protocol lowering in an app (not the Occult engine), or before adding
  .occult next to a host. Triggers: userspace Occult, application Occult,
  Occult analog, G(env), middle functor, protocol planes.
---

# Designing a userspace Occult application

Four hops: theory → middle functor (theory to components/actors) → `G`
(handle threading) → last-hop backend. Application Occult requires stdlib
models. It does not add syntax, a `pipe` keyword, or a stdlib `G`.

Start with the most pure, most minimal program that is still the app: an
abstract IO sketch, then free terms, then a `check_*.occult` that reduces
to `true`. Host, natives, HTTP, and compiled module blobs stay last.

Full treatment: `references/GUIDE.md`. Worked example (squine):
`docs/DESIGNING_USERSPACE_OCCULT_APPLICATIONS.md` and
`plans/occult-g/DESIGN.md` in a squine checkout.

## Common Mistakes

1. **Start at the native, HTTP, or a compiled module blob.** First write
   the IO sketch with free names, then say what those names mean. A
   `check_*.occult` that reduces to `true` is the existence proof. Host
   Dispatch stays put until that rewrite exists.
2. **`G` is TCP.** Stdlib `conn : Socket` is the echo demo's handle.
   `G` threads a handle; transport is the last hop.
3. **`P_leader` is the middle layer.** It is one theory-to-actors functor.
   Thompson, `C`, `compile_protocol` are the same job (KR level 4).
4. **`P_mine = P_leader` when the actors are not those roles.** Names are
   the spec. Write honest functor names; reuse stdlib `P_leader` only when
   the components really are leader/follower.
5. **Mix `G` and `execute`.** `execute` is the TCP backend (`send_wire`).
6. **`ground(verb, isolation)` as `pipe`.** Constructor raise names a
   surface. `G` sequences send/recv on a handle. Keep both.
7. **One protocol term for two machines.** Product state stays visible.
8. **`occult/testdata`.** Go `testdata/` is for Go golden files.
   Application Occult self-checks live in `occult/tests/`. Do not
   create `occult/testdata/`.
9. **HOWTO distributed as language.** That file is `execute` to `std.net`.
   Read INDEX, then Expectations, Thinking in Occult, Language Guide,
   `demo/echo`, `demo/kv_terminal` planes. Cells/streams if the host
   deposits named ports. Named cells are a legal last hop (no userspace HTTP).
10. **Engine work in the app.** New `syntax(...)`, new `pipe` keyword, edit
   of pin `grounding.occult` / `projection.occult` / `io_binding.occult`,
   userspace HTTP: stop for owner review.
11. **Unrolling a projection into concrete arities.** `resp_array1` /
    `resp_array2` / `resp_array3` are three concrete arities of RESP `*N`, not
    an array. Write `resp_array(items)` and constrain command arity on that
    vector. Same trick as `w1`/`w4`.
12. **JSON as the object model.** `J.put(J.object(), ...)` records standing in
    for domain terms. JSON is a wire format and a data payload only. Domain
    terms use the engine's userspace constructor idiom (tagged vectors with
    accessor equations, as in `userspace/tool_service`); a separate wire
    module lowers them to JSON at the last hop and raises replies from it.
    Caller payloads that are JSON by contract (a state blob) stay opaque.
    A bare `∃ ctor;` with no equation lints as `callable-no-meaning`.
13. **One `Run` per turn.** Depositing a request, calling `Runtime.Run` on
    the source, and reading the result (from the return or from an output
    native) is per-request re-evaluation, the debt Cells, streams, and turns
    names; `FrameRunner` and the actor are the same thing per frame. The
    host runs one program for the process: a recursive session loop whose
    input is a blocking host io native (re-entered on every demand, like
    `std.io` `read_line`) and whose output is a host io native, with a
    host-produced observation on its own rendezvous cell written before the
    message that reads it. A loop over a rendezvous cell does not work: a
    cell is latest-value inside one evaluation. Measured in occult-jev at
    4.4 ms per message over 2000 messages with the module count flat
    (`occult/io_pump_test.go`). Build that fixture before proposing any
    per-turn form.

## Before finishing

- [ ] IO sketch first, then free terms, then a `check_*.occult` that is true?
- [ ] Four hops named (theory, middle functor, `G`, last hop)?
- [ ] Middle functor names the actual actors?
- [ ] Last hop is one backend, not `execute` unless the handle is TCP?
- [ ] Self-check files live in `occult/tests/`, not `occult/testdata/`?
- [ ] Read `references/GUIDE.md`?
