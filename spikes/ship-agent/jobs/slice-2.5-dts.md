# Job: scoped SDK `.d.ts` emitter (slice 2.5)

## Context
Work in `spikes/ship-agent/` — a deno_core-based Ship agent runtime spike (its
own Cargo workspace; `cd spikes/ship-agent` to build/test). `src/sdkgen.rs`
already generates a **JS** scoped SDK from an agent's capability grants
(`generate_sdk(grants) -> String`). The agent codes against this; the gate in
`gateway.rs` is the real enforcer (the SDK is the soft layer, do NOT add any
enforcement to it).

## Task
Add a **TypeScript declaration (`.d.ts`) emitter** alongside `generate_sdk`, so
the same `proto ∩ grant` surface is available as typed signatures for a language
server (tsgo). Name it `generate_dts(grants: &[String]) -> String`.

Requirements:
- For each granted service domain, emit a typed namespace whose methods take an
  args object and return `Promise<unknown>` (typed args can stay loose for now —
  the point is the *surface*, typed, scoped to the grant).
- `runtime` grant surfaces `export function agent(spec, input): Promise<...>` —
  present ONLY with that grant (mirror `generate_sdk`'s behavior exactly).
- The two generators MUST agree on which names exist for a given grant. Consider
  factoring the shared op-registry (`ops_for`) so they cannot drift.
- Keep the crate compiling and all existing tests green.

## Acceptance (behavioral tests — NOT brittle)
Add `#[cfg(test)]` tests asserting *semantics*, never counts or message strings:
- a granted service's name appears in the `.d.ts`; an ungranted one does not
- `agent` is declared only when `runtime` is granted
- empty grant => no declarations
- the set of exported names in `.d.ts` matches the set in the JS SDK for the
  same grant (the anti-drift property — derive both from the shared registry)

Run `cargo test` in `spikes/ship-agent`; all tests (existing + new) must pass.

## Constraints
- Touch only `src/sdkgen.rs` (or a new `src/dtsgen.rs` + its `mod`). Do NOT touch
  `gate.rs`, `gateway.rs`, `runtime.rs`, `spine.rs`, `auth.rs`, `module_loader.rs`.
- Pure string generation. No new heavy deps.
- Behavioral tests only. If a test hardcodes a count or an exact generated line,
  rewrite it to assert the behavior instead.
