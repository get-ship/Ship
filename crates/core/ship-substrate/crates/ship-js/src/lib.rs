//! The Ship JS engine (L0). Two backends, one crate:
//!
//! - **Default** — a QuickJS (`rquickjs`) bundle evaluator: [`eval`]/[`eval_global`]
//!   run a self-contained bundle and return its JSON result. Lightweight, `Sync`,
//!   no V8; this is the stateless batch-eval surface.
//! - **`deno` feature** — a deno_core/V8 [`deno`] runtime: a locked-down isolate
//!   whose only effect is the [`deno::HostDispatch`] wire, with a deny-by-default
//!   module loader and an `!Send` VM-thread bridge. This is the agent JS runtime
//!   (S16 Phase C1). It's opt-in because deno_core pulls ~200 transitive crates
//!   incl. V8, so the default build pays nothing.

use rquickjs::{Context, Runtime};

/// The deno_core / V8 backend — the agent JS engine. Gated behind the `deno`
/// feature; the whole subtree (runtime, module loader, `!Send` bridge, the
/// `HostDispatch` seam) is absent from the default build.
#[cfg(feature = "deno")]
pub mod deno;

#[derive(Debug, thiserror::Error)]
pub enum JsError {
    #[error("js evaluation failed: {0}")]
    Eval(String),
    #[error("result global '{0}' missing or not JSON-serializable")]
    MissingResult(String),
}

/// Evaluate a self-contained JS bundle and return the JSON value the entry
/// point produces. The bundle must assign its result to
/// `globalThis.__ship_result` (JSON-serializable).
///
/// A fresh QuickJS context is created per call. Context pooling is a future
/// optimization — YAGNI for v1 (cold paths only).
pub fn eval(bundle: &str) -> Result<serde_json::Value, JsError> {
    eval_global(bundle, "__ship_result")
}

/// Same as [`eval`], but reads `global` instead of the default `__ship_result`.
pub fn eval_global(bundle: &str, global: &str) -> Result<serde_json::Value, JsError> {
    let rt = Runtime::new().map_err(|e| JsError::Eval(e.to_string()))?;
    let ctx = Context::full(&rt).map_err(|e| JsError::Eval(e.to_string()))?;
    ctx.with(|ctx| {
        ctx.eval::<(), _>(bundle)
            .map_err(|e| JsError::Eval(e.to_string()))?;

        // JSON-encode the global name to safely use it as a property key.
        let key = serde_json::to_string(global).expect("global name is a valid JSON string");
        let script = format!("JSON.stringify(globalThis[{key}])");

        // JSON.stringify(undefined) → JS `undefined`, which rquickjs cannot
        // coerce to String → the map_err below fires → MissingResult.
        let json_str: String = ctx
            .eval(script.as_str())
            .map_err(|_| JsError::MissingResult(global.to_string()))?;

        serde_json::from_str(&json_str)
            .map_err(|_| JsError::MissingResult(global.to_string()))
    })
}
