//! Signal-backed `ship.git.overview` source.
//!
//! `GIT_OVERVIEW` is a process-wide `Signal<serde_json::Value>` that holds
//! the most recently resolved git.overview payload. A single background task
//! (started lazily on first SSE connection) subscribes to the `"config"`
//! kernel namespace and refreshes the signal whenever the underlying state
//! changes. SSE connections subscribe to the signal directly via `effect()`
//! instead of spawning their own kernel actors.
//!
//! This establishes the SDK source authoring pattern for phase 1:
//! maintain a `Signal<T>` singleton, wire kernel events → signal update,
//! let SSE connections subscribe to the signal.

use std::sync::{OnceLock, atomic::{AtomicBool, Ordering}};

use ship_substrate::events::ActorConfig;
use ship_substrate::gen_ulid;
use serde_json::Value;
use sui_signals::Signal;

use crate::rest_api::ApiState;
use crate::services::registry;

/// The canonical git.overview signal. `Null` until first resolver run.
static GIT_OVERVIEW: OnceLock<Signal<Value>> = OnceLock::new();

/// Tracks whether the background updater task has been spawned.
static UPDATER_STARTED: AtomicBool = AtomicBool::new(false);

/// Returns the process-wide git.overview signal.
pub fn git_overview_signal() -> &'static Signal<Value> {
    GIT_OVERVIEW.get_or_init(|| Signal::new(Value::Null))
}

/// Registry name handled by this signal-backed source.
pub const REGISTRY_NAME: &str = "git.overview";

/// Resolve git.overview and push the result into the signal.
/// Called once at SSE startup (for the initial value) and by the background
/// task on each kernel event.
pub fn refresh(state: &ApiState) {
    let args = serde_json::json!({});
    match registry::call(REGISTRY_NAME, &args, state) {
        Ok(value) => git_overview_signal().set_unchecked(value),
        Err(e) => tracing::warn!(error = %e, "git_signal: refresh failed"),
    }
}

/// Ensure the background kernel-event → signal-update task is running.
/// Idempotent: the task is spawned at most once per daemon process.
pub async fn ensure_updater(state: ApiState) {
    if UPDATER_STARTED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return; // already started
    }
    tokio::spawn(run_updater(state));
}

/// Background task: subscribe to `"config"` namespace events and refresh
/// the signal on each event (coalescing bursts like the existing namespace path).
async fn run_updater(state: ApiState) {
    let actor_id = format!("git-signal-updater-{}", gen_ulid());
    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["config".to_string()],
    };
    let mailbox = match state.kernel().lock().await.spawn_actor(&actor_id, config) {
        Ok((_store, mb)) => mb,
        Err(e) => {
            tracing::error!(error = %e, actor = %actor_id, "git_signal: spawn_actor failed");
            UPDATER_STARTED.store(false, Ordering::Release); // allow retry
            return;
        }
    };
    let _guard = crate::runtime_api::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };
    let mut mb = mailbox;

    loop {
        let Some(_ev) = mb.recv().await else { break };
        while mb.try_recv().is_ok() {} // coalesce burst
        refresh(&state);
    }

    tracing::warn!("git_signal: updater task exited");
    UPDATER_STARTED.store(false, Ordering::Release); // allow restart on next connection
}
