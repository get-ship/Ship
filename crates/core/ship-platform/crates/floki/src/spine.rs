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
            tracing::warn!(label = %label, error = %e, "spine execute failed");
            return Err(e);
        }
    };
    match verify(&out) {
        Ok(accepted) => {
            tracing::debug!(label = %label, "spine verify ok");
            Ok(accepted)
        }
        Err(e) => {
            tracing::warn!(label = %label, reason = %e, "spine verify REFUSED");
            Err(SpineError::Unverified(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    // execute ok + verify ok → the verified value is accepted.
    #[tokio::test]
    async fn verify_accepts_a_confirmed_result() {
        let out = run("ok", async { Ok::<i32, SpineError>(42) }, |v| {
            if *v == 42 {
                Ok("accepted".into())
            } else {
                Err("bad".into())
            }
        })
        .await
        .expect("a confirmed result is accepted");
        assert_eq!(out, "accepted");
    }

    // execute ok + verify err → refused as Unverified (NOT folded in as success).
    #[tokio::test]
    async fn verify_refuses_an_unverifiable_result() {
        let r = run("bad", async { Ok::<i32, SpineError>(1) }, |_v| {
            Err("nope".into())
        })
        .await;
        assert!(
            matches!(r, Err(SpineError::Unverified(_))),
            "an unverifiable result must be refused, not accepted",
        );
    }

    // execute err → short-circuit: verify MUST NOT run (nothing to verify), and
    // the original error propagates unchanged.
    #[tokio::test]
    async fn execute_failure_short_circuits_before_verify() {
        let verified = AtomicBool::new(false);
        let r = run::<i32>(
            "fail",
            async { Err(SpineError::Execute("boom".into())) },
            |_v| {
                verified.store(true, Ordering::SeqCst);
                Ok("should never run".into())
            },
        )
        .await;
        assert!(matches!(r, Err(SpineError::Execute(_))));
        assert!(
            !verified.load(Ordering::SeqCst),
            "verify must NOT run when execute fails — there is nothing to verify",
        );
    }

    // A gate denial passes through the spine unchanged (denial is data; the spine
    // doesn't re-classify it as Unverified).
    #[tokio::test]
    async fn denial_propagates_as_denied() {
        let r = run::<i32>(
            "denied",
            async { Err(SpineError::Denied("not granted".into())) },
            |_v| Ok("x".into()),
        )
        .await;
        assert!(matches!(r, Err(SpineError::Denied(_))));
    }
}
