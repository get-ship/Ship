//! Agent-def Watch prefilters + scope keys — the pure, testable half of the
//! unified reactive `AgentService.Watch` (agents-arc E5).
//!
//! The shipd `watch()` handler composes three pieces: a `ReactiveRegistry`
//! (ship-reactive, owns the event loop + diff + fanout), a snapshot closure
//! (`services::registry::call` over the agents.* reads), and the cheap event
//! prefilter that decides whether a kernel event should re-run the snapshot.
//! THIS module owns that prefilter logic — kept here in ship-platform so it is
//! unit-testable WITHOUT the shipd build wall, mirroring `config::agent_index`.
//!
//! Two scopes:
//!   * ALL  — snapshots `{summaries, catalogs}`. Wakes on the broad config
//!     namespace: any of the config-domain change events ([`CONFIG_DOMAIN_EVENTS`]),
//!     because a skill / mcp / rule / preset edit changes the catalogs, not just
//!     an agent-def edit. This is the same broad surface the dropped
//!     `live_read.rs` subscribed to (the `"config"` actor namespace).
//!   * AGENT(id) — snapshots `{detail, wire, settings}` for one id. Wakes ONLY
//!     on that agent's `agent.def.updated` (event_type match + path stem == id),
//!     so another agent's edit does not patch this subscriber.

use ship_types::envelope::EventEnvelope;
use ship_types::event_types;
use ship_types::payload::AgentDefUpdated;

/// The config-domain change events — the `"config"` namespace the old
/// `live_read.rs` woke the all-scope read on. A change to ANY of these can move
/// the agent summaries (agent edits) or the picker catalogs (skill / mcp / rule
/// / preset edits). The ONE place this set is named.
pub const CONFIG_DOMAIN_EVENTS: &[&str] = &[
    event_types::AGENT_CHANGED,
    event_types::AGENT_DEF_UPDATED,
    event_types::SKILL_CHANGED,
    event_types::RULE_CHANGED,
    event_types::VIEW_CHANGED,
    event_types::PIPELINE_CHANGED,
    event_types::COLLECTION_CHANGED,
    event_types::CONFIG_CHANGED,
    event_types::CONFIG_FILE_CHANGED,
    event_types::PERMISSIONS_UPDATED,
];

/// Scope-key fragment for the `WatchKey` of the ALL scope. The full key is
/// `(space_id, scope, query_hash)`; the daemon prepends the bound space_id.
pub const SCOPE_ALL: &str = "agents:all";

/// Build the scope-key fragment for a single-agent watch.
pub fn scope_agent(id: &str) -> String {
    format!("agent:{id}")
}

/// ALL-scope prefilter: does this event possibly change `{summaries, catalogs}`?
/// True for any config-domain change event (see [`CONFIG_DOMAIN_EVENTS`]).
pub fn all_scope_relevant(env: &EventEnvelope) -> bool {
    CONFIG_DOMAIN_EVENTS.contains(&env.event_type.as_str())
}

/// AGENT-scope prefilter: is this `agent.def.updated` for THIS agent id? Matches
/// only `agent.def.updated` whose def path stem equals `id` — another agent's
/// def edit is ignored (prefilter isolation). The id is derived from the
/// payload path the same way the def index does (`<stem>.jsonc`, last path
/// segment), so the two stay in lock-step.
pub fn agent_scope_relevant(env: &EventEnvelope, id: &str) -> bool {
    if env.event_type != event_types::AGENT_DEF_UPDATED {
        return false;
    }
    serde_json::from_str::<AgentDefUpdated>(&env.payload_json)
        .ok()
        .map(|p| def_path_stem(&p.path) == id)
        .unwrap_or(false)
}

/// Agent id from a def file path — the last path segment minus the `.jsonc`
/// suffix. Mirrors `ship_config::agents_index::file_stem` so a watch's id match
/// and the index's row key are derived identically.
fn def_path_stem(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel).trim_end_matches(".jsonc")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn env(event_type: &str, entity: &str, payload: serde_json::Value) -> EventEnvelope {
        EventEnvelope::new(event_type, entity, &payload).unwrap()
    }

    fn def_updated(path: &str) -> EventEnvelope {
        env(
            event_types::AGENT_DEF_UPDATED,
            path,
            json!({ "path": path, "content_hash": "abc" }),
        )
    }

    #[test]
    fn all_scope_wakes_on_agent_def_edit() {
        assert!(all_scope_relevant(&def_updated("foo.jsonc")));
        assert!(all_scope_relevant(&env(event_types::AGENT_CHANGED, "agents", json!({}))));
    }

    #[test]
    fn all_scope_wakes_on_catalog_inputs_not_just_agent_defs() {
        // Catalogs depend on skill / rule (+ mcp/permissions) changes — the
        // broad config-namespace prefilter must wake on those, not just agents.
        assert!(all_scope_relevant(&env(event_types::SKILL_CHANGED, "skills", json!({}))));
        assert!(all_scope_relevant(&env(event_types::RULE_CHANGED, "rules", json!({}))));
        assert!(all_scope_relevant(&env(
            event_types::PERMISSIONS_UPDATED,
            "global",
            json!({}),
        )));
        assert!(all_scope_relevant(&env(
            event_types::CONFIG_FILE_CHANGED,
            "mcp.jsonc",
            json!({ "domain": "mcp", "path": "mcp.jsonc", "content_hash": "" }),
        )));
    }

    #[test]
    fn all_scope_ignores_unrelated_events() {
        assert!(!all_scope_relevant(&env(event_types::SESSION_STARTED, "s1", json!({}))));
        assert!(!all_scope_relevant(&env(event_types::PROCESS_SPAWNED, "p1", json!({}))));
    }

    #[test]
    fn agent_scope_matches_only_its_own_def() {
        assert!(agent_scope_relevant(&def_updated("foo.jsonc"), "foo"));
        assert!(!agent_scope_relevant(&def_updated("bar.jsonc"), "foo"));
    }

    #[test]
    fn agent_scope_handles_nested_paths() {
        assert!(agent_scope_relevant(&def_updated("sub/foo.jsonc"), "foo"));
    }

    #[test]
    fn agent_scope_ignores_non_def_events() {
        // A broad agent.changed (coarse domain) does NOT target one agent — only
        // the fine-grained agent.def.updated carries a def path to match on.
        assert!(!agent_scope_relevant(&env(event_types::AGENT_CHANGED, "agents", json!({})), "foo"));
        assert!(!agent_scope_relevant(&env(event_types::SKILL_CHANGED, "skills", json!({})), "foo"));
    }

    #[test]
    fn scope_keys_are_distinct_per_agent() {
        assert_eq!(scope_agent("foo"), "agent:foo");
        assert_ne!(scope_agent("foo"), scope_agent("bar"));
        assert_ne!(SCOPE_ALL, scope_agent("foo"));
    }
}
