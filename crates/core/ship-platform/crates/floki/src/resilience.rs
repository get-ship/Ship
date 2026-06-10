//! Resilience — retry the transient, never the terminal (S15).
//!
//! A transient execution failure (a flaky dispatch) should not end a run. But a
//! real `Denied` (the gate said no) or `Unverified` (the verifier refused the
//! result) MUST NOT be retried: a denial is data, an unverified result is a
//! refusal, and re-running either would mask a real signal — or, worse, keep
//! re-asking until a flake lets a refused result through. So only `Execute` is
//! retryable; the rest are terminal by construction.
//!
//! `run_resilient` wraps `spine::run` with that policy and emits the canonical
//! `session.failed` event on exhausted retries via the [`SessionEventSink`], so
//! the resilience story feeds the real kernel event stream. Per-attempt retry
//! diagnostics are logged via `tracing::warn!` — they are internal loop state,
//! not session lifecycle.

use std::future::Future;

use agent_def::{SessionEventSink, SessionFailed};

use crate::spine::{self, SpineError};

/// How many times to attempt a transient operation before giving up.
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    pub max_attempts: u32,
}

impl RetryPolicy {
    /// At least one attempt is always made (a 0 is clamped to 1).
    pub fn new(max_attempts: u32) -> Self {
        Self { max_attempts: max_attempts.max(1) }
    }
}

/// Whether a spine error is worth retrying. ONLY a transient `Execute` failure
/// is. `Denied` and `Unverified` are terminal — see the module doc.
fn is_transient(e: &SpineError) -> bool {
    matches!(e, SpineError::Execute(_))
}

/// Run `make_execute` → verify through the spine, retrying ONLY transient
/// `Execute` failures up to `policy.max_attempts`. Each retry is logged via
/// `tracing::warn!`; an exhausted run emits a `session.failed` canonical event
/// into `session_sink`. `Denied`/`Unverified` return immediately and are never
/// retried.
///
/// `make_execute` is a thunk so each attempt gets a FRESH future (a future is
/// single-use); `verify` is the same post-check the spine applies every time.
pub async fn run_resilient<T, F, Fut>(
    session_id: &str,
    session_sink: &SessionEventSink,
    label: &str,
    policy: RetryPolicy,
    mut make_execute: F,
    verify: impl Fn(&T) -> Result<String, String>,
) -> Result<String, SpineError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, SpineError>>,
{
    let mut last_err: Option<SpineError> = None;
    for attempt in 1..=policy.max_attempts {
        match spine::run(label, make_execute(), &verify).await {
            Ok(accepted) => return Ok(accepted),
            // Transient: retry if attempts remain, logging the retry via tracing.
            Err(e) if is_transient(&e) => {
                last_err = Some(e);
                if attempt < policy.max_attempts {
                    tracing::warn!(
                        label,
                        attempt,
                        max = policy.max_attempts,
                        "transient failure — retrying"
                    );
                }
            }
            // Terminal: a denial is data, an unverified result is a refusal.
            // Surface immediately — retrying is wrong, no event emitted.
            Err(terminal) => return Err(terminal),
        }
    }
    let err = last_err.unwrap_or_else(|| SpineError::Execute("no attempts made".into()));
    // Exhausted transient retries — emit the canonical session.failed event.
    session_sink
        .emit_session_failed(
            session_id,
            &SessionFailed {
                session_id: session_id.to_string(),
                reason: format!("{label}: exhausted {} attempt(s): {err}", policy.max_attempts),
            },
        )
        .ok();
    Err(err)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    use agent_def::SessionEventSink;
    use ship_kernel::testing::MockSink;

    use super::*;

    /// Build a fresh (MockSink, SessionEventSink) pair for a test.
    fn fresh_sink() -> (Arc<MockSink>, SessionEventSink) {
        let mock = Arc::new(MockSink::new());
        let session_sink = SessionEventSink::new(mock.clone());
        (mock, session_sink)
    }

    // A transient Execute failure that clears on a later attempt → Ok. Proof
    // of retry is the call count (≥2); no session.failed is emitted for a
    // run that ultimately succeeds.
    #[tokio::test]
    async fn transient_failure_is_retried_then_succeeds() {
        let (mock, session_sink) = fresh_sink();
        let calls = AtomicU32::new(0);
        let r = run_resilient(
            "test-session",
            &session_sink,
            "op",
            RetryPolicy::new(3),
            || {
                let n = calls.fetch_add(1, Ordering::SeqCst);
                async move {
                    if n == 0 {
                        Err(SpineError::Execute("flaky".into()))
                    } else {
                        Ok::<_, SpineError>("payload".to_string())
                    }
                }
            },
            |_v: &String| Ok("accepted".to_string()),
        )
        .await;
        assert!(r.is_ok(), "should succeed on retry; got {r:?}");
        // Proof of retry: first attempt failed, second succeeded → two calls.
        assert_eq!(calls.load(Ordering::SeqCst), 2, "must have retried once");
        // No session.failed on a successful run.
        assert!(
            !mock.captured().iter().any(|e| e.event_type == "session.failed"),
            "no session.failed must be emitted for a successful retried run",
        );
    }

    // A Denied is terminal: returned immediately, NOT retried, no session.failed.
    #[tokio::test]
    async fn denied_is_not_retried() {
        let (mock, session_sink) = fresh_sink();
        let calls = AtomicU32::new(0);
        let r = run_resilient(
            "test-session",
            &session_sink,
            "op",
            RetryPolicy::new(3),
            || {
                calls.fetch_add(1, Ordering::SeqCst);
                async { Err::<String, _>(SpineError::Denied("no grant".into())) }
            },
            |_v: &String| Ok("x".to_string()),
        )
        .await;
        assert!(matches!(r, Err(SpineError::Denied(_))));
        assert_eq!(calls.load(Ordering::SeqCst), 1, "a denial must not be retried");
        // Terminal denial does NOT emit session.failed (not the same as session exhaustion).
        assert!(
            !mock.captured().iter().any(|e| e.event_type == "session.failed"),
            "a terminal denial must not record session.failed",
        );
    }

    // An Unverified refusal is terminal too — the verifier said no.
    #[tokio::test]
    async fn unverified_is_not_retried() {
        let (mock, session_sink) = fresh_sink();
        let calls = AtomicU32::new(0);
        let r = run_resilient(
            "test-session",
            &session_sink,
            "op",
            RetryPolicy::new(3),
            || {
                calls.fetch_add(1, Ordering::SeqCst);
                async { Ok::<_, SpineError>("payload".to_string()) }
            },
            |_v: &String| Err("bad shape".to_string()),
        )
        .await;
        assert!(matches!(r, Err(SpineError::Unverified(_))));
        assert_eq!(calls.load(Ordering::SeqCst), 1, "an unverified result must not be retried");
        assert!(
            !mock.captured().iter().any(|e| e.event_type == "session.failed"),
            "an unverified terminal must not record session.failed",
        );
    }

    // A persistently-transient Execute exhausts retries, returns Err(Execute),
    // AND emits session.failed via the canonical builder. Proof of retry: both
    // attempts of RetryPolicy::new(2) ran before the exhaustion.
    #[tokio::test]
    async fn persistent_transient_exhausts_and_emits_session_failed() {
        let (mock, session_sink) = fresh_sink();
        let calls = AtomicU32::new(0);
        let r = run_resilient(
            "test-session",
            &session_sink,
            "op",
            RetryPolicy::new(2),
            || {
                calls.fetch_add(1, Ordering::SeqCst);
                async { Err::<String, _>(SpineError::Execute("always flaky".into())) }
            },
            |_v: &String| Ok("x".to_string()),
        )
        .await;
        assert!(matches!(r, Err(SpineError::Execute(_))));
        // Proof of retry: both attempts of RetryPolicy::new(2) executed.
        assert_eq!(calls.load(Ordering::SeqCst), 2, "both attempts must run before exhausting");
        // Canonical session.failed was emitted via the kernel-emit port.
        let captured = mock.captured();
        assert!(
            captured.iter().any(|e| e.event_type == "session.failed"),
            "an exhausted run must emit session.failed; got: {captured:?}",
        );
        // The reason references the label and the attempt count.
        let failed = captured.iter().find(|e| e.event_type == "session.failed").unwrap();
        let payload: serde_json::Value = serde_json::from_str(&failed.payload_json).unwrap();
        let reason = payload["reason"].as_str().unwrap_or("");
        assert!(reason.contains("op"), "reason must reference the label: {reason}");
        assert!(reason.contains("2"), "reason must reference the attempt count: {reason}");
    }
}
