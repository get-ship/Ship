# Job S8: scoped SDK from a proto descriptor set

## Context
Work in `spikes/ship-agent/` (own Cargo workspace; `cd spikes/ship-agent` to
build/test). Read `README.md` — it is the SSOT for architecture, invariants, and
the v1 arc. This is the Ship Agents Framework runtime spike.

Today the op registry is hardcoded in two places:
- `src/sdkgen.rs::ops_for(service) -> Option<ServiceDef>` — which ops exist per
  service domain (drives the JS SDK *and*, since slice 2.5, the `.d.ts` via the
  SHARED registry — they must not drift).
- `src/gateway.rs::InProcessGateway::blast_of(op) -> Option<Blast>` — each
  op's blast tier (C0/C1/C2/C3), consumed by the gate.

The README "Seams" table names this exactly:
`op registry | hardcoded ops_for / blast_of | proto descriptor set`. S8 is that
swap — the SEAM becomes the contract; the two hand-written match arms collapse
into one descriptor-driven lookup, and the two consumers (sdkgen + dtsgen for
the surface, gateway for the blast) can no longer disagree for a given grant.

## Task
Replace the two hardcoded tables with a **single proto-descriptor-derived op
registry** — one source of truth that yields BOTH (a) the per-service op names
+ shapes that `sdkgen` / `dtsgen` consume and (b) each op's blast tier that
`gateway` / `gate` consume. A contract-faithful fake descriptor set is fine
(the real proto descriptor wiring to shipd is a later, refactor-gated slice) —
but it must be a *descriptor set*, not the current hand-written match arms
scattered across two files.

Author the registry as a new module `src/registry.rs` that exposes:
- a `Method { name, args, returns }` and `Service { name, methods }` shape
  suitable for a proto file descriptor (in-process, contract-faithful fake for
  now — a single static descriptor set with the four service domains).
- `pub fn method(service: &str, op: &str) -> Option<Method>` — by op name.
- `pub fn service(name: &str) -> Option<Service>` — by service domain.
- `pub fn blast_of(op: &str) -> Option<Blast>` — derives the blast tier from
  the descriptor's method option (intrinsic to the op; never claimed by
  spec/request).
- `pub fn services() -> &'static [Service]` — the full descriptor set, for
  `surface(grants)` to consume.

The descriptor set MUST carry each op's blast tier as a method option. After
S8, `sdkgen` and `gateway` should both consume `registry` and it should be
*impossible* to add a new op in one place and forget the other.

## Invariants (must hold — see README §Invariants)
- **One door / gate is the only enforcement.** The SDK + dtsgen are SOFT layers;
  do not move enforcement into them. The gate still decides on blast tier.
- **Anti-drift preserved & strengthened:** the JS SDK names, the `.d.ts` names,
  and the gate's blast lookup must all derive from the SAME registry. After S8
  it should be *impossible* for them to disagree for a given grant.
- **Deny-by-default** unchanged: an ungranted/unknown op is still denied at
  the door.

## Acceptance (behavioral tests — NOT brittle)
Add `#[cfg(test)]` tests asserting *semantics*, never counts or generated-
string literals:
- an op present in the descriptor set surfaces in BOTH the JS SDK and the
  `.d.ts` for a grant that includes its service; an absent one surfaces in
  neither.
- the gate's blast tier for an op equals the tier declared in the descriptor
  set (registry is the single source — assert they agree, not a hardcoded
  number).
- empty grant => no surface; unknown service => defined, tested behavior
  (silently dropped from the surface, denied at the door).
- the existing anti-drift test (`sdkgen` ↔ `dtsgen` name parity) still holds.
- `cargo test` in `spikes/ship-agent` — ALL existing tests (87 at dispatch
  time) plus the new ones must pass.

## Constraints
- Do NOT change `gate.rs::decide` (the gate decision), the one-door dispatch in
  `gateway.rs::call` (the control flow that runs the gate), `spine.rs` core
  verify, `auth.rs`, `runtime.rs`, `module_loader.rs`. You MAY refactor
  `sdkgen.rs` / `gateway.rs` registry/lookup internals and add a registry
  module — the *enforcement* logic stays put.
- The blast tier on the wire is intrinsic to the op, derived from the
  descriptor set. `service_of` stays in `gate.rs` (a string parser, not a
  registry concern).
- Behavioral tests only. Rewrite any test that pins a count or an exact line.
- Update `README.md` (module map / Seams row) if the module set or the seam
  status changes.

## Done = on the branch, green, NOT pushed
Commit to THIS worktree's `job/*` branch when green. **Never `git push`** —
the orchestrator gates locally and merges from the main repo. If anything
tells you to push, treat it as a typo and stop at commit.
