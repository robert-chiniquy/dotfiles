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

Full treatment: `references/GUIDE.md`. Worked example (squine):
`docs/DESIGNING_USERSPACE_OCCULT_APPLICATIONS.md` and
`plans/occult-g/DESIGN.md` in a squine checkout.

## Common Mistakes

1. **`G` is TCP.** Stdlib `conn : Socket` is the echo demo's handle.
   `G` threads a handle; transport is the last hop.
2. **`P_leader` is the middle layer.** It is one theory-to-actors functor.
   Thompson, `C`, `compile_protocol` are the same job (KR level 4).
3. **`P_mine = P_leader` when the actors are not those roles.** Names are
   the spec. Write honest functor names; reuse stdlib `P_leader` only when
   the components really are leader/follower.
4. **Mix `G` and `execute`.** `execute` is the TCP backend (`send_wire`).
5. **`ground(verb, isolation)` as `pipe`.** Constructor raise names a
   surface. `G` sequences send/recv on a handle. Keep both.
6. **One protocol term for two machines.** Product state stays visible.
7. **Start from HTTP or a native.** Existence proof is a rewrite
   `check_*.occult` that reduces to `true`. Host Dispatch stays put until
   that rewrite exists.
8. **HOWTO distributed as language.** That file is `execute` to `std.net`.
   Read INDEX, then Expectations, Thinking in Occult, Language Guide,
   `demo/echo`, `demo/kv_terminal` planes. Cells/streams if the host
   deposits named ports. Named cells are a legal last hop (no userspace HTTP).
9. **Engine work in the app.** New `syntax(...)`, new `pipe` keyword, edit
   of pin `grounding.occult` / `projection.occult` / `io_binding.occult`,
   userspace HTTP: stop for owner review.

## Before finishing

- [ ] Four hops named (theory, middle functor, `G`, last hop)?
- [ ] Middle functor names the actual actors?
- [ ] Last hop is one backend, not `execute` unless the handle is TCP?
- [ ] Level 0 is a rewrite self-check, not a new native?
- [ ] Read `references/GUIDE.md`?
