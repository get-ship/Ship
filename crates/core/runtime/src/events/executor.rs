//! Executor substrate events — tmux sessions, tmux windows, process groups.
//!
//! Distinct from `process.*` (which tracks the agent process itself): these
//! observe the container the executor spawned the agent into. The bug this
//! closes: an operator `tmux kill-window` used to look indistinguishable from
//! a clean agent exit. Now `executor.session_destroyed` / `executor.window_destroyed`
//! fire with a `DestructionCause`, and the liveness handler branches on it.
//!
//! Creation events (`session_created`, `window_created`) are defined here
//! but not yet emitted — deferred to executor-events-02.

use serde::{Deserialize, Serialize};
use specta::Type;

pub use ship_types::event_types::{
    EXECUTOR_SESSION_CREATED, EXECUTOR_SESSION_DESTROYED, EXECUTOR_WINDOW_CREATED,
    EXECUTOR_WINDOW_DESTROYED,
};

/// Why a session/window is gone. Drives the liveness handler's decision
/// between `job.completed` and `job.failed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum DestructionCause {
    /// Agent process exited cleanly (we observed it).
    CleanExit,
    /// Agent process crashed (non-zero exit code, signal, etc.).
    Crash,
    /// Session/window vanished without us seeing the agent exit.
    /// Usually operator action or executor-substrate failure.
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExecutorSessionCreatedPayload {
    pub executor_kind: String,
    pub session_name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExecutorSessionDestroyedPayload {
    pub executor_kind: String,
    pub session_name: String,
    pub slug: String,
    pub cause: DestructionCause,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExecutorWindowCreatedPayload {
    pub executor_kind: String,
    pub session_name: String,
    pub window_name: String,
    pub slug: String,
    pub step: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExecutorWindowDestroyedPayload {
    pub executor_kind: String,
    pub session_name: String,
    pub window_name: String,
    pub slug: String,
    pub step: Option<String>,
    pub cause: DestructionCause,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_destroyed_payload_round_trips() {
        let payload = ExecutorSessionDestroyedPayload {
            executor_kind: "tmux_window".to_string(),
            session_name: "ship-test".to_string(),
            slug: "test-slug".to_string(),
            cause: DestructionCause::External,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let decoded: ExecutorSessionDestroyedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.executor_kind, "tmux_window");
        assert_eq!(decoded.session_name, "ship-test");
        assert_eq!(decoded.slug, "test-slug");
        assert_eq!(decoded.cause, DestructionCause::External);
        assert!(json.contains("\"external\""), "cause must serialize snake_case");
    }

    #[test]
    fn window_destroyed_payload_round_trips() {
        let payload = ExecutorWindowDestroyedPayload {
            executor_kind: "tmux_window".to_string(),
            session_name: "ship-test".to_string(),
            window_name: "step-1".to_string(),
            slug: "test-slug".to_string(),
            step: Some("test-writer".to_string()),
            cause: DestructionCause::Crash,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let decoded: ExecutorWindowDestroyedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.window_name, "step-1");
        assert_eq!(decoded.step.as_deref(), Some("test-writer"));
        assert_eq!(decoded.cause, DestructionCause::Crash);
    }

    #[test]
    fn session_created_payload_round_trips() {
        let payload = ExecutorSessionCreatedPayload {
            executor_kind: "tmux".to_string(),
            session_name: "ship-slug".to_string(),
            slug: "slug".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let decoded: ExecutorSessionCreatedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.session_name, "ship-slug");
    }

    #[test]
    fn window_created_payload_round_trips() {
        let payload = ExecutorWindowCreatedPayload {
            executor_kind: "tmux_window".to_string(),
            session_name: "ship-s".to_string(),
            window_name: "test-writer".to_string(),
            slug: "s".to_string(),
            step: Some("test-writer".to_string()),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let decoded: ExecutorWindowCreatedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.step.as_deref(), Some("test-writer"));
    }

    #[test]
    fn destruction_cause_round_trips_all_variants() {
        for c in [
            DestructionCause::CleanExit,
            DestructionCause::Crash,
            DestructionCause::External,
        ] {
            let json = serde_json::to_string(&c).unwrap();
            let decoded: DestructionCause = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, c);
        }
    }
}
