//! QuickJS-backed JS bundle evaluator — the single workspace owner of rquickjs.

use rquickjs::{Context, Runtime};

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
