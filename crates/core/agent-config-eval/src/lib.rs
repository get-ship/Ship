//! Evaluate a `ship.ts` agent config — authored with `defineAgent` — in a
//! locked-down ship-js isolate and harvest the result as
//! [`compiler::AgentProfile`]s.
//!
//! # The boundary
//!
//! `ship-js` is a standalone JS engine with **zero** knowledge of ship's config
//! types. THIS crate supplies everything ship-specific — the `@ship/sdk` stub
//! ([`SDK_STUB`]) and the [`AgentProfile`] harvest. The engine only runs a
//! closed-world module and returns the values it emits. The SDK is a *string
//! parameter* to the engine, never vendored into it.
//!
//! # The closed world
//!
//! - **Imports:** ship-js's deny-by-default loader resolves only `"ship"`, which
//!   it serves from [`SDK_STUB`]. Any other import fails to load.
//! - **Host wire:** [`DenyAll`] refuses every `ship_call` — config eval performs
//!   no side effects.
//! - **Determinism:** the stub reinstates a guard at module top level — which
//!   runs before the entry module's body, since ES modules evaluate their
//!   imports first — that blocks `Date.now`, `Math.random`, and zero-arg
//!   `new Date()`. The guard lives here (ship-supplied), not in the engine.

use std::sync::Arc;

use anyhow::Context;
use compiler::AgentProfile;
use ship_js::{Dispatch, DispatchError};

/// The ship-supplied `@ship/sdk` module — served to `import … from "ship"`.
///
/// It has two jobs, both at module top level so they run before the entry
/// module's body:
///
/// 1. Install the determinism guard.
/// 2. Export `defineAgent`, the collector: each call emits one config into the
///    engine's collector registry, harvested back on the Rust side after eval.
pub const SDK_STUB: &str = r#"
// --- determinism guard: config eval must be pure -------------------------
// Runs at SDK-module load, before the entry module's body (ES modules evaluate
// imports first), so authored config never observes an un-guarded clock/RNG.
const __block = (what) => () => {
  throw new Error(what + " is blocked in ship config eval — config must be deterministic");
};
Math.random = __block("Math.random()");
Date.now = __block("Date.now()");
{
  const _Date = Date;
  globalThis.Date = new Proxy(_Date, {
    construct(target, args) {
      if (args.length === 0) {
        throw new Error("new Date() is blocked in ship config eval — config must be deterministic");
      }
      return Reflect.construct(target, args);
    },
    apply() {
      throw new Error("Date() is blocked in ship config eval — config must be deterministic");
    },
  });
}

// --- defineAgent: the collector ------------------------------------------
// Each call harvests one agent definition into the engine's collector registry
// (op_emit), read back on the Rust side once the module settles. N calls — e.g.
// in a loop — collect N entries, in call order.
export function defineAgent(config) {
  Deno.core.ops.op_emit(config);
  return config;
}
"#;

/// A host dispatch that denies every op: config eval has no effects. A denied
/// op surfaces to the program as `{ error }` DATA (the engine's contract), so a
/// config that reaches for the wire keeps running but changes nothing.
struct DenyAll;

#[async_trait::async_trait]
impl Dispatch for DenyAll {
    async fn call(
        &self,
        op: &str,
        _args: serde_json::Value,
    ) -> Result<serde_json::Value, DispatchError> {
        Err(DispatchError::Terminal(format!(
            "host op `{op}` is denied during config eval"
        )))
    }
}

/// Evaluate a `ship.ts` config module and harvest its `defineAgent` calls as
/// [`AgentProfile`]s, in call order.
///
/// `source` is the config module. It must be JavaScript; a `.ts` file that uses
/// only JS-valid syntax (no `: Type` annotations, no `import type`) is accepted
/// as-is. Stripping TypeScript *type annotations* is a deferred follow-up — no
/// strip utility (oxc/swc) exists in-tree yet.
///
/// N `defineAgent(…)` calls — including inside a loop — roll up to N profiles.
pub async fn resolve_agents(source: &str) -> anyhow::Result<Vec<AgentProfile>> {
    let dispatch: Arc<dyn Dispatch> = Arc::new(DenyAll);
    let emitted = ship_js::deno::run_module(dispatch, SDK_STUB.to_string(), source.to_string())
        .await
        .context("config eval failed")?;

    let mut profiles = Vec::with_capacity(emitted.len());
    for (i, value) in emitted.into_iter().enumerate() {
        let profile: AgentProfile = serde_json::from_value(value)
            .with_context(|| format!("defineAgent call #{i} did not match AgentProfile"))?;
        profiles.push(profile);
    }
    Ok(profiles)
}
