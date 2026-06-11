//! `InvocationMode` — how a harness is invoked.
//!
//! Domain-owned at L1 (keystone-A): the type rode along in the retired L0
//! `events::agent_session` module long after the `agent.session.*` family
//! was severed (#26); every consumer is L1 dispatch/runner code, so it lives
//! with the dispatch intent now. The L0 module was deleted with the move.

use serde::{Deserialize, Serialize};

/// How the harness is invoked. Replaces the old `headless: bool` that
/// conflated UX-attach with billing concerns.
///
/// - `Pty`: harness runs in a PTY (claude TUI, codex TUI, ...). Works
///   for all 5 harnesses. Subscription-billed where the harness has
///   a subscription/API split.
/// - `Print`: one-shot non-interactive mode (claude `-p`, codex `exec`).
///   No keepalive. API-billed for claude post-2026-06-19.
/// - `Stream`: server-mode JSON-RPC or REST session (codex
///   `app-server`, opencode REST). Long-lived; supports keepalive.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "lowercase")]
pub enum InvocationMode {
    Pty,
    Print,
    Stream,
}

impl InvocationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvocationMode::Pty => "pty",
            InvocationMode::Print => "print",
            InvocationMode::Stream => "stream",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "pty" => InvocationMode::Pty,
            "print" => InvocationMode::Print,
            "stream" => InvocationMode::Stream,
            _ => InvocationMode::Pty,
        }
    }
}
