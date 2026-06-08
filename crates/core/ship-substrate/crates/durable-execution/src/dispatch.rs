//! The op-dispatch seam — the engine's one door, and its only coupling to the
//! outside.
//!
//! The engine sequences and journals op *outcomes*; it does not know how an op is
//! authorized, gated, or transported. The host (the agent runtime) implements
//! [`OpDispatch`] over whatever it likes — its capability gateway + credential,
//! an RPC-as-IPC client, a test stub — and the engine calls it. So the engine
//! stays L0: no auth type, no gate, no transport crosses this boundary; the
//! credential is captured *inside* the host's impl, invisible here.
//!
//! This mirrors ship-js's `HostDispatch` in spirit (one wire to the host) but
//! carries a retryability-aware error, because the engine's retry policy must
//! distinguish a transient blip (retry) from a terminal outcome (a denial is
//! data — never retried).

use serde_json::Value;

/// A dispatch failure, classified for the engine's retry policy.
#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    /// A transient blip — safe to retry (a flaky transport, lock contention, a
    /// momentarily-unavailable peer). [`ExecCtx::op_with_retry`] re-attempts these.
    ///
    /// [`ExecCtx::op_with_retry`]: crate::engine::ExecCtx::op_with_retry
    #[error("transient: {0}")]
    Transient(String),
    /// A terminal outcome — NEVER retried. A capability denial, an unknown op,
    /// malformed args: a denial is data, and re-attempting it would only mask a
    /// real signal (or let a flake eventually slip a refused call through).
    #[error("terminal: {0}")]
    Terminal(String),
}

/// The engine's one door: execute one op and return its result, or a failure
/// classified for retry. Implemented host-side; the engine only ever calls it.
#[async_trait::async_trait]
pub trait OpDispatch: Send + Sync {
    /// Dispatch one op with JSON args; return the host's JSON result, or a
    /// classified failure.
    async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError>;
}
