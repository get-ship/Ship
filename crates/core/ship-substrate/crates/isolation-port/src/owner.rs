//! `SandboxOwner` — binds a sandbox lifetime to an actor handle.
//!
//! When an `ProcessHandle` is dropped (last clone gone, no more attaches,
//! orchestrator released it), the `Arc<SandboxOwner>` it carries drops
//! too, and `Drop` spawns the isolator's `shutdown()` on the active
//! tokio runtime. This is what makes option (a) from the
//! lifecycle-binding discussion work without coupling substrate to
//! the L1 strategy types.
//!
//! The owner takes the prepared `SandboxHandle` once. After drop, the
//! sandbox is gone — any later `exec()` against it would fail at the
//! backend level (container doesn't exist).
//!
//! Behaviour outside a tokio runtime: warned + leaked. Substrate is
//! always run on a tokio runtime in practice (shipd, tests). The
//! warning catches the rare case where a SandboxOwner is dropped from
//! a thread without a runtime handle — that's a bug at the caller.

use std::sync::Arc;
use std::sync::Mutex;

use crate::handle::SandboxHandle;
use crate::trait_def::Isolator;

/// Holds the prepared sandbox + the isolator that prepared it. Drop
/// fires shutdown on the active tokio runtime. Cheap to clone via Arc
/// at the caller — each clone shares the same shutdown.
pub struct SandboxOwner {
    isolator: Arc<dyn Isolator>,
    handle: Mutex<Option<SandboxHandle>>,
}

impl SandboxOwner {
    pub fn new(isolator: Arc<dyn Isolator>, handle: SandboxHandle) -> Self {
        Self {
            isolator,
            handle: Mutex::new(Some(handle)),
        }
    }

    /// The backend's identifier (e.g. "docker", "youki"). Convenience
    /// for callers that want to render which sandbox an actor lives in
    /// without downcasting the trait object.
    pub fn backend(&self) -> &'static str {
        self.isolator.name()
    }

    /// Take the sandbox handle without shutting it down. Used by
    /// downstream callers that want to issue another exec against the
    /// same sandbox (e.g. attach a follow-up command). After this
    /// returns, the owner becomes a no-op on drop.
    pub fn take_handle(&self) -> Option<SandboxHandle> {
        self.handle.lock().ok().and_then(|mut g| g.take())
    }

    /// A clone of the handle for inspection. The handle stays owned by
    /// the SandboxOwner — drop will still fire shutdown.
    pub fn handle(&self) -> Option<SandboxHandle> {
        self.handle.lock().ok().and_then(|g| g.clone())
    }
}

impl Drop for SandboxOwner {
    fn drop(&mut self) {
        let Some(handle) = self.handle.get_mut().ok().and_then(|h| h.take()) else {
            return;
        };
        let isolator = self.isolator.clone();
        match tokio::runtime::Handle::try_current() {
            Ok(rt) => {
                rt.spawn(async move {
                    if let Err(e) = isolator.shutdown(handle).await {
                        tracing::warn!(error = %e, "sandbox shutdown failed during drop");
                    }
                });
            }
            Err(_) => {
                tracing::warn!(
                    backend = isolator.name(),
                    "SandboxOwner dropped outside a tokio runtime; sandbox may leak"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handle::SandboxId;
    use crate::policy::SandboxPolicy;
    use crate::trait_def::IsolatorError;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingIsolator {
        shutdown_count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Isolator for CountingIsolator {
        fn name(&self) -> &'static str {
            "counter"
        }
        async fn available(&self) -> Result<(), IsolatorError> {
            Ok(())
        }
        async fn prepare(
            &self,
            _policy: &SandboxPolicy,
        ) -> Result<SandboxHandle, IsolatorError> {
            unreachable!()
        }
        async fn shutdown(&self, _sandbox: SandboxHandle) -> Result<(), IsolatorError> {
            self.shutdown_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn fake_handle() -> SandboxHandle {
        SandboxHandle {
            id: SandboxId::new(),
            backend: "counter",
            root: std::env::temp_dir(),
            state: Arc::new(()),
        }
    }

    #[tokio::test]
    async fn drop_fires_shutdown_exactly_once() {
        let counter = Arc::new(AtomicUsize::new(0));
        let isolator: Arc<dyn Isolator> = Arc::new(CountingIsolator {
            shutdown_count: counter.clone(),
        });
        {
            let owner = Arc::new(SandboxOwner::new(isolator, fake_handle()));
            let _clone = owner.clone();
            // Both go out of scope here; only the last drop fires shutdown.
        }
        // The shutdown is spawned on the runtime; yield once so it runs.
        tokio::task::yield_now().await;
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn take_handle_disables_drop_shutdown() {
        let counter = Arc::new(AtomicUsize::new(0));
        let isolator: Arc<dyn Isolator> = Arc::new(CountingIsolator {
            shutdown_count: counter.clone(),
        });
        {
            let owner = SandboxOwner::new(isolator, fake_handle());
            let taken = owner.take_handle();
            assert!(taken.is_some());
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }
}
