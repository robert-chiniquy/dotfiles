# Designing a userspace Occult application

How to design a program in application Occult: require stdlib models, write
the domain theory and the functors that realize it, keep host IO as a last
hop. This is not how to change the Occult engine.

Worked example: `plans/occult-g/DESIGN.md` (squine live-pass / env handle).

## Table of Contents

- [Reading path](#reading-path)
- [Four hops](#four-hops)
- [Middle functor](#middle-functor)
- [Handle threading is not a transport](#handle-threading-is-not-a-transport)
- [Last hop](#last-hop)
- [Product of machines](#product-of-machines)
- [Constructor raise is not G](#constructor-raise-is-not-g)
- [Start from an IO sketch](#start-from-an-io-sketch)
- [Existence proof first](#existence-proof-first)
- [Application Occult vs engine](#application-occult-vs-engine)
- [Contracts the sources leave implicit](#contracts-the-sources-leave-implicit)
- [Collapses](#collapses)
- [Sources](#sources)

---

## Reading path

Occult `docs/INDEX.md` is the front door. Three tracks: Thinking in Occult,
Language Guide, Engine Developer Guide. Then:

1. `docs/guide/EXPECTATIONS_VS_OCCULT.md` (imported priors)
2. Thinking in Occult: modules and lowering; C then K; KR levels 4 and 5
3. Language Guide: protocol model; bridges (`thompson`, `projection`)
4. Demos, in this order: `demo/echo/DEMO.md`, `demo/kv_terminal/` (planes),
   `demo/protocol_kv_io/DEMO.md`
5. `docs/protocol/GROUNDING_EXPLAINED.md`
6. `docs/guide/CELLS_STREAMS_AND_TURNS.md` if the host deposits named cells

`docs/howto/DISTRIBUTED_ALGORITHM.md` is a TCP backend recipe (`execute` →
`std.net`). It is not the language. Engine HOWTO is not language semantics
(Expectations: language vs engine).

## Four hops

A userspace application has four hops. The docs name them in different
files. Assembling them is the design.

1. Theory. A protocol, grammar, KV algebra, or other model. Transport-independent.
   Example: `echo = recv(msg) pipe send(msg)`.
2. Middle functor. Turns the theory into components or actors. KR level 4
   (product/component). Examples: `P_leader` / `P_follower` (protocol roles),
   Thompson (grammar → automaton), `C` (KV/crypto canonicalize),
   `compile_protocol` (protocol → ProtocolTerm handles).
3. Handle threading. `G(conn, io_send(d)) = io_send(conn, d)`, distributes
   over `pipe`. 1-arg IO becomes 2-arg IO. No transport in the rewrite.
   Grounding Explained cites Milner; this hop is a parameterized map from
   abstract IO to IO-on-a-handle.
4. Last hop. 2-arg IO onto a backend. `execute` → `std.net` is TCP.
   Terminal plane 3 is `std.io`. Actor demos use wire frames. A host that
   deposits JSON on `rendezvous(name)` is another backend.

Do not mix hop 3 with hop 4. `G` and `execute` are different functors
(handle threading vs Elliott compile onto `send_wire`).

A stream library (TCP, Unix pipe, TTY as one read/write API) is the same
split: the application talks to the stream; bind/connect/TTY-init are
backends that produce a handle. Occult does not use subtype inheritance.
The handle is a classifier on `conn`; each backend is an explicit last hop.

## Middle functor

`P_leader` is one middle functor, not the middle layer.

The job: a general theory becomes a product of components/actors. Language
Guide lists bridges (`thompson`, `projection`) without naming that job.
Thinking in Occult KR level 4 is the analysis name (product/component).
Level 5 is refinement/lowering (hops 3-4).

Name the actual actors. If the components are not leader and follower, do
not reuse `P_leader`. Duplicate send/recv-flip laws under honest names
(`P_laptop` / `P_inner`) rather than `P_laptop = P_leader`. Names are the
spec: a lying alias makes the next author dispatch the wrong protocol.

A second protocol in the same app may still use stdlib `P_leader` when its
actors really are those roles (inner squad collection round vs live-pass
pair). That is two uses of middle functors, not one alias.

## Handle threading is not a transport

Stdlib `grounding.occult` annotates `conn : Socket` because the echo demo's
handle is TCP. The rewrite does not mention TCP. `Socket` is the backend's
handle classifier, like typing a stream write as TCP-only.

Application `G` uses the domain handle (env, session, stdin bundle). Do not
require stdlib `grounding` unless `conn` is actually a `Socket`. Do not add
a new `G` to Occult stdlib; write it next to the application theory.

## Last hop

Pick one backend per 2-arg IO kind:

| Backend | Last hop |
|---|---|
| TCP | `execute` / `send_wire` / `recv_wire` |
| Terminal | `std.io` write / read_line (kv_terminal plane 3) |
| Actor | actor wire frames |
| Host HTTP or other non-userspace IO | named `rendezvous` cells; host produces, Occult raises |

Occult has no userspace HTTP. HTTP stays host IO onto a named cell. A native
is the last hop of IO with no userspace form, not a substitute for the cell
or for hops 1-3.

## Product of machines

Composed protocols must keep product state visible (Language Guide, protocol
model). Two conversations are two protocol terms and two middle functors.

Do not fuse a pair protocol (A sends work, A recv product) with a collection
round among children into one `work_round`. That hides a machine in transport
calls. KR level 4 fails: the factors are gone.

Roles are not phases. Two roles with three phases (laptop initiate/report,
inner control) stay two actors.

A handle that does not exist yet (listen vs accept, gateway vs env) is a
different `conn`. Do not thread `G(env, …)` through a create that yields
`env`. Stage that as a later level.

## Constructor raise is not G

Mapping verb × isolation to a surface *name* (`task_create`, `create_env`,
`local`) is raise/lower of constructors. It is not `pipe` sequencing.

Keep both. The name says which surface. `G` says send then recv on a handle.
Replacing `G` with more string aliases is vacuity (Occult as a string table
the host `Run`s).

## Start from an IO sketch

Write the app as abstract IO first, with names that are not yet grounded:

```
request = get("www.google.com");
result = process(request);
write(result);
```

Those names are free terms. `get`, `process`, `write`, `request`, and
`result` have no backend yet. The next step is to say what each one is:
a protocol action, a theory operation, a handle-threaded IO, or a last
hop. Occult has no userspace HTTP; `get("www.google.com")` in the sketch
does not become `std.http`. It becomes a free `get` whose last hop is a
named cell or a native the host already has.

Do not start by picking a native, an HTTP client, or a compiled module
blob (`.ocma`). The smallest program that is still the app is this
sketch plus a `check_*.occult` that reduces to `true`.

## Existence proof first

Echo's axiom test rewrites

```text
G(conn, P_leader(recv(d) pipe send(d)))
  = io_recv(conn, d) pipe io_send(conn, d)
```

before any socket. Application Level 0 is the same: a `check_*.occult` that
reduces to `true` on the rewrite. Host Dispatch and natives stay unchanged
until the rewrite exists.

Self-checks require the application module and do not call `entry()`.
`make lint-occult` at `-fail-on-warn`. Do not add a one-line catalog that
binds strings.

Then last hop: observe existing named cells in that order. Then new handles
(create/listen). Then classify over reply fields.

## Application Occult vs engine

Application Occult: `.occult` next to the app, `require` of protocol /
projection / json / host, domain theory, domain middle functor, domain `G`,
`rendezvous` names the host already opens.

Engine (owner review): new `syntax(...)` model, new `pipe` keyword, edit of
pin `grounding.occult` / `projection.occult` / `io_binding.occult`, userspace
HTTP, parser changes.

`require("protocol")` loads `pipe`. You do not add the keyword.

## Contracts the sources leave implicit

These are true of Occult. They are not stated as one stack in one file.

1. Four hops, not three. HOWTO and some demos jump from projection to
   `execute`/`std.net` and skip `G` as a separate hop.
2. Middle functor = theory → components/actors (KR level 4). `projection`
   is one bridge. Thompson, `C`, `compile_protocol` are the same job.
3. `G` is handle threading. Transport is hop 4. `conn : Socket` is echo's
   classifier.
4. Named stream cells are a legal hop-4 backend. Cells, streams, and turns
   is a separate document from Grounding Explained; the application that
   cannot do userspace HTTP uses both.
5. Constructor raise/lower and protocol `G` can coexist. One names; one
   sequences.
6. `G` (Milner, 1-arg IO to 2-arg IO) is not `execute` (Elliott, 2-arg IO
   to `send_wire`). Mixing the spellings collapses hop 3 into TCP.

## Collapses

1. `G` is TCP / the only grounding.
2. `P_leader` is the only middle functor.
3. `P_mine = P_leader` when the actors are not those roles.
4. `G` and `execute` as one spelling.
5. `ground(verb, isolation)` as a substitute for `pipe`.
6. Two machines written as one protocol term.
7. Design starts at the native, the HTTP client, or a compiled module blob.
8. HOWTO distributed / Engine Guide as the authoring model.
9. One `.occult` catalog file per rendezvous name.
10. New env and existing env as the same `conn`.

## Sources

Occult pin (`docs/INDEX.md` and files it names):

- Thinking in Occult (C then K; KR levels 4-5; modules and lowering)
- Language Guide (protocol; bridges)
- Grounding Explained; `model/stdlib/grounding.occult`, `projection.occult`,
  `io_binding.occult`
- `demo/echo/DEMO.md`, `demo/kv_terminal/`, `demo/protocol_kv_io/`
- Cells, streams, and turns
- Expectations vs the Occult reality
- Writing Demos (planes; machine visible in source)
- HOWTO distributed (TCP last hop only)

Squine:

- `docs/OCCULT_FACTORING_V2.md`
- `plans/occult-g/DESIGN.md`
- `occult/app.occult`, `occult/tests/check_*.occult`
