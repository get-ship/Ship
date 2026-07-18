//! Idle reaper for the unified session registry, and the agent-rules
//! loader the session-spawn path uses to prepend `rules.inline` to the
//! harness prompt.

use chrono::Utc;

use ship_config::agents_index::AgentDefIndex;

use super::registry::Registry;
use super::types::SessionStatus;

/// Seconds of output-quiet before a thinking session flips back to idle.
const IDLE_AFTER_SECS: i64 = 3;

/// Idle-reaper: flips Thinking → Idle after a quiet window. Run this on a
/// timer in the daemon's main tick, or expose as a helper the API can call
/// before serving `meta()`. Cheap — just compares clock.
pub fn sweep_idle(registry: &Registry) {
    let now = Utc::now();
    for meta in registry.list_all() {
        if meta.status != SessionStatus::Thinking {
            continue;
        }
        if (now - meta.last_activity).num_seconds() < IDLE_AFTER_SECS {
            continue;
        }
        if let Some(entry) = registry.get(&meta.id)
            && entry.meta_snapshot().status == SessionStatus::Thinking
            && (Utc::now() - entry.meta_snapshot().last_activity).num_seconds() >= IDLE_AFTER_SECS
        {
            let mut m = entry.meta.lock();
            if m.status == SessionStatus::Thinking {
                m.status = SessionStatus::Idle;
            }
        }
    }
}

/// Load an agent's `rules.inline`, prepended with a Ship-specific preamble so
/// the agent knows which session id to reply into. Returns `None` when the def
/// is absent or has no inline rules — callers fall back to vanilla harness
/// behaviour.
///
/// When an [`AgentDefIndex`] is supplied, the inline rules are served from the
/// DataStore snapshot (project scope) — so a running spawn does NOT depend on
/// the `.ship/agents/*.jsonc` file. The file reader is the cold / cross-space
/// fallback (no index, or an agent that lives outside the indexed space).
pub async fn load_agent_rules(
    index: Option<&AgentDefIndex>,
    work_dir: &std::path::Path,
    agent_id: &str,
) -> Option<String> {
    let inline = load_inline_rules(index, work_dir, agent_id).await?;
    if inline.trim().is_empty() {
        return None;
    }
    Some(format!(
        "# Ship agent context\n\nYou are the \"{agent_id}\" agent spawned by the Ship daemon. \
         Your chat session id is in the SHIP_ACTOR_ID environment variable — \
         pass it to the chat_reply MCP tool to reply to the user.\n\n{inline}"
    ))
}

/// Resolve `rules.inline` for `agent_id`: index snapshot first, file fallback.
async fn load_inline_rules(
    index: Option<&AgentDefIndex>,
    work_dir: &std::path::Path,
    agent_id: &str,
) -> Option<String> {
    if let Some(idx) = index
        && let Ok(Some(doc)) = idx.get_doc(agent_id, "project").await
    {
        return doc
            .get("rules")
            .and_then(|r| r.get("inline"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
    }
    let ship_dir = work_dir.to_path_buf();
    let fs = ship_substrate::ports::LocalFsBroker::new();
    ship_config::agents::read_agent_profile(&fs, &ship_dir, agent_id)
        .ok()
        .flatten()
        .and_then(|p| p.rules.inline)
}
