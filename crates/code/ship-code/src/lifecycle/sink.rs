//! Host observation port for harness lifecycle.
//!
//! Harness code (process/tmux executors, teardown, exit routing) reports
//! outcomes through this trait. It must **never** take or name
//! `KernelRouter` / `EventEnvelope` — the Ship host implements the sink
//! and owns emission. That is the Bridge/Ship seam: DTO-shaped callbacks,
//! not a shared event type graph.

use async_trait::async_trait;

/// Observation port filled by the host (shipd / helmd / ship-agents).
///
/// Default methods are no-ops so tests and kernel-free callers can use
/// [`NoopLifecycleSink`] or a partial mock.
#[async_trait]
pub trait LifecycleSink: Send + Sync {
    /// Process group / child was signalled (pre-stop).
    async fn process_signal_sent(
        &self,
        _slug: &str,
        _pid: Option<u32>,
        _signal: &str,
        _reason: &str,
        _actor: &str,
    ) {
    }

    /// Process was killed (SIGTERM path, force kill, etc.).
    async fn process_killed(
        &self,
        _slug: &str,
        _pid: Option<u32>,
        _signal: &str,
        _reason: &str,
        _actor: &str,
    ) {
    }

    /// Bare session completed successfully (exit code 0).
    async fn session_completed(&self, _session_id: &str, _slug: &str) {}

    /// Bare session failed (non-zero exit or external destroy).
    async fn session_failed(&self, _session_id: &str, _slug: &str, _reason: String) {}

    /// Supervisor is about to stop all managed agents.
    async fn supervisor_shutting_down(&self, _agent_count: usize) {}

    /// Harness stream: tool use observed (synth path).
    fn op_tool_use_started(&self, _actor_id: &str, _tool: &str, _input_preview: Option<&str>) {}

    /// Harness stream: turn ended with token usage (synth path).
    #[allow(clippy::too_many_arguments)]
    fn turn_ended(
        &self,
        _actor_id: &str,
        _model: Option<&str>,
        _input_tokens: u64,
        _output_tokens: u64,
        _reasoning_tokens: u64,
        _cache_creation_tokens: u64,
        _cache_read_tokens: u64,
        _stop_reason: Option<&str>,
        _harness_reported_cost_usd: Option<f64>,
    ) {
    }
}

/// Sink that drops every observation. Use in unit tests and when no host
/// is wired.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopLifecycleSink;

#[async_trait]
impl LifecycleSink for NoopLifecycleSink {}
