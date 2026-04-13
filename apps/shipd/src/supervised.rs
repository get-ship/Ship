//! Supervised background task spawner.
//!
//! Wraps `tokio::spawn` with panic catching, structured logging, and
//! automatic restart for critical daemon tasks. Each supervised task gets
//! a `TaskHealth` handle that the `/api/health` endpoint can query.

use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

/// Health status for a supervised background task.
#[derive(Debug)]
pub struct TaskHealth {
    pub name: &'static str,
    alive: AtomicBool,
    restart_count: AtomicU64,
}

impl TaskHealth {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            alive: AtomicBool::new(true),
            restart_count: AtomicU64::new(0),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }

    pub fn restart_count(&self) -> u64 {
        self.restart_count.load(Ordering::Relaxed)
    }
}

/// Shared handle to a supervised task's health.
pub type SharedTaskHealth = Arc<TaskHealth>;

/// Registry of all supervised tasks in the daemon.
#[derive(Default, Clone)]
pub struct HealthRegistry {
    tasks: Arc<std::sync::Mutex<Vec<SharedTaskHealth>>>,
}

impl HealthRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn register(&self, health: SharedTaskHealth) {
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.push(health);
        }
    }

    /// Snapshot of all task health statuses.
    pub fn snapshot(&self) -> Vec<TaskSnapshot> {
        let tasks = match self.tasks.lock() {
            Ok(t) => t,
            Err(poisoned) => poisoned.into_inner(),
        };
        tasks
            .iter()
            .map(|h| TaskSnapshot {
                name: h.name,
                alive: h.is_alive(),
                restart_count: h.restart_count(),
            })
            .collect()
    }
}

#[derive(Debug, serde::Serialize)]
pub struct TaskSnapshot {
    pub name: &'static str,
    pub alive: bool,
    pub restart_count: u64,
}

/// Maximum restarts before giving up.
const MAX_RESTARTS: u64 = 50;

/// Initial backoff delay after a panic.
const INITIAL_BACKOFF: Duration = Duration::from_secs(1);

/// Maximum backoff delay.
const MAX_BACKOFF: Duration = Duration::from_secs(30);

/// Spawn a monitored one-shot background task.
///
/// Like `spawn_supervised` but does NOT restart on panic. Use for tasks
/// that own a consumed resource (e.g., a mailbox receiver) and can't be
/// re-created. Panics are caught, logged, and the task is marked dead
/// in the health registry.
pub fn spawn_monitored<Fut>(
    registry: &HealthRegistry,
    name: &'static str,
    fut: Fut,
) -> JoinHandle<()>
where
    Fut: Future<Output = ()> + Send + 'static,
{
    let health = Arc::new(TaskHealth::new(name));
    registry.register(health.clone());

    tokio::spawn(async move {
        health.alive.store(true, Ordering::Relaxed);
        let result = tokio::spawn(fut).await;
        health.alive.store(false, Ordering::Relaxed);

        match result {
            Ok(()) => {
                tracing::info!(task = name, "monitored task exited normally");
            }
            Err(join_error) => {
                if join_error.is_panic() {
                    let panic_info = join_error.into_panic();
                    let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        (*s).to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    tracing::error!(
                        task = name,
                        panic = %msg,
                        "monitored task panicked (non-restartable)"
                    );
                } else {
                    tracing::error!(
                        task = name,
                        error = %join_error,
                        "monitored task was cancelled"
                    );
                }
            }
        }
    })
}

/// Spawn a supervised background task that restarts on panic.
///
/// `name` identifies the task in logs and health checks.
/// `factory` is called each time the task (re)starts — it must produce
/// a new future since the previous one is consumed on panic.
///
/// Returns a `JoinHandle` for the supervisor loop itself (not the inner task).
pub fn spawn_supervised<F, Fut>(
    registry: &HealthRegistry,
    name: &'static str,
    factory: F,
) -> JoinHandle<()>
where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let health = Arc::new(TaskHealth::new(name));
    registry.register(health.clone());

    tokio::spawn(async move {
        let mut backoff = INITIAL_BACKOFF;

        loop {
            health.alive.store(true, Ordering::Relaxed);

            let fut = factory();
            let result = tokio::spawn(fut).await;

            match result {
                Ok(()) => {
                    // Task completed normally (e.g., mailbox closed).
                    tracing::info!(task = name, "supervised task exited normally");
                    health.alive.store(false, Ordering::Relaxed);
                    return;
                }
                Err(join_error) => {
                    health.alive.store(false, Ordering::Relaxed);
                    let count = health.restart_count.fetch_add(1, Ordering::Relaxed) + 1;

                    if join_error.is_panic() {
                        let panic_info = join_error.into_panic();
                        let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                            (*s).to_string()
                        } else if let Some(s) = panic_info.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "unknown panic".to_string()
                        };
                        tracing::error!(
                            task = name,
                            panic = %msg,
                            restart_count = count,
                            "supervised task panicked"
                        );
                    } else {
                        tracing::error!(
                            task = name,
                            error = %join_error,
                            restart_count = count,
                            "supervised task was cancelled"
                        );
                    }

                    if count >= MAX_RESTARTS {
                        tracing::error!(
                            task = name,
                            "supervised task exceeded max restarts ({MAX_RESTARTS}), giving up"
                        );
                        return;
                    }

                    tracing::info!(
                        task = name,
                        backoff_secs = backoff.as_secs(),
                        "restarting supervised task"
                    );
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[tokio::test]
    async fn normal_exit_does_not_restart() {
        let registry = HealthRegistry::new();
        let handle = spawn_supervised(&registry, "test-normal", || async {
            // Exits immediately.
        });
        handle.await.unwrap();

        let snap = registry.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0].restart_count, 0);
        assert!(!snap[0].alive);
    }

    #[tokio::test]
    async fn panic_triggers_restart() {
        let registry = HealthRegistry::new();
        let call_count = Arc::new(AtomicUsize::new(0));
        let cc = call_count.clone();

        let handle = spawn_supervised(&registry, "test-panic", move || {
            let cc = cc.clone();
            async move {
                let n = cc.fetch_add(1, Ordering::Relaxed);
                if n < 2 {
                    panic!("intentional test panic #{n}");
                }
                // Third call exits normally.
            }
        });
        handle.await.unwrap();

        assert_eq!(call_count.load(Ordering::Relaxed), 3);
        let snap = registry.snapshot();
        assert_eq!(snap[0].restart_count, 2);
    }

    #[tokio::test]
    async fn max_restarts_stops_supervisor() {
        let registry = HealthRegistry::new();
        let handle = spawn_supervised(&registry, "test-max", || async {
            panic!("always panic");
        });
        handle.await.unwrap();

        let snap = registry.snapshot();
        assert_eq!(snap[0].restart_count, MAX_RESTARTS);
        assert!(!snap[0].alive);
    }
}
