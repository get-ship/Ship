//! Host-side [`ship_code::LifecycleSink`] that owns kernel emission.
//!
//! Harness code never sees `KernelRouter`. This adapter is the composition
//! root's bridge: outcomes from process/tmux executors become
//! `process.*` / `session.*` / stream synth events on the host event bus.

use std::sync::Arc;

use agent_schema::canonical_session::{
    SessionCompleted, SessionFailed, TurnEnded, event_types as canonical_session_event_types,
};
use agent_schema::op::OpToolUseStarted;
use async_trait::async_trait;
use ship_code::LifecycleSink;
use ship_substrate::db::block_on_anyhow;
use ship_substrate::events::emit::{emit_process_signal_sent, persist_and_route};
use ship_substrate::events::process::{event_types as process_event_types, ProcessKilledPayload};
use ship_substrate::events::{EmitContext, EventEnvelope, KernelRouter};
use agent_schema::canonical_session::event_types as stream_event_types;
use ship_types::event_types as substrate_event_types;
use tokio::sync::Mutex;

type Kernel = Arc<Mutex<KernelRouter>>;

/// Maps harness lifecycle observations onto the Ship kernel.
pub struct KernelLifecycleSink {
    kernel: Kernel,
}

impl KernelLifecycleSink {
    pub fn new(kernel: Kernel) -> Self {
        Self { kernel }
    }
}

fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[async_trait]
impl LifecycleSink for KernelLifecycleSink {
    async fn process_signal_sent(
        &self,
        slug: &str,
        pid: Option<u32>,
        signal: &str,
        reason: &str,
        actor: &str,
    ) {
        emit_process_signal_sent(&self.kernel, slug, pid, signal, reason, actor).await;
    }

    async fn process_killed(
        &self,
        slug: &str,
        pid: Option<u32>,
        signal: &str,
        reason: &str,
        actor: &str,
    ) {
        let payload = ProcessKilledPayload {
            slug: slug.to_string(),
            pid,
            signal: signal.to_string(),
            reason: reason.to_string(),
            actor: actor.to_string(),
        };
        match EventEnvelope::new(process_event_types::PROCESS_KILLED, slug, &payload) {
            Ok(env) => {
                let env = env.with_actor_id("service.process-executor");
                persist_and_route(&self.kernel, env, slug).await;
            }
            Err(e) => {
                tracing::warn!(slug, "kernel sink: failed to build process.killed: {e}");
            }
        }
    }

    async fn session_completed(&self, session_id: &str, slug: &str) {
        let payload = SessionCompleted {
            session_id: session_id.to_string(),
        };
        if let Ok(env) = EventEnvelope::new(
            canonical_session_event_types::SESSION_COMPLETED,
            session_id,
            &payload,
        ) {
            persist_and_route(&self.kernel, env, slug).await;
        }
    }

    async fn session_failed(&self, session_id: &str, slug: &str, reason: String) {
        let payload = SessionFailed {
            session_id: session_id.to_string(),
            reason,
        };
        if let Ok(env) = EventEnvelope::new(
            canonical_session_event_types::SESSION_FAILED,
            session_id,
            &payload,
        ) {
            persist_and_route(&self.kernel, env, slug).await;
        }
    }

    async fn supervisor_shutting_down(&self, agent_count: usize) {
        let envelope = match EventEnvelope::new(
            substrate_event_types::SUPERVISOR_SHUTTING_DOWN,
            "supervisor",
            &serde_json::json!({ "agent_count": agent_count }),
        ) {
            Ok(e) => e.with_actor_id("supervisor"),
            Err(e) => {
                tracing::error!("kernel sink: failed to build supervisor.shutting_down: {e}");
                return;
            }
        };
        let ctx = EmitContext::daemon_internal(None);
        if let Err(e) = self.kernel.lock().await.emit(envelope, &ctx).await {
            tracing::warn!("kernel sink: failed to emit supervisor.shutting_down: {e}");
        }
    }

    fn op_tool_use_started(&self, actor_id: &str, tool: &str, input_preview: Option<&str>) {
        let payload = OpToolUseStarted {
            session_id: actor_id.to_string(),
            harness_session_id: None,
            tool: tool.to_string(),
            input: input_preview.map(|s| serde_json::Value::String(s.to_string())),
            blast: None,
            started_at: now_iso8601(),
        };
        let Ok(envelope) = EventEnvelope::new(
            stream_event_types::OP_TOOL_USE_STARTED,
            actor_id,
            &payload,
        )
        .map(|e| e.with_actor_id(actor_id)) else {
            return;
        };
        let kernel = Arc::clone(&self.kernel);
        let _ = block_on_anyhow(async move {
            let ctx = EmitContext::daemon_internal(None);
            kernel.lock().await.emit(envelope, &ctx).await
        });
    }

    fn turn_ended(
        &self,
        actor_id: &str,
        model: Option<&str>,
        input_tokens: u64,
        output_tokens: u64,
        reasoning_tokens: u64,
        cache_creation_tokens: u64,
        cache_read_tokens: u64,
        stop_reason: Option<&str>,
        harness_reported_cost_usd: Option<f64>,
    ) {
        let payload = TurnEnded {
            session_id: actor_id.to_string(),
            idx: 0,
            stop_reason: stop_reason.map(str::to_string),
            harness_session_id: None,
            model: model.map(str::to_string),
            input_tokens: Some(input_tokens),
            output_tokens: Some(output_tokens),
            reasoning_tokens: if reasoning_tokens > 0 {
                Some(reasoning_tokens)
            } else {
                None
            },
            cache_creation_tokens: if cache_creation_tokens > 0 {
                Some(cache_creation_tokens)
            } else {
                None
            },
            cache_read_tokens: if cache_read_tokens > 0 {
                Some(cache_read_tokens)
            } else {
                None
            },
            harness_reported_cost_usd,
            observed_at: Some(now_iso8601()),
        };
        let Ok(envelope) =
            EventEnvelope::new(stream_event_types::TURN_ENDED, actor_id, &payload)
                .map(|e| e.with_actor_id(actor_id))
        else {
            return;
        };
        let kernel = Arc::clone(&self.kernel);
        let _ = block_on_anyhow(async move {
            let ctx = EmitContext::daemon_internal(None);
            kernel.lock().await.emit(envelope, &ctx).await
        });
    }
}
