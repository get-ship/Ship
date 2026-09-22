//! Database integrity check — detect corruption on startup.
//!
//! Runs two checks against each per-scope event db:
//!   1. **Absence detection** (claim c-2026-07-11T01-30-00): the registered
//!      projections own a known set of tables — static tables (declared in
//!      [`scope_schema::static_tables`]) and self-provisioning dynamic tables
//!      (one per `collection.created` / `artifact.created` event). A projection
//!      table that is absent while events for it exist in this scope's log is
//!      drift, not fatal — the rebuild path recreates it from the log.
//!   2. **Storage integrity**: `PRAGMA integrity_check` reports page-level
//!      damage. Projection-only damage is replayable (`ProjectionsCorrupt`);
//!      core-schema (`events`, `cursors`) damage is fatal (`EventsCorrupt`).
//!
//! After the Phase-4 storage partition there is no single platform.db: every
//! scope owns its own db file (`runtime.db`, `account.db`, each mounted
//! `spaces/<id>/space.db`), each carrying its own `events` log and projection
//! tables. The boot path checks each present scope db.

use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use sqlx::Row;
use std::path::{Path, PathBuf};

use crate::db::scope_schema::static_tables;
use crate::db::{block_on, open_db_at};
use crate::events::EventScope;

/// Result of an integrity check.
#[derive(Debug)]
pub enum IntegrityStatus {
    /// Database is healthy.
    Ok,
    /// Projection tables are corrupt or missing but events table is intact.
    /// Caller should trigger a projection rebuild.
    ProjectionsCorrupt { details: String },
    /// Events table or core schema is corrupt. Cannot recover.
    EventsCorrupt { details: String },
}

/// Run the full integrity check on the scope db at `db_path`.
///
/// Order of operations:
///   1. **Static-table absence**: if any projection table declared by
///      [`scope_schema::static_tables`] for this scope is missing AND the
///      scope's event log has at least one event of a type handled by
///      that projection, classify as `ProjectionsCorrupt` (recoverable).
///   2. **Dynamic-table absence**: walk the events log for `collection.*`
///      and `artifact.*` events, resolve each one's physical table
///      (`collection_<id>` for collections; `collection_artifacts` for
///      artifacts). A missing target is `ProjectionsCorrupt`.
///   3. **`PRAGMA integrity_check`**: classify detail rows — any row that
///      references a projection table is recoverable; anything pointing
///      at the core `events` schema is fatal.
///
/// Returns `IntegrityStatus::Ok` when the db has neither storage damage
/// nor drift. Missing-table detection does NOT fire on an empty events
/// log: a fresh db that simply never received events is not drift.
pub fn check_integrity(db_path: &Path) -> Result<IntegrityStatus> {
    check_integrity_inner(db_path)
        .map(|status| prefix_path(db_path, status))
        .with_context(|| format!("integrity check: {}", db_path.display()))
}

/// Prefix a status's `details` with the offending db path, so every
/// diagnostic — whether read via [`check_integrity`] directly or through
/// [`check_scope_integrity`] — names the file to inspect.
fn prefix_path(db_path: &Path, status: IntegrityStatus) -> IntegrityStatus {
    match status {
        IntegrityStatus::ProjectionsCorrupt { details } => IntegrityStatus::ProjectionsCorrupt {
            details: format!("{}: {details}", db_path.display()),
        },
        IntegrityStatus::EventsCorrupt { details } => IntegrityStatus::EventsCorrupt {
            details: format!("{}: {details}", db_path.display()),
        },
        ok => ok,
    }
}

fn check_integrity_inner(db_path: &Path) -> Result<IntegrityStatus> {
    let mut conn = match open_db_at(db_path) {
        Ok(c) => c,
        Err(e) => {
            return Ok(IntegrityStatus::EventsCorrupt {
                details: format!("cannot open database: {e}"),
            });
        }
    };

    let scope = crate::db::scope_for_path(db_path);

    // -- (1)+(2) absence detection -----------------------------------------
    if let Some(drift) = check_absence(&mut conn, scope)? {
        return Ok(IntegrityStatus::ProjectionsCorrupt { details: drift });
    }

    // -- (3) storage integrity check ---------------------------------------
    let rows: Vec<(String,)> = block_on(async {
        sqlx::query_as("PRAGMA integrity_check")
            .fetch_all(&mut conn)
            .await
    })?;

    if rows.len() == 1 && rows[0].0 == "ok" {
        return Ok(IntegrityStatus::Ok);
    }

    let details: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
    let detail_str = details.join("; ");

    // Storage damage. Classify each detail row: a row that names a
    // projection table is replayable; anything pointing at the core
    // events schema is fatal. Projection tables include the static
    // per-scope list + the dynamic `collection_*` / `_schema_registry` /
    // `collection_artifacts` family.
    let projection_only = details
        .iter()
        .all(|d| PROJECTION_TABLE_TAGS.iter().any(|t| d.contains(t)));

    if projection_only {
        Ok(IntegrityStatus::ProjectionsCorrupt {
            details: detail_str,
        })
    } else {
        Ok(IntegrityStatus::EventsCorrupt {
            details: detail_str,
        })
    }
}

/// Substrings that appear in `PRAGMA integrity_check` output to identify a
/// detail row as "projection-only damage" (replayable). Add a new prefix
/// here when a new projection family joins the bus.
///
/// `sqlite_master` corruption also surfaces here with the table name; the
/// dynamic prefixes (`collection_`, `_schema_registry`) catch every
/// per-collection item table without us hand-listing each one.
pub const PROJECTION_TABLE_TAGS: &[&str] = &[
    // Static per-scope tables — listed explicitly so a row that names
    // one is classified as projection-only.
    "actors",
    "canonical_sessions",
    "spaces",
    "project_compile_state",
    "auth_sessions",
    "daemon_owner",
    "provisioned_grants",
    "analytics_sessions",
    "analytics_tool_uses",
    "analytics_turns",
    "cursors",
    "daemon_settings",
    "durable_journals",
    // Dynamic tables — `collection_<id>` and
    // `collection_artifacts` / `_schema_registry` are replayable by
    // CollectionItemProjection / ArtifactProjection from the same
    // event log.
    "collection_",
    "collection_artifacts",
    "_schema_registry",
];

/// Check the scope db for missing projection tables. Returns
/// `Some(details)` describing the drift when a registered projection's
/// table is absent; `None` when the db is intact.
///
/// Two scans:
///   1. **Static tables** (per [`scope_schema::static_tables`]). For each
///      expected table that is missing, we walk the events log for any
///      event whose type is handled by the projection that owns that
///      table. Empty log = no drift.
///   2. **Dynamic tables** (space scope only). Walk `collection.*` events
///      and extract `collection_id`; check `collection_<id>` exists.
///      Walk `artifact.*` events; check `collection_artifacts` exists.
fn check_absence(conn: &mut sqlx::SqliteConnection, scope: EventScope) -> Result<Option<String>> {
    let mut drift: Vec<String> = Vec::new();

    // User-defined tables actually present in the db (excludes sqlite
    // internals + index/trigger objects).
    let present = user_tables(conn)?;
    let present_set: std::collections::HashSet<&str> = present.iter().map(|s| s.as_str()).collect();

    // A scope db with NO tables at all — not even `events` — is a fresh or
    // crash-husk-mid-provision file, not drift: there is no log to check
    // absence against, so treat it as clean rather than erroring on the
    // `events` query below (which would otherwise fail with "no such table").
    if !present_set.contains("events") {
        return Ok(None);
    }

    // Pull event types + payloads once; absence detection only needs
    // "any event of these types exists" + the dynamic collection_id.
    let event_types: Vec<(String, String)> = block_on(async {
        sqlx::query_as::<_, (String, String)>("SELECT event_type, payload_json FROM events")
            .fetch_all(&mut *conn)
            .await
    })
    .context("absence check: SELECT event_type FROM events")?;

    let seen_types: std::collections::HashSet<&str> =
        event_types.iter().map(|(t, _)| t.as_str()).collect();

    // ── (1) static tables ─────────────────────────────────────────────
    for table in static_tables(scope) {
        if present_set.contains(*table) {
            continue;
        }
        // Table missing. Only flag drift when the events log has at
        // least one event of a type owned by the projection that
        // populates this table.
        if any_event_for_table(&seen_types, table) {
            drift.push(format!("missing static table {table:?}"));
        }
    }

    // ── (2) dynamic tables (space scope only) ─────────────────────────
    if matches!(scope, EventScope::Space) {
        for (event_type, payload_json) in &event_types {
            if !event_type.starts_with("collection.") {
                continue;
            }
            let collection_id = collection_id_from_payload(payload_json).unwrap_or_default();
            if collection_id.is_empty() {
                continue;
            }
            let table = format!("collection_{collection_id}");
            if !present_set.contains(table.as_str()) {
                drift.push(format!(
                    "missing dynamic table {table:?} (referenced by {event_type})"
                ));
            }
        }
        if event_types.iter().any(|(t, _)| t.starts_with("artifact."))
            && !present_set.contains("collection_artifacts")
        {
            drift.push(
                "missing dynamic table \"collection_artifacts\" (referenced by artifact.* events)"
                    .to_string(),
            );
        }
    }

    if drift.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format!(
            "absent projection tables (rebuildable): {}",
            drift.join("; ")
        )))
    }
}

/// Read all user-defined table names (excluding sqlite internal + indexes
/// + triggers). Returns the names sorted, deterministic for diagnostics.
fn user_tables(conn: &mut sqlx::SqliteConnection) -> Result<Vec<String>> {
    let rows = block_on(async {
        sqlx::query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .fetch_all(&mut *conn)
        .await
    })?;
    Ok(rows.iter().map(|r| r.get::<String, _>("name")).collect())
}

/// Map a missing static table name to the event types owned by the
/// projection that populates it. Hand-coded against the bus
/// composition at `ship_platform::install_defaults` plus
/// `ActorProjection` (substrate L0). Adding a new projection requires
/// updating this table.
fn any_event_for_table(seen_types: &std::collections::HashSet<&str>, table: &str) -> bool {
    use crate::events::types::event_types as ev;
    let projection_event_types: &[&str] = match table {
        "actors" => &[
            ev::ACTOR_CREATED,
            ev::ACTOR_WOKE,
            ev::ACTOR_SLEPT,
            ev::ACTOR_STOPPED,
            ev::ACTOR_CRASHED,
            crate::events::process::event_types::PROCESS_SPAWNED,
        ],
        "canonical_sessions" => agent_schema::canonical_session::event_types::ALL_CANONICAL_SESSION,
        "project_compile_state" => &[ev::COMPILE_COMPLETED],
        "auth_sessions" => &[
            "auth.session.created",
            "auth.session.revoked",
            "auth.session.expired",
        ],
        "daemon_owner" => &["daemon.owner.bound", "daemon.owner.unbound"],
        "provisioned_grants" => &["agent.grants.provisioned"],
        "analytics_sessions" | "analytics_tool_uses" | "analytics_turns" => &[
            "op.tool_use.started",
            "op.tool_use.completed",
            "session.turn.ended",
            ev::ACTOR_CREATED,
            ev::ACTOR_STOPPED,
        ],
        // The rest (daemon_settings, durable_journals) are
        // non-event-sourced; missing is operator-actionable, not drift.
        _ => return false,
    };

    projection_event_types
        .iter()
        .any(|t| seen_types.contains(t))
}

/// Extract `collection_id` from a `collection.*` event payload JSON.
/// Tolerates malformed payloads so a single broken event never hides a
/// missing table.
fn collection_id_from_payload(payload_json: &str) -> Option<String> {
    let v: Value = match serde_json::from_str(payload_json) {
        Ok(v) => v,
        Err(_) => return None,
    };
    v.get("collection_id")
        .or_else(|| v.get("id"))
        .and_then(|x| x.as_str())
        .map(str::to_string)
}

/// Quick integrity check — runs `PRAGMA quick_check` for a faster partial
/// check (stops at the first error).
pub fn quick_check(db_path: &std::path::Path) -> Result<bool> {
    let mut conn = open_db_at(db_path)?;
    let row: (String,) = block_on(async {
        sqlx::query_as("PRAGMA quick_check")
            .fetch_one(&mut conn)
            .await
    })?;
    Ok(row.0 == "ok")
}

/// The per-scope event db files that exist under `ship_dir`: `runtime.db`,
/// `account.db` (if present), and every mounted `spaces/<id>/space.db`. Only
/// existing files are returned — a scope whose db has never been written is
/// not a corruption candidate.
pub fn scope_db_paths(ship_dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    let runtime = crate::db::runtime_db_path(ship_dir);
    if runtime.exists() {
        paths.push(runtime);
    }
    let account = crate::db::account_db_path(ship_dir);
    if account.exists() {
        paths.push(account);
    }
    let spaces_dir = ship_dir.join(crate::db::SPACES_DIR);
    if let Ok(entries) = std::fs::read_dir(&spaces_dir) {
        for entry in entries.flatten() {
            if let Some(id) = entry.file_name().to_str() {
                let space_db = crate::db::space_db_path(ship_dir, id);
                if space_db.exists() {
                    paths.push(space_db);
                }
            }
        }
    }

    paths
}

/// Check integrity across every present per-scope db under `ship_dir`.
///
/// Returns the most severe status observed: any `EventsCorrupt` wins (fatal),
/// else any `ProjectionsCorrupt` (recoverable by rebuild), else `Ok`. The
/// reported `details` are prefixed with the offending db path so an operator
/// knows which scope to inspect. Errors only when no scope db exists at all.
pub fn check_scope_integrity(ship_dir: &Path) -> Result<IntegrityStatus> {
    let paths = scope_db_paths(ship_dir);
    if paths.is_empty() {
        return Err(anyhow!(
            "no scope event db found under {} (expected runtime.db / account.db / spaces/*/space.db)",
            ship_dir.display()
        ));
    }

    let mut projections_corrupt: Option<String> = None;
    for path in &paths {
        // `check_integrity` already prefixes `details` with the db path.
        match check_integrity(path)? {
            IntegrityStatus::Ok => {}
            IntegrityStatus::ProjectionsCorrupt { details } => {
                projections_corrupt.get_or_insert(details);
            }
            IntegrityStatus::EventsCorrupt { details } => {
                // Fatal — report immediately with the offending db named.
                return Ok(IntegrityStatus::EventsCorrupt { details });
            }
        }
    }

    match projections_corrupt {
        Some(details) => Ok(IntegrityStatus::ProjectionsCorrupt { details }),
        None => Ok(IntegrityStatus::Ok),
    }
}

/// Run the integrity check across the per-scope dbs at `ship_dir`.
///
/// Thin alias for [`check_scope_integrity`] — kept so the boot path and tests
/// can name "the integrity check for this ship-home" without restating the
/// scope-iteration rule.
pub fn check_platform_integrity(ship_dir: &std::path::Path) -> Result<IntegrityStatus> {
    check_scope_integrity(ship_dir)
}
