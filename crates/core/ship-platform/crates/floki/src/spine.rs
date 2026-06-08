//! The verification spine — "nothing gets blindly accepted."
//!
//! For ops, the PERMIT gate lives in the gateway (the daemon side). The spine's
//! job here is the post-exec half the other harnesses lack: given a result,
//! VERIFY it before it enters context. An unverifiable result is refused, not
//! folded in as success.
//!
//! ```text
//! (permit = gateway gate)  ->  Execute (await op)  ->  Verify (post-check)
//! ```

use std::future::Future;

#[derive(Debug, thiserror::Error)]
pub enum SpineError {
    #[error("DENIED: {0}")]
    Denied(String),
    #[error("EXECUTE FAILED: {0}")]
    Execute(String),
    #[error("UNVERIFIED (refused, not accepted): {0}")]
    Unverified(String),
}

/// Run an async side effect through execute -> verify. `execute` already passed
/// the gateway gate (or returns a Denied error from it). `verify` is the
/// adversarial post-check: `Ok(accepted)` or `Err(reason)` => refuse.
pub async fn run<T>(
    label: &str,
    execute: impl Future<Output = Result<T, SpineError>>,
    verify: impl FnOnce(&T) -> Result<String, String>,
) -> Result<String, SpineError> {
    let out = match execute.await {
        Ok(o) => o,
        Err(e) => {
            eprintln!("  spine[{label}] {e}");
            return Err(e);
        }
    };
    match verify(&out) {
        Ok(accepted) => {
            eprintln!("  spine[{label}] verify ok");
            Ok(accepted)
        }
        Err(e) => {
            eprintln!("  spine[{label}] verify REFUSED: {e}");
            Err(SpineError::Unverified(e))
        }
    }
}
