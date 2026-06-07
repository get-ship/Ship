//! Resilience — retry the transient, never the terminal (S15).
//!
//! A transient execution failure (a flaky dispatch) should not end a run. But a
//! real `Denied` (the gate said no) or `Unverified` (the verifier refused the
//! result) MUST NOT be retried: a denial is data, an unverified result is a
//! refusal, and re-running either would mask a real signal — or, worse, keep
//! re-asking until a flake lets a refused result through. So only `Execute` is
//! retryable; the rest are terminal by construction.
//!
//! `run_resilient` wraps `spine::run` with that policy and narrates each retry
//! (and an exhausted run) into the event log, so the resilience story is itself
//! event-sourced — exactly what S16's graduation will point at the daemon's
//! persistent `ship_subscriber` log.

use std::future::Future;

use crate::events::{AgentEvent, EventSink};
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
/// `Execute` failures up to `policy.max_attempts`. Each retry emits a `Status`
/// event into `sink`; an exhausted run emits `Error`. `Denied`/`Unverified`
/// return immediately and are never retried.
///
/// `make_execute` is a thunk so each attempt gets a FRESH future (a future is
/// single-use); `verify` is the same post-check the spine applies every time.
pub async fn run_resilient<T, F, Fut>(
    sink: &dyn EventSink,
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
            // Transient: retry if attempts remain, recording the retry.
            Err(e) if is_transient(&e) => {
                last_err = Some(e);
                if attempt < policy.max_attempts {
                    sink.emit(&AgentEvent::Status {
                        message: format!(
                            "{label}: transient failure on attempt {attempt}/{}, retrying",
                            policy.max_attempts
                        ),
                    });
                }
            }
            // Terminal: a denial is data, an unverified result is a refusal.
            // Surface immediately — retrying is wrong.
            Err(terminal) => return Err(terminal),
        }
    }
    let err = last_err.unwrap_or_else(|| SpineError::Execute("no attempts made".into()));
    sink.emit(&AgentEvent::Error {
        message: format!("{label}: exhausted {} attempt(s): {err}", policy.max_attempts),
    });
    Err(err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::CollectingSink;
    use std::sync::atomic::{AtomicU32, Ordering};

    // A transient Execute failure that clears on a later attempt → Ok, and the
    // log shows a retry Status before the success.
    #[tokio::test]
    async fn transient_failure_is_retried_then_succeeds() {
        let sink = CollectingSink::new();
        let calls = AtomicU32::new(0);
        let r = run_resilient(
            &sink,
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
        let evs = sink.collected();
        assert!(
            evs.iter().any(|e| matches!(e, AgentEvent::Status { message } if message.contains("retry"))),
            "a retry must be recorded in the log",
        );
    }

    // A Denied is terminal: returned immediately, NOT retried, no retry Status.
    #[tokio::test]
    async fn denied_is_not_retried() {
        let sink = CollectingSink::new();
        let calls = AtomicU32::new(0);
        let r = run_resilient(
            &sink,
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
        assert!(
            !sink.collected().iter().any(|e| matches!(e, AgentEvent::Status { .. })),
            "a terminal denial records no retry",
        );
    }

    // An Unverified refusal is terminal too — the verifier said no.
    #[tokio::test]
    async fn unverified_is_not_retried() {
        let sink = CollectingSink::new();
        let calls = AtomicU32::new(0);
        let r = run_resilient(
            &sink,
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
    }

    // A persistently-transient Execute exhausts retries, returns Err(Execute),
    // and the log ends with an Error event.
    #[tokio::test]
    async fn persistent_transient_exhausts_and_errors() {
        let sink = CollectingSink::new();
        let r = run_resilient(
            &sink,
            "op",
            RetryPolicy::new(2),
            || async { Err::<String, _>(SpineError::Execute("always flaky".into())) },
            |_v: &String| Ok("x".to_string()),
        )
        .await;
        assert!(matches!(r, Err(SpineError::Execute(_))));
        let evs = sink.collected();
        assert!(
            matches!(evs.last(), Some(AgentEvent::Error { .. })),
            "an exhausted run ends with an Error event; got {evs:?}",
        );
    }
}
