//! The host-dispatch seam — ship-js's only effect channel.
//!
//! A ship-js isolate has exactly ONE bound effect: it can call back into the
//! host with `(op, args)` and receive a JSON value. ship-js owns no policy, no
//! auth, no gateway — it is a sandbox with a single wire. The host (the agent
//! runtime) implements [`HostDispatch`] over whatever it likes — its capability
//! gateway, an RPC-as-IPC client, a test stub — and injects it per run.
//!
//! This is what keeps ship-js **L0**: the credential and the gate live entirely
//! host-side, captured *inside* the `HostDispatch` impl. ship-js never imports
//! the agent's `Gateway` or `AuthContext`; it sees only `op: &str, args: Value`,
//! and collapses every host failure to a flat [`HostError`] — the structured
//! shape (denial vs. unknown-op vs. approval-required) stays the host's business.

use serde_json::Value;

/// A flat host-side failure. The op layer turns this into **data** — the isolate
/// observes `{ "error": "<msg>" }`, never a thrown host exception — so a denial
/// is a result the program handles. ship-js deliberately does not model the
/// host's error taxonomy; one string is the whole contract.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct HostError(pub String);

impl HostError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

/// The one wire from a ship-js isolate back to its host.
///
/// `dispatch` returns `Ok(value)` for a result or `Err(HostError)` for a
/// failure; the op layer maps `Err` to `{ "error": .. }` uniformly, so every
/// host gets denial-as-data without re-implementing the convention.
#[async_trait::async_trait]
pub trait HostDispatch: Send + Sync {
    /// Dispatch one op with JSON args; return the host's JSON result, or a flat
    /// error the op layer will surface as data.
    async fn dispatch(&self, op: &str, args: Value) -> Result<Value, HostError>;
}
