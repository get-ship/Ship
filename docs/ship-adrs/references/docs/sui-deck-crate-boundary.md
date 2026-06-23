---
title: "SUI / Deck Crate Boundary"
description: "ADR: the Rust workspace mirrors the JS package split — /home/madvib/dev/sui/crates/sui/ is the view framework (pure leaves), /home/madvib/dev/sui/crates/deck/ is the meta-framework that hosts, serves, and bundles SUI views. The host/server/facade crates move out of crates/ship/ and lose their ship-sui-* prefix."
group: ADRs
order: 40
audience: internal
status: accepted
date: 2026-05-20
supersedes: []
tags: [crate-layout, sui, deck, meta-framework, naming]
---

# SUI / Deck Crate Boundary

## Context

Ship has two distinct framework concepts that the codebase already names on the
JS side but not on the Rust side:

- **SUI** — the view framework: the `.sui` language, its AOT compiler, lint
  rules, and the reactive signal runtime. Pure, portable, WASM-safe where it
  matters. The "React" of the stack.
- **Deck** — the meta-framework built *on* SUI: it resolves views, SSRs and
  hosts them, serves them over HTTP, bundles assets, builds the component
  catalog, generates the SDK, and provides LSP + eval. The "Next.js" of the
  stack.

`/home/madvib/dev/sui/packages/sui/{grammar, compiler-wasm, dom, runtime}` versus
`packages/deck/{build, bundler, catalog, sdk, sdk-client, lsp, lsp-browser, eval}`
(the last group lives in `/home/madvib/dev/sui` post-carve).

`crates/` did **not**. It had `/home/madvib/dev/sui/crates/sui/{compiler, lint, signals}` (the pure
leaves) but the Rust *deck* tier was misfiled under `crates/ship/` with vestigial
`ship-sui-*` names:

| Crate (before) | What it is | `src/` |
|---|---|---|
| `ship-sui-host` | view resolution, bundle orchestration, SSR, payload assembly | `resolver.rs`, `bundle.rs`, `view_payload.rs`, `render_helpers.rs` |
| `ship-sui-server` | axum HTTP handlers the daemon mounts | — |
| `ship-sui` | thin facade over host + server | — |

None of these is the SUI framework — they are the layer *above* it that
orchestrates it (they depend on `sui-compiler` + `sui-signals`). That is exactly
Deck. The `ship-sui-*` prefix predated the sui/deck split being named and was
misleading. `/home/madvib/dev/sui/crates/sui/README.md` compounded the confusion: it listed all seven
crates as one `/home/madvib/dev/sui/crates/sui/` cluster and still described a *Jinja + Datastar*
architecture that no longer exists (the runtime is ESM + signals).

## Decision

Mirror the JS split in the Rust workspace. Move the Deck Rust tier out of
`crates/ship/` into a new `/home/madvib/dev/sui/crates/deck/` cluster and rename to match the
concept:

```
crates/ship/ship-sui-host    →  /home/madvib/dev/sui/crates/deck/deck-host     (lib: deck_host)
crates/ship/ship-sui-server  →  /home/madvib/dev/sui/crates/deck/deck-server   (lib: deck_server)
crates/ship/ship-sui         →  /home/madvib/dev/sui/crates/deck/deck          (lib: deck)   [facade/umbrella]
```

`crates/ship/` held only these three, so it emptied out and was removed. The
resulting top-level convention is symmetric:

```
/home/madvib/dev/sui/crates/sui/   = SUI view framework (Rust)   ↔  /home/madvib/dev/sui/packages/sui/   = SUI (JS)
/home/madvib/dev/sui/crates/deck/  = Deck meta-framework (Rust)  ↔  packages/deck/  = Deck (JS)
crates/core/  = substrate + platform tier (kernel, pipelines, supervisor, …)
```

`/home/madvib/dev/sui/crates/sui/README.md` is rewritten around this boundary and de-staled (no more
Jinja/Datastar framing; describe the ESM + signals reality and the sui-vs-deck
line).

## Consequences

### Positive

- One mental model, stated once: SUI is the framework, Deck is the platform on
  top of it. Crates and packages agree.
- Crate names tell the truth about their role. A reader who knows "deck = the
  thing that serves views" finds `deck-host`/`deck-server` immediately.
- `crates/ship/` stops being a catch-all; the `ship-*` prefix is freed for
  genuine product-level crates if ever needed.

### Negative

- Workspace-wide rename: root `Cargo.toml` members, inline path deps in
  `apps/shipd` and `crates/xtask`, and every `use ship_sui*` import (~12 shipd
  files) change in one commit. Mechanical but broad; conflicts with any branch
  touching those crates while it lands.
- Churn in `git blame` for the moved files (mitigated by `git log --follow`).

## Alternatives Considered

- **Leave it.** Rejected: the names actively mislead, and the asymmetry with
  `packages/` is a recurring source of "where does this live" confusion.
- **Keep them in `crates/ship/` but rename to `deck-*`.** Rejected: half-measure
  — the directory still implies a "ship" tier that doesn't exist, and the
  packages/crates symmetry is the whole point.
- **Fold host+server into a single `deck` crate.** Rejected: the host (no HTTP,
  WASM-relevant) and server (axum) split is a real layer boundary worth keeping;
  the facade gives dependents one name to import.

## Migration

Single job (`rust-lane`): move the three crate dirs, rename packages + lib
targets, update root `Cargo.toml` members, update inline path deps in
`apps/shipd` + `crates/xtask`, rewrite every `ship_sui*` import to `deck*`,
remove the empty `crates/ship/`, and rewrite `/home/madvib/dev/sui/crates/sui/README.md`. Validated
by `cargo build --workspace` + `cargo test --workspace`.
