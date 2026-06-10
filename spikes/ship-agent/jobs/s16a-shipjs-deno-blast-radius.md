# Job S16.a — ship-js → deno blast-radius (findings)

> Status: **done, inline** (omp was degraded; investigated directly).
> Verdict: **the swap is risk-free — `ship-js` has zero consumers.**

## 1. Verdict
Replacing `ship-js`'s engine (rquickjs → deno_core) breaks **nothing**, because
**nothing depends on `ship-js` today.** It is a workspace member that is built but
unwired — a placeholder waiting to be the engine. There is no migration to
sequence and no consumer to preserve compatibility for; we design the new API
fresh.

## 2. Evidence (verified)
- `Cargo.toml` (root): `ship-js` is a workspace member (line 14).
- **No crate depends on it.** `grep ship-js|ship_js` across every `Cargo.toml` →
  only `ship-js`'s own manifest. `grep 'ship-substrate/crates/ship-js'` across all
  manifests → nothing.
- **No source references it.** `grep 'ship_js::'` across all `.rs` (excluding the
  crate itself) → empty. No `pub use ship_js` re-export in `ship-substrate`.
- So: zero Cargo deps, zero source refs, zero re-exports = **unused.**

## 3. The current API (what we're replacing)
`crates/core/ship-substrate/crates/ship-js/src/lib.rs` — 44 lines, two fns:
- `eval(bundle: &str) -> Result<Value, JsError>` — fresh QuickJS context per call;
  the bundle assigns to `globalThis.__ship_result`; the crate `JSON.stringify`s it
  back. **Synchronous, value-in/value-out, self-contained bundle.**
- `eval_global(bundle, global)` — same, custom result global.
- The `rquickjs` `loader` feature is enabled but **the API never uses imports** —
  it's eval-a-self-contained-bundle. No async, no ops/FFI, no module loader in use.

So the existing surface is the trivial case; deno_core covers it and far more
(our spike's `runtime.rs`/`script_executor.rs`/`module_loader.rs` already do
imports, async ops, deny-by-default `import "ship"`, the `!Send` VM-thread bridge).

## 4. The V8-weight question — moot for now
The one risk the brief flagged (a consumer for whom QuickJS's lightness is the
point) **does not exist** — there are no consumers. V8 isolate weight is therefore
a *forward design* concern for the agent runtime + pipeline engine, not a
regression. It's handled by the architecture's isolate-pooling + startup-snapshot
plan (`s16-architecture.md §1`, the many-agents-per-process requirement), not by
keeping rquickjs.

## 5. Migration shape + recommendation
**Clean swap. `ship-js` becomes the home for the deno runtime.**
- Graduate the spike's `runtime.rs` + `script_executor.rs` + `module_loader.rs`
  into `ship-js` (replacing the rquickjs impl outright). `ship-js` stays the
  "single workspace owner of the JS engine" — deno_core instead of rquickjs.
- **Redesign the API freely** (no compat constraint): async eval, the `op_ship_call`
  FFI bridge, the scoped-SDK module loader. Optionally keep a thin synchronous
  `eval(bundle) -> Value` shim for the simple self-contained-bundle case, but no
  consumer needs it today.
- `rquickjs` leaves the workspace; `deno_core`/`deno_error` enter as the engine
  deps (the spike already pins them).

Blast radius: the `ship-js` crate itself + adding the deno deps to the workspace.
Nothing downstream. This is the safest possible "swap the engine under a crate"
because the crate has no users.

## 6. References
- `crates/core/ship-substrate/crates/ship-js/{Cargo.toml,src/lib.rs}` — the crate.
- `spikes/ship-agent/src/{runtime,script_executor,module_loader}.rs` — the deno
  runtime that graduates in.
- `spikes/ship-agent/jobs/s16-architecture.md` §2 — "ship-js becomes deno."
