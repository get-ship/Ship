//! L1 supervisor-stream synth-emit helpers — emit op.* and session.turn.ended
//! events from the supervisor's harness-stream parser WITHOUT going through
//! the L0 hook_events module.
//!
//! Architectural note (R3 / keystone-A): synthesising domain events in L0
//! ship-substrate was a layering smell. This module lifts the synthesis to
//! L1 (ship-code) where op.* payloads live. L0 remains a dumb event store.
//!
//! Emit path: build the EventEnvelope → route through the global
//! `KernelRouter::emit`, the sole place Retention is honored. `op.*` and
//! `session.turn.*` are registered `Retention::Ephemeral` — the kernel
//! delivers them to live subscribers without persisting.

use ship_substrate::db::block_on_anyhow;
use ship_substrate::events::EventEnvelope;
use ship_types::event_types::canonical_session as event_types;

fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Route an op.* or session.turn.* envelope through the global kernel.
fn emit<P: serde::Serialize>(
    actor_id: &str,
    event_type: &'static str,
    payload: &P,
) -> anyhow::Result<()> {
    let envelope = EventEnvelope::new(event_type, actor_id, payload)?.with_actor_id(actor_id);
    let kernel = ship_substrate::events::kernel_router()
        .expect("kernel must be initialized before synth_emit emits");
    let ctx = ship_types::EmitContext::daemon_internal(None);
    block_on_anyhow(async move { kernel.lock().await.emit(envelope, &ctx).await })
}

/// Emit `op.tool_use_started` from the supervisor harness-stream parser.
/// The `actor_id` (slug) is used as the canonical `session_id` since the
/// synth path operates before the canonical session.started event resolves
/// the harness's internal session id to a Ship canonical id.
pub fn emit_op_tool_use_started_synth(
    actor_id: &str,
    tool: &str,
    input_preview: Option<&str>,
) -> anyhow::Result<()> {
    let payload = agent_def::op::OpToolUseStarted {
        session_id: actor_id.to_string(),
        harness_session_id: None,
        tool: tool.to_string(),
        input: input_preview.map(|s| serde_json::Value::String(s.to_string())),
        blast: None,
        started_at: now_iso8601(),
    };
    emit(actor_id, event_types::OP_TOOL_USE_STARTED, &payload)
}

/// Emit `session.turn.ended` with token-usage fields from the supervisor
/// harness-stream parser. This is the canonical analytics_turns source for
/// both the opencode synth path and the claude stream-json path.
#[allow(clippy::too_many_arguments)]
pub fn emit_turn_ended_synth(
    actor_id: &str,
    model: Option<&str>,
    input_tokens: u64,
    output_tokens: u64,
    reasoning_tokens: u64,
    cache_creation_tokens: u64,
    cache_read_tokens: u64,
    stop_reason: Option<&str>,
    harness_reported_cost_usd: Option<f64>,
) -> anyhow::Result<()> {
    let payload = agent_def::canonical_session::TurnEnded {
        session_id: actor_id.to_string(),
        idx: 0, // synth path doesn't track turn index
        stop_reason: stop_reason.map(str::to_string),
        harness_session_id: None,
        model: model.map(str::to_string),
        input_tokens: Some(input_tokens),
        output_tokens: Some(output_tokens),
        reasoning_tokens: if reasoning_tokens > 0 { Some(reasoning_tokens) } else { None },
        cache_creation_tokens: if cache_creation_tokens > 0 { Some(cache_creation_tokens) } else { None },
        cache_read_tokens: if cache_read_tokens > 0 { Some(cache_read_tokens) } else { None },
        harness_reported_cost_usd,
        observed_at: Some(now_iso8601()),
    };
    emit(actor_id, event_types::TURN_ENDED, &payload)
}
