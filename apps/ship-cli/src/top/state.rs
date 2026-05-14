//! Data fetching for `ship top` — queries runtime state on each tick.

use std::collections::VecDeque;
use std::path::Path;

pub struct TopState {
    pub workspaces: Vec<WorkspaceRow>,
    pub sessions: Vec<SessionRow>,
    pub events: VecDeque<EventRow>,
    pub agents: Vec<AgentRow>,
    pub network_port: Option<String>,
    pub actor_count: usize,
    pub event_cursor: Option<String>,
    pub error: Option<String>,
}

pub struct WorkspaceRow {
    pub branch: String,
    pub status: String,
    pub agent: String,
    pub providers: String,
    pub session_status: String,
}

pub struct SessionRow {
    pub id_short: String,
    pub branch: String,
    pub status: String,
    pub provider: String,
    pub agent: String,
    pub tool_calls: i64,
    pub started: String,
    pub goal: String,
}

pub struct EventRow {
    pub time: String,
    pub event_type: String,
    pub actor: String,
    pub entity: String,
}

pub struct AgentRow {
    pub agent_id: String,
    pub capabilities: String,
    pub status: String,
    pub registered: String,
}

const MAX_EVENTS: usize = 200;

impl TopState {
    pub fn empty() -> Self {
        Self {
            workspaces: vec![],
            sessions: vec![],
            events: VecDeque::with_capacity(MAX_EVENTS),
            agents: vec![],
            network_port: None,
            actor_count: 0,
            event_cursor: None,
            error: None,
        }
    }
}

/// Full refresh of workspaces, sessions, agents, network. Events are incremental.
pub fn refresh(state: &mut TopState, ship_dir: &Path) {
    state.error = None;
    refresh_workspaces(state, ship_dir);
    refresh_sessions(state);
    refresh_events(state, ship_dir);
    refresh_agents(state);
    refresh_network(state);
}

fn refresh_workspaces(state: &mut TopState, _ship_dir: &Path) {
    // The workspace primitive retired (workspace-retirement spec arc).
    // The top view's "workspaces" panel is being repurposed as a branch
    // listing — pending an actual data source. For now leave it empty.
    state.workspaces.clear();
}

fn refresh_sessions(state: &mut TopState) {
    state.sessions.clear();
    match ship_substrate::db::session::list_sessions_db(None, 50) {
        Ok(sessions) => {
            for s in sessions {
                state.sessions.push(SessionRow {
                    id_short: s.id.chars().take(8).collect(),
                    branch: s.branch,
                    status: s.status,
                    provider: s.primary_provider.unwrap_or_else(|| "—".into()),
                    agent: s.agent_id.unwrap_or_else(|| "—".into()),
                    tool_calls: s.tool_call_count,
                    started: format_time(&s.started_at),
                    goal: s.goal.as_deref().unwrap_or("—").chars().take(40).collect(),
                });
            }
        }
        Err(e) => set_error(state, format!("sessions: {e}")),
    }
}

fn refresh_events(state: &mut TopState, ship_dir: &Path) {
    let _ = ship_dir;
    // First load: get recent events. Subsequent: incremental via cursor.
    if state.event_cursor.is_none() {
        match ship_substrate::events::read_recent_events(MAX_EVENTS) {
            Ok(events) => {
                for e in events {
                    let id = e.id.clone();
                    state.events.push_back(to_event_row(e));
                    state.event_cursor = Some(id);
                }
            }
            Err(e) => set_error(state, format!("events: {e}")),
        }
        return;
    }
    match ship_substrate::events::query_events_since(state.event_cursor.as_deref(), false) {
        Ok(new_events) => {
            for e in new_events {
                let id = e.id.clone();
                state.events.push_back(to_event_row(e));
                state.event_cursor = Some(id);
                if state.events.len() > MAX_EVENTS {
                    state.events.pop_front();
                }
            }
        }
        Err(e) => set_error(state, format!("events: {e}")),
    }
}

fn refresh_agents(state: &mut TopState) {
    state.agents.clear();

    // Reconstruct mesh agents from event log.
    if let Ok(events) = ship_substrate::events::query_events_since(None, false) {
        let mut agents: std::collections::HashMap<String, AgentRow> =
            std::collections::HashMap::new();
        for e in &events {
            if e.event_type == "mesh.register"
                && let Ok(p) = serde_json::from_str::<serde_json::Value>(&e.payload_json)
            {
                let agent_id = p["agent_id"].as_str().unwrap_or("unknown");
                let caps: Vec<&str> = p["capabilities"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();
                agents.insert(
                    agent_id.to_string(),
                    AgentRow {
                        agent_id: agent_id.to_string(),
                        capabilities: if caps.is_empty() {
                            "—".into()
                        } else {
                            caps.join(", ")
                        },
                        status: "active".into(),
                        registered: format_time(&e.created_at.to_rfc3339()),
                    },
                );
            } else if e.event_type == "mesh.deregister"
                && let Ok(p) = serde_json::from_str::<serde_json::Value>(&e.payload_json)
                && let Some(id) = p["agent_id"].as_str()
            {
                agents.remove(id);
            } else if e.event_type == "mesh.status"
                && let Ok(p) = serde_json::from_str::<serde_json::Value>(&e.payload_json)
                && let (Some(id), Some(status)) =
                    (p["agent_id"].as_str(), p["status"].as_str())
                && let Some(entry) = agents.get_mut(id)
            {
                entry.status = status.to_string();
            }
        }
        state.agents = agents.into_values().collect();
        state.agents.sort_by(|a, b| a.agent_id.cmp(&b.agent_id));
    }

    // Actor count from KernelRouter (if initialized).
    if let Some(kr) = ship_substrate::events::kernel_router()
        && let Ok(kr_guard) = kr.try_lock()
    {
        state.actor_count = kr_guard.actor_count();
    }
}

fn refresh_network(state: &mut TopState) {
    let port_path =
        ship_substrate::ShipHome::from_env().daemon_file(ship_substrate::home::DaemonFile::Port);
    state.network_port = std::fs::read_to_string(port_path)
        .ok()
        .map(|p| p.trim().to_string());
}

fn to_event_row(e: ship_substrate::events::EventEnvelope) -> EventRow {
    EventRow {
        time: format_time(&e.created_at.to_rfc3339()),
        event_type: e.event_type,
        actor: e
            .actor_id
            .unwrap_or_else(|| e.actor.chars().take(20).collect()),
        entity: e.entity_id.chars().take(30).collect(),
    }
}

fn format_time(rfc3339: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .map(|dt| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|_| rfc3339.chars().take(8).collect())
}

fn set_error(state: &mut TopState, msg: String) {
    if state.error.is_none() {
        state.error = Some(msg);
    }
}
