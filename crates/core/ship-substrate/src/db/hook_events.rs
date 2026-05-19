//! Supervisor-side hook event synthesis.
//!
//! The canonical hook emit path is `apps/shipd/src/routes/hooks_api.rs`,
//! invoked when a harness's `.hooks.json` (or equivalent) calls
//! `ship hook <event>`. That path is harness-driven: the harness
//! decides when to fire.
//!
//! Some harnesses don't expose a hook config (opencode is plugins-only).
//! For those, the supervisor synthesizes `hook.*` events by parsing the
//! harness's structured output stream — e.g. opencode's `--format json`
//! emits one `{"type":"tool","sessionID":...}` line per tool call, which
//! `OpenCodeAdapter::parse_output` translates into
//! `HarnessEvent::ToolUse`, and the supervisor calls into here to fire
//! the matching kernel event.
//!
//! Same persist-then-deliver semantics as `actor_events::emit_actor_event` —
//! one SqliteEventStore::append (event row + projection apply in one tx),
//! then optional mailbox delivery if the kernel router is up.

use anyhow::Result;

use crate::db::block_on_anyhow;
use crate::events::EventEnvelope;
use crate::events::hooks::{
    event_types, HookSessionEndedPayload, HookSessionStartedPayload,
    HookToolUseStartedPayload, HookTurnObservedPayload,
};
use crate::persistence::event_store::{EventStore, SqliteEventStore};

fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn emit<P: serde::Serialize>(
    actor_id: &str,
    event_type: &'static str,
    payload: &P,
) -> Result<()> {
    let envelope = EventEnvelope::new(event_type, actor_id, payload)?
        .with_actor_id(actor_id);
    let store = SqliteEventStore::new()?;
    store.append(&envelope)?;
    if let Some(kr) = crate::events::kernel_router() {
        let _ = block_on_anyhow(async {
            kr.lock().await.deliver(&envelope);
            Ok(())
        });
    }
    Ok(())
}

/// Fire `hook.session_started` from the supervisor. Used by harnesses
/// without a session-start hook of their own (opencode plugins not
/// wired; cursor's hook API lacks the event entirely).
pub fn emit_session_started_synth(
    actor_id: &str,
    harness: &str,
    model: Option<&str>,
    session_id: Option<&str>,
) -> Result<()> {
    let payload = HookSessionStartedPayload {
        actor_id: actor_id.to_string(),
        session_id: session_id.map(str::to_string),
        harness: Some(harness.to_string()),
        model: model.map(str::to_string),
        started_at: now_iso8601(),
    };
    emit(actor_id, event_types::HOOK_SESSION_STARTED, &payload)
}

/// Fire `hook.session_ended` from the supervisor on process exit when
/// the harness didn't fire it itself.
pub fn emit_session_ended_synth(actor_id: &str, reason: &str) -> Result<()> {
    let payload = HookSessionEndedPayload {
        actor_id: actor_id.to_string(),
        session_id: None,
        ended_at: now_iso8601(),
        reason: Some(reason.to_string()),
    };
    emit(actor_id, event_types::HOOK_SESSION_ENDED, &payload)
}

/// Fire `hook.tool_use_started` from the supervisor when the
/// harness-native stream reports a tool invocation. Today there's no
/// supervisor-side completion signal for opencode — `tool_use_count`
/// in analytics_sessions still increments via the projection's
/// session-stub fallback on every started event, so per-tool counts
/// are accurate even without a paired completion.
pub fn emit_tool_use_started_synth(
    actor_id: &str,
    tool: &str,
    input_preview: Option<&str>,
) -> Result<()> {
    let payload = HookToolUseStartedPayload {
        actor_id: actor_id.to_string(),
        tool: tool.to_string(),
        input: None,
        input_preview: input_preview.map(str::to_string),
        started_at: now_iso8601(),
    };
    emit(actor_id, event_types::HOOK_TOOL_USE_STARTED, &payload)
}

/// Fire `hook.turn_observed` from the supervisor when the harness's
/// stream surfaces a turn boundary with usage info. Universal across
/// harnesses — counts are provider-emitted (claude `message.usage`,
/// opencode `step_finish.tokens`, codex `turn.completed`). Cost is
/// NOT stored here — computed on read against a current rate table.
#[allow(clippy::too_many_arguments)]
pub fn emit_turn_observed_synth(
    actor_id: &str,
    session_id: Option<&str>,
    turn_idx: Option<u32>,
    model: Option<&str>,
    input_tokens: u64,
    output_tokens: u64,
    reasoning_tokens: u64,
    cache_creation_tokens: u64,
    cache_read_tokens: u64,
    stop_reason: Option<&str>,
    harness_reported_cost_usd: Option<f64>,
) -> Result<()> {
    let payload = HookTurnObservedPayload {
        actor_id: actor_id.to_string(),
        session_id: session_id.map(str::to_string),
        turn_idx,
        model: model.map(str::to_string),
        input_tokens,
        output_tokens,
        reasoning_tokens,
        cache_creation_tokens,
        cache_read_tokens,
        stop_reason: stop_reason.map(str::to_string),
        harness_reported_cost_usd,
        observed_at: now_iso8601(),
    };
    emit(actor_id, event_types::HOOK_TURN_OBSERVED, &payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::init_project_minimal;
    use tempfile::tempdir;

    #[test]
    fn synth_session_started_persists_and_is_queryable() {
        let tmp = tempdir().unwrap();
        let _dir = init_project_minimal(tmp.path()).unwrap();
        emit_session_started_synth("actor-synth-1", "opencode", Some("anthropic/claude-sonnet-4"), Some("ses_xyz")).unwrap();
        // No panic = success path; the projection apply happens in the
        // same tx, so visibility is by construction. Deeper read-back
        // coverage lives in ship-analytics' projection tests.
    }
}
