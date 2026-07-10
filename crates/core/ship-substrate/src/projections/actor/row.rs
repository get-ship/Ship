//! Read-side row shape + column mapping for the `actors` table.

/// Read-side row shape returned by `list_actors` / `get_actor`. JSON-friendly,
/// transport-agnostic — the HTTP layer can pass it through directly. All
/// columns mirror the `actors` table after migration 0006.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActorRow {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub space_id: Option<String>,
    pub cwd: Option<String>,
    pub agent_id: Option<String>,
    pub harness: Option<String>,
    pub model: Option<String>,
    pub transport: Option<String>,
    pub persistence: Option<String>,
    pub harness_session_id: Option<String>,
    pub system_prompt_hash: Option<String>,
    pub pid: Option<i64>,
    pub capabilities: Vec<String>,
    pub parent_actor_id: Option<String>,
    pub restart_count: i64,
    pub exit_code: Option<i64>,
    pub exit_reason: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub last_event_at: Option<String>,
    /// Derived: process alive right now (kill -0 probe via /proc).
    /// `false` for dead pids and rows without a pid (services).
    pub live: bool,
}

/// Filter for `list_actors`. All `Option`s default to "no filter."
#[derive(Default, Debug, Clone)]
pub struct ListActorsFilter {
    pub kind: Option<String>,
    pub status: Option<String>,
    pub limit: i64,
}

pub(super) const SELECT_COLS: &str =
    "id, kind, status, space_id, cwd, agent_id, harness, model, \
     transport, persistence, harness_session_id, system_prompt_hash, \
     pid, capabilities_json, parent_actor_id, restart_count, exit_code, \
     exit_reason, duration_ms, created_at, updated_at, last_event_at";

pub(super) fn row_to_actor(row: &sqlx::sqlite::SqliteRow) -> ActorRow {
    use sqlx::Row;
    // hardening-r2 §5.5: a failed column decode is a silently-wrong empty
    // value. Keep the fallback (never panic on read) but warn with row id +
    // column so a torn/forward-incompatible row doesn't masquerade as a
    // legitimate "no value".
    let id: String = row
        .try_get("id")
        .unwrap_or_else(|e| {
            tracing::warn!(column = "id", error = %e, "actor row: id decode failed");
            String::new()
        });
    let caps_json: String = row
        .try_get("capabilities_json")
        .unwrap_or_else(|e| {
            tracing::warn!(row_id = %id, column = "capabilities_json", error = %e, "actor row decode failed");
            "[]".into()
        });
    let capabilities: Vec<String> = match serde_json::from_str(&caps_json) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(row_id = %id, column = "capabilities_json", error = %e,
                "actor row: capabilities_json parse failed");
            Vec::new()
        }
    };
    // SQLite + sqlx sometimes surfaces a NULL pid column as `Some(0)`
    // here (the binding side writes NULL, but the read path collapses).
    // pid 0 is the kernel swapper, never a real child — treat it as
    // "no pid recorded" so `activity_live` can take over for actors
    // whose pid hasn't been wired into the projection yet (Task #14).
    let pid: Option<i64> = row.try_get("pid").ok().filter(|&p| p > 0);
    let status: String = row
        .try_get("status")
        .unwrap_or_else(|e| {
            tracing::warn!(row_id = %id, column = "status", error = %e, "actor row decode failed");
            String::new()
        });
    let last_event_at: Option<String> = row.try_get("last_event_at").ok();
    // Liveness sources, in order of confidence:
    //   1. pid recorded → /proc probe (definitive for daemon-spawned actors)
    //   2. no pid → recent activity (within 90s) means the mailbox is
    //      probably still draining (used by MCP-bridge actors that don't
    //      advertise their pid via actor_spawn).
    let pid_live = pid.map(pid_alive).unwrap_or(false);
    let activity_live = pid.is_none()
        && last_event_at
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|t| (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_seconds() < 90)
            .unwrap_or(false);
    let live = (pid_live || activity_live)
        && status != "stopped"
        && status != "crashed";
    ActorRow {
        id: id.clone(),
        kind: row
            .try_get("kind")
            .unwrap_or_else(|e| {
                tracing::warn!(column = "kind", error = %e, "actor row: kind decode failed");
                String::new()
            }),
        status,
        space_id: row.try_get("space_id").ok(),
        cwd: row.try_get("cwd").ok(),
        agent_id: row.try_get("agent_id").ok(),
        harness: row.try_get("harness").ok(),
        model: row.try_get("model").ok(),
        transport: row.try_get("transport").ok(),
        persistence: row.try_get("persistence").ok(),
        harness_session_id: row.try_get("harness_session_id").ok(),
        system_prompt_hash: row.try_get("system_prompt_hash").ok(),
        pid,
        capabilities,
        parent_actor_id: row.try_get("parent_actor_id").ok(),
        restart_count: row
            .try_get("restart_count")
            .unwrap_or_else(|e| {
                tracing::warn!(row_id = %id, column = "restart_count", error = %e,
                    "actor row decode failed");
                0
            }),
        exit_code: row.try_get("exit_code").ok(),
        exit_reason: row.try_get("exit_reason").ok(),
        duration_ms: row.try_get("duration_ms").ok(),
        created_at: row
            .try_get("created_at")
            .unwrap_or_else(|e| {
                tracing::warn!(row_id = %id, column = "created_at", error = %e,
                    "actor row decode failed");
                String::new()
            }),
        updated_at: row
            .try_get("updated_at")
            .unwrap_or_else(|e| {
                tracing::warn!(row_id = %id, column = "updated_at", error = %e,
                    "actor row decode failed");
                String::new()
            }),
        last_event_at: row.try_get("last_event_at").ok(),
        live,
    }
}

/// Check if a pid exists. Used to derive real liveness for the dashboard
/// since pipeline `actor.stopped` events don't fire today and chat actor
/// `created` status is sticky after process exit.
///
/// Linux: `/proc/<pid>` exists iff the process is alive.
pub fn pid_alive(pid: i64) -> bool {
    if pid <= 0 {
        return false;
    }
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}
