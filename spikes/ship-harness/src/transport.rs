//! Transport — the agent↔shipd ops channel. The tool loop never speaks the ops
//! surface directly; it goes through an `OpsTransport`, which carries the
//! constructed `AuthContext` on every call.
//!
//! Two provenances of transport, mirroring the two auth provenances:
//!   - InProcess  — a direct call into shipd's gate. No socket, no PID. This is
//!                  the base Ship agent, and the thing that lets the shell host
//!                  an embedded agent.
//!   - Socket     — out-of-process, peer-cred attested (external harness / the
//!                  future Ship Code sidecar). Sketched here, not wired.
//!
//! IMPORTANT: the real ops surface (`services::command::call`) is mid-refactor
//! (ops SSOT in progress). The harness therefore depends ONLY on the `Gateway`
//! seam below — shipd implements it once the refactor lands. Nothing here
//! reaches into the RPC surface.

use async_trait::async_trait;
use serde_json::Value;

use crate::auth::AuthContext;

// `Denied`/`Unavailable` are produced by the real gate + socket impls; the stub
// path only hits `Op`. Kept for shape fidelity.
#[allow(dead_code)]
#[derive(Debug)]
pub enum TransportError {
    /// The gate refused the op (capability / blast radius).
    Denied(String),
    /// The op surface isn't reachable (not wired, socket down, refactor stub).
    Unavailable(String),
    /// The op ran and returned an error.
    Op(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::Denied(m) => write!(f, "DENIED: {m}"),
            TransportError::Unavailable(m) => write!(f, "UNAVAILABLE: {m}"),
            TransportError::Op(m) => write!(f, "OP ERROR: {m}"),
        }
    }
}
impl std::error::Error for TransportError {}

/// The channel the tool loop calls. Async because the real impls (in-process
/// `command::call` is `async`, and the socket impl) are async.
#[async_trait]
pub trait OpsTransport: Send + Sync {
    async fn call(
        &self,
        op: &str,
        args: Value,
        auth: &AuthContext,
    ) -> Result<Value, TransportError>;

    fn kind(&self) -> &'static str;
}

/// The seam shipd fills AFTER the ops SSOT refactor. The real impl will be a
/// thin wrapper:
///
/// ```ignore
/// async fn dispatch(&self, op, args, auth) -> Result<Value, String> {
///     let ctx = CommandCtx { runtime, auth: &to_shipd_authcontext(auth) };
///     services::command::call(op, &args, &ctx).await   // the ONE gate
/// }
/// ```
///
/// The harness knows nothing of `command::call` — only this trait.
#[async_trait]
pub trait Gateway: Send + Sync {
    async fn dispatch(&self, op: &str, args: Value, auth: &AuthContext) -> Result<Value, String>;
}

/// In-process transport: identity constructed, no PID. Delegates to the host's
/// `Gateway`. This is the base Ship agent's path.
pub struct InProcessTransport {
    pub gateway: std::sync::Arc<dyn Gateway>,
}

#[async_trait]
impl OpsTransport for InProcessTransport {
    async fn call(
        &self,
        op: &str,
        args: Value,
        auth: &AuthContext,
    ) -> Result<Value, TransportError> {
        eprintln!(
            "  transport[in-process] op={op} as agent={:?} operator={}",
            auth.agent_id, auth.operator_cred
        );
        self.gateway
            .dispatch(op, args, auth)
            .await
            .map_err(TransportError::Op)
    }

    fn kind(&self) -> &'static str {
        "in-process (constructed identity, no PID)"
    }
}

/// Out-of-process transport sketch: a socketed agent (external harness / Ship
/// Code sidecar) whose identity is KERNEL-ATTESTED (SO_PEERCRED), not
/// constructed. Not wired in the spike — present to show the symmetry.
#[allow(dead_code)]
pub struct SocketTransport;

#[async_trait]
impl OpsTransport for SocketTransport {
    async fn call(
        &self,
        _op: &str,
        _args: Value,
        _auth: &AuthContext,
    ) -> Result<Value, TransportError> {
        Err(TransportError::Unavailable(
            "socket transport not wired — out-of-process agents attest via SO_PEERCRED".into(),
        ))
    }

    fn kind(&self) -> &'static str {
        "socket (peer-cred attested) — sketch only"
    }
}

/// Stand-in Gateway so the loop runs end-to-end with the real ops surface
/// untouched. Echoes args back, namespaced by op. Swap for the real
/// `command::call` wrapper post-refactor.
pub struct StubGateway;

#[async_trait]
impl Gateway for StubGateway {
    async fn dispatch(&self, op: &str, args: Value, _auth: &AuthContext) -> Result<Value, String> {
        Ok(serde_json::json!({
            "op": op,
            "echo": args,
            "note": "stub gateway — real impl routes through services::command::call (the gate)"
        }))
    }
}
