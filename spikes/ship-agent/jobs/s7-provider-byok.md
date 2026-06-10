# Job S7: Provider / BYOK credential resolution

## Context
Work in `spikes/ship-agent/` (own Cargo workspace). Read `README.md` (SSOT).
`run_step` in `agent_loop.rs` takes a generic `M: CompletionModel` (rig). We need
**BYOK credential resolution** and per-provider model construction so a real
provider model can be built and handed to `run_step` (today only the test
`MockCompletionModel` is used).

## Task — add `src/provider.rs` (+ `mod provider;` in main.rs)
1. Types:
   - `enum Provider { Anthropic, OpenAiCompat { base_url: String } }`
   - `struct Credential { provider: Provider, api_key: String, model: String }`
   - `trait CredentialSource { fn resolve(&self) -> anyhow::Result<Credential>; }`
2. `struct Byok` implementing `CredentialSource`, reading from a **provided
   key/value getter** (NOT global `std::env` directly — take a
   `impl Fn(&str) -> Option<String>` so tests don't mutate process env). Provide a
   thin `Byok::from_env()` convenience that wraps `std::env::var`.
   - `SHIP_PROVIDER` = `anthropic` (default) | `openai-compat`
   - keys: `ANTHROPIC_API_KEY` / `OPENAI_API_KEY`; `SHIP_BASE_URL` (openai-compat);
     `SHIP_MODEL` (model id).
3. Per-provider rig model construction behind the seam — a function that, given a
   `Credential`, builds the rig provider client + model. Because rig's provider
   model types differ, expose either an enum-dispatching helper or two typed
   constructors; the caller matches on `Provider`. Do NOT call a live API.
   - Add whatever rig provider features are needed to `Cargo.toml` (e.g. the
     anthropic / openai features). Keep additions minimal.

## Acceptance (behavioral tests — NOT brittle)
- `Byok` with a getter returning `SHIP_PROVIDER=anthropic` + `ANTHROPIC_API_KEY`
  + `SHIP_MODEL` resolves a `Credential` with the right provider/model.
- `openai-compat` resolves with `base_url` from `SHIP_BASE_URL`.
- a missing API key is an **error** (not a panic, not an empty key).
- default provider is anthropic when `SHIP_PROVIDER` unset.
- tests use the injected getter — they must NOT read or mutate global env.

No live-API test. Run `cargo test`; ALL (existing 53 + new) pass. Commit on this
branch when green.

## Constraints
- New file `src/provider.rs` (+ its `mod` line). You MAY edit `Cargo.toml` to add
  rig provider features. Do NOT touch `agent_loop.rs`, the enforcement modules
  (`gate.rs`/`gateway.rs`/`spine.rs`), execution (`runtime.rs`/`script_executor.rs`),
  `module_loader.rs`, or `session`/history files.
- Behavioral tests only. Credential resolution via injected getter, never global env.
- Update `README.md` seams table only if you change a seam.
