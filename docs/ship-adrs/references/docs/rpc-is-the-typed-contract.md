---
title: "RPC Is the Typed Contract (and the Gate)"
description: "ADR: the proto/RPC surface is the single source of the typed data contract AND the auth gate. Opaque JsonResponse returns and the un-typed SDK client are contract leaks that force the typed shape into side channels (specta TS + inline UI interfaces). Promote opaque responses to typed messages and buf-wire the SDK; specta survives only for types with no RPC home."
group: ADRs
order: 60
audience: internal
status: accepted
date: 2026-06-07
supersedes: []
tags: [rpc, proto, specta, sdk, contract-freeze, auth]
---

# RPC Is the Typed Contract (and the Gate)

## Context

Ship routes every operator/agent call through the proto RPC surface
(`proto/ship/ops/v1/*.proto`), and that surface is the **auth gate** — every
method carries a `capability` + `blast_radius`, and `services::command::call` is the
one place writes are gated. #89 collapsed 122 ops onto proto-SSOT; #79 set up a
buf-generated TS client to replace the specta TS emit.

But a survey of the surface against its TypeScript consumers shows the contract
**leaks around the gate** in two structural ways:

1. **42 of 141 RPC methods return `JsonResponse`** — a bare `{ json: string }` blob,
   not a typed message. By service: `git` (11), `pipelines` (8), `view` (6),
   `skills` (5), `isolation` (4), `sui` (3), and one each in
   `registry`/`permissions`/`mcp`/`events`/`agents`. These pass the auth gate but
   carry **no typed contract** — the caller gets an opaque string.

2. **The SDK client is not buf-wired.** `@ship/sdk-client` imports **zero**
   buf-generated proto types and exposes **33 `unknown`-typed methods**. So even
   where the proto *is* typed (e.g. `agents.detail` → `AgentDetail`), the SDK hands
   the UI `LiveSource<unknown>`.

The consequence is a **parallel system**: the typed shape every consumer needs is
reconstructed in two side channels neither of which is the gate —

- **specta-derived TS** (`cargo xtask gen-types` → `deck/sdk/generated.ts` etc.), and
- **inline UI interfaces.** The shipped `ship-agent-card.controller.ts` literally
  re-declares `interface AgentDetail` with the comment *"Mirrors the daemon
  AgentDetail shape"* because the SDK gave it `unknown`.

So a single logical contract (`AgentDetail`) exists in **three** places — the Rust
struct, the specta TS, and the inline UI interface — and the RPC, the thing that
*is* the gate, carries none of them typed. When the proto is the gate but not the
type, the type drifts; when the type lives off to the side, it evades the gate.

A separate read (the specta-retirement pass) found that the specta "keeps" — the
derives we can't yet delete — map almost exactly onto this leak: they are the typed
shapes for the 42 opaque responses, plus a genuinely-non-RPC residue (the `.ship/`
config-file schema, the view-authoring tool contract, the pipeline DAG).

## Decision

**The proto/RPC surface is the single source of the typed data contract, not just
the auth gate.** One contract per boundary; the gate and the type are the same
artifact.

1. **Promote opaque responses to typed messages.** Every `returns (JsonResponse)`
   that *can* be structured becomes a typed proto message. `git` is the clearest
   case: the typed messages (`GitOverview`, `StatusBuckets`, `GitDiffResponse`, …)
   **already exist** in `types.proto` — the methods just return `JsonResponse`
   instead of pointing at them.

2. **Keep open payloads, but only inside a typed envelope.** Genuinely
   schema-unknowable bodies stay JSON-bytes *by design*: `CollectionItem.data_json`,
   `CollectionDetail.schema_json`, `AgentDoc.doc_json`, `SettingValue.value_json`,
   feedback/annotation `event_json`. These are already wrapped in a typed message
   (`Item`, `AgentDoc`, …) — that is the correct pattern and is **not** a leak. The
   distinction is *opaque-by-design* (open payload in a typed envelope) vs
   *opaque-by-omission* (a bare `JsonResponse` where a message is possible).

3. **Buf-wire `@ship/sdk-client`.** The SDK returns the buf-generated proto types,
   not `unknown`. This kills the inline UI re-types at the source.

4. **specta is not a data path to the daemon.** It is retained *only* for types with
   **no RPC home** — the `.ship/` config schema (`compiler`), the view-authoring MCP
   contract (`ship-views`/`deck-host`), and the pipeline DAG (`ship-pipelines`).
   Anything that is a daemon read/write response belongs in proto. As promotion lands,
   the specta "keeps" shrink to that residue and the side channels are deleted.

## Consequences

### Positive
- One typed contract per boundary. The gate carries the type; drift between
  Rust/specta/inline-UI shapes becomes impossible.
- The UI stops hand-mirroring daemon shapes. `unknown` and inline `interface X`
  re-types disappear; components consume the generated client directly.
- specta's footprint collapses to the genuinely-non-RPC residue, then is bounded and
  documented rather than load-bearing-by-accident.
- Closes the conceptual gap behind #78 (delete REST/OpenAPI) and #90 (complete the
  unified shape): there is no second typed surface to keep in sync.

### Negative
- Promotion touches every opaque method's wire impl + handler (≈42 sites) and the
  corresponding fixtures/conformance.
- Buf-wiring the SDK is a TS-side migration with consumer churn (the inline re-types
  must flip to imports in lockstep).
- A few responses are awkward to type (heterogeneous "view-context" aggregates in
  `shipd`); some may stay opaque-by-design and must be *explicitly* marked so, not
  left ambiguous.

## Migration

Phased, additive, each gated by `cargo build --workspace` + `gen-types` +
`gen-fixtures --check` + `ship-contract-tests`:

1. **specta vestige sweep (in progress, `job/specta-retire`).** Delete derives that
   are proto-covered OR have no TS consumer. This *measures* the leak — what remains
   is the promotion backlog.
2. **`git` promotion (cheap first win).** Point the 11 `JsonResponse` methods at the
   already-existing typed messages in `types.proto`.
3. **Per-service promotion.** `pipelines` (8), `view` (6), `skills` (5),
   `isolation` (4), `sui` (3), then the singletons. For each: define/point the typed
   message, update the wire impl + handler, drop the corresponding specta derive.
4. **Buf-wire `@ship/sdk-client`.** Generate from buf; flip consumers off `unknown`
   and delete inline re-types (e.g. `ship-agent-card.controller.ts`).
5. **`ship-account`.** The blocked specta removal (discriminated-union vs flat-struct
   shape) resolves once the SDK consumes the proto `Me`/`AccountState` types.

Explicitly out of scope: the open-payload envelopes (§Decision-2) stay JSON-bytes.

## How to Measure Success
- `grep -c "returns (JsonResponse)" proto/ship/ops/v1/*.proto` → only the
  explicitly-documented open-payload cases remain (target: the 5 envelope fields, 0
  opaque-by-omission).
- `@ship/sdk-client` `unknown`-typed method count → 0.
- specta source-file count trends to the non-RPC residue (compiler config schema +
  view-authoring + pipeline DAG) and nothing else.
- No `// Mirrors the daemon …` inline interfaces in `packages/components/`.

## Alternatives Considered

- **Keep `JsonResponse` + lean on specta for the typed TS.** Rejected — this *is* the
  current state, and it is the parallel system: the type lives off the gate, drifts,
  and the UI re-mirrors it anyway. Two sources of truth for one contract.
- **Type everything in proto, including open payloads.** Rejected — proto cannot
  statically type N user-defined collection schemas or arbitrary agent-def JSONC.
  The typed-envelope-around-JSON-bytes pattern is the correct compromise and is
  already in use.
- **Generate the SDK from specta instead of buf.** Rejected — specta is a Rust-type
  exporter, not a wire contract; it has no notion of the auth gate, capabilities, or
  blast radius. The gate must be the source.
