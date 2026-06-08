//! The dispatch seam — the one wire from a program back to its host.
//!
//! This is the shared op-execution contract: a ship-js isolate calls it to reach
//! the host, and the durable-execution engine calls it to run an op. It carries
//! no auth, no gate, no transport — the host (the agent runtime) implements
//! [`Dispatch`] over its capability gateway + credential and injects it, so the
//! credential is captured *inside* the impl and never crosses this boundary.
//!
//! It lives outside the `deno` feature on purpose: it's just a trait + an error,
//! so a consumer (e.g. `durable-execution`) can depend on this contract without
//! pulling V8.

use serde_json::Value;

/// A dispatch failure, classified for retry. ship-js itself doesn't retry — it
/// flattens any error to `{ error }` data for the isolate — but the contract
/// carries the classification because the durable-execution engine retries
/// `Transient` and never `Terminal` (a denial is data, not a thing to re-attempt).
#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    /// A transient blip — safe to retry (a flaky transport, lock contention).
    #[error("transient: {0}")]
    Transient(String),
    /// A terminal outcome — never retried (a denial, an unknown op, bad args).
    #[error("terminal: {0}")]
    Terminal(String),
}

/// The one wire from a program to its host: execute one op, return its result or
/// a classified failure. Implemented host-side; ship-js and the engine only call it.
#[async_trait::async_trait]
pub trait Dispatch: Send + Sync {
    /// Dispatch one op with JSON args; return the host's JSON result, or a
    /// classified failure.
    async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError>;
}
