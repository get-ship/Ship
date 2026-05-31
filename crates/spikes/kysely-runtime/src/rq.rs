//! rquickjs hosting layer — bundles Kysely + a query script into one ESM,
//! evaluates inside QuickJS, returns the OperationNode JSON.
//!
//! Why this exists: agents need to produce Kysely ASTs at runtime. In
//! production Ship may or may not include rquickjs, but proving rquickjs
//! can host Kysely is the answer to "can we use Kysely without shipping a
//! heavy JS runtime."
//!
//! Approach: caller passes a pre-bundled ESM string (kysely + the query)
//! and the bundle exports `queryAst` as a JSON string. We evaluate, read
//! the global, return it. The bundling itself is out of scope for the
//! spike — see `js/bundle.mjs` for the bun-based bundler that produces
//! the test bundle.

use rquickjs::{Context, Runtime};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RqError {
    #[error("rquickjs runtime: {0}")]
    Runtime(String),
    #[error("script did not export queryAst")]
    NoExport,
}

/// Evaluate a self-contained ESM bundle that ends with
/// `globalThis.__queryAst = JSON.stringify(<query>.toOperationNode())`
/// and return the AST JSON string.
pub fn run_bundle(bundle: &str) -> Result<String, RqError> {
    let runtime = Runtime::new().map_err(|e| RqError::Runtime(e.to_string()))?;
    let ctx = Context::full(&runtime).map_err(|e| RqError::Runtime(e.to_string()))?;
    ctx.with(|ctx| {
        ctx.eval::<(), _>(bundle)
            .map_err(|e| RqError::Runtime(format!("eval: {e}")))?;
        let globals = ctx.globals();
        let s: String = globals
            .get("__queryAst")
            .map_err(|_| RqError::NoExport)?;
        Ok(s)
    })
}
