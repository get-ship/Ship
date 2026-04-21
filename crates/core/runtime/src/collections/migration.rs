//! One-time migration of user content from `platform.db` to `data.db` collections.
//!
//! Historically user-authored content (notes, ADRs, targets/capabilities)
//! lived in dedicated tables on `platform.db`. The collections system now
//! owns all user content, so we copy existing rows into the matching
//! built-in collections and record a completion flag so the job never
//! re-runs.
//!
//! The source tables on `platform.db` are **not** deleted. They remain as
//! a safety net until a later release is confident migration has settled.

use anyhow::{Context, Result};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, Row, SqliteConnection};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use super::builtin;
use super::schema::CollectionSchema;
use super::store;
use crate::data::{DataService, SqlParam};

const MIGRATION_KEY: &str = "platform_to_collections_v1";

/// Summary of a migration run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MigrationReport {
    pub already_migrated: bool,
    pub source_missing: bool,
    pub notes_migrated: usize,
    pub adrs_migrated: usize,
    pub specs_migrated: usize,
}

/// Run the one-time migration. Idempotent — a completion flag in `data.db`
/// prevents re-execution.
pub fn migrate_platform_to_collections(
    svc: &mut DataService,
    platform_db_path: &Path,
) -> Result<MigrationReport> {
    ensure_migration_state_table(svc)?;

    if is_migration_complete(svc, MIGRATION_KEY)? {
        return Ok(MigrationReport { already_migrated: true, ..Default::default() });
    }

    let Some(mut platform) = open_platform_readonly(platform_db_path)? else {
        // No platform.db present — nothing to migrate, but mark complete so
        // subsequent startups don't probe the filesystem every time.
        mark_migration_complete(svc, MIGRATION_KEY)?;
        return Ok(MigrationReport { source_missing: true, ..Default::default() });
    };

    let notes_schema = builtin::notes_schema();
    let adrs_schema = builtin::adrs_schema();
    let specs_schema = builtin::specs_schema();

    // Ensure adrs/specs tables exist for migration targets. These were removed
    // from register_builtin_schemas (they're no longer user-facing built-ins)
    // but the migration still needs them to land legacy data.
    store::ensure_collection_table(svc, &adrs_schema)?;
    store::ensure_collection_table(svc, &specs_schema)?;

    let notes_migrated = migrate_notes(svc, &notes_schema, &mut platform)?;
    let adrs_migrated = migrate_adrs(svc, &adrs_schema, &mut platform)?;
    let specs_migrated = migrate_specs(svc, &specs_schema, &mut platform)?;

    mark_migration_complete(svc, MIGRATION_KEY)?;

    tracing::info!(
        notes = notes_migrated,
        adrs = adrs_migrated,
        specs = specs_migrated,
        "platform.db → data.db migration complete"
    );

    Ok(MigrationReport {
        already_migrated: false,
        source_missing: false,
        notes_migrated,
        adrs_migrated,
        specs_migrated,
    })
}

// ── Migration state ──────────────────────────────────────────────────────────

fn ensure_migration_state_table(svc: &mut DataService) -> Result<()> {
    svc.execute(
        "CREATE TABLE IF NOT EXISTS _collection_migration_state (
            name TEXT PRIMARY KEY,
            completed_at TEXT NOT NULL
         )",
        &[],
    )?;
    Ok(())
}

fn is_migration_complete(svc: &mut DataService, key: &str) -> Result<bool> {
    let rows = svc.query(
        "SELECT name FROM _collection_migration_state WHERE name = ?",
        &[SqlParam::from(key)],
    )?;
    Ok(!rows.is_empty())
}

fn mark_migration_complete(svc: &mut DataService, key: &str) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    svc.execute(
        "INSERT OR REPLACE INTO _collection_migration_state (name, completed_at)
         VALUES (?, ?)",
        &[SqlParam::from(key), SqlParam::from(now.as_str())],
    )?;
    Ok(())
}

// ── Platform db access ───────────────────────────────────────────────────────

fn open_platform_readonly(path: &Path) -> Result<Option<SqliteConnection>> {
    if !path.exists() {
        return Ok(None);
    }
    let mut raw = path.to_string_lossy().replace('\\', "/");
    if !raw.starts_with('/') {
        raw = format!("/{raw}");
    }
    let url = format!("sqlite://{raw}");
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .busy_timeout(Duration::from_secs(5))
        .read_only(true);
    let conn = crate::db::block_on(async { SqliteConnection::connect_with(&opts).await })?;
    Ok(Some(conn))
}

fn table_exists(conn: &mut SqliteConnection, name: &str) -> Result<bool> {
    let row: Option<(String,)> = crate::db::block_on(async {
        sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name = ?")
            .bind(name)
            .fetch_optional(&mut *conn)
            .await
    })?;
    Ok(row.is_some())
}

// ── Notes migration ──────────────────────────────────────────────────────────

fn migrate_notes(
    svc: &mut DataService,
    schema: &CollectionSchema,
    platform: &mut SqliteConnection,
) -> Result<usize> {
    if !table_exists(platform, "note")? {
        return Ok(0);
    }

    let rows = crate::db::block_on(async {
        sqlx::query("SELECT id, title, content, tags_json, branch FROM note")
            .fetch_all(&mut *platform)
            .await
    })?;

    let mut migrated = 0;
    for row in rows {
        let id: String = row.try_get("id")?;
        let title: String = row.try_get("title")?;
        let content: String = row.try_get("content").unwrap_or_default();
        let tags_json: String = row.try_get("tags_json").unwrap_or_else(|_| "[]".into());
        let branch: Option<String> = row.try_get("branch").ok();

        let tags = serde_json::from_str::<serde_json::Value>(&tags_json)
            .unwrap_or_else(|_| serde_json::json!([]));

        let mut data = serde_json::Map::new();
        data.insert("title".into(), serde_json::Value::String(title));
        if !content.is_empty() {
            data.insert("content".into(), serde_json::Value::String(content));
        }
        data.insert("tags".into(), tags);
        if let Some(b) = branch {
            if !b.is_empty() {
                data.insert("category".into(), serde_json::Value::String(b));
            }
        }

        if item_exists(svc, &schema.id, &id)? {
            continue;
        }
        store::create_item(svc, schema, &id, &serde_json::Value::Object(data), None)?;
        migrated += 1;
    }

    Ok(migrated)
}

// ── ADRs migration ───────────────────────────────────────────────────────────

fn migrate_adrs(
    svc: &mut DataService,
    schema: &CollectionSchema,
    platform: &mut SqliteConnection,
) -> Result<usize> {
    if !table_exists(platform, "adr")? {
        return Ok(0);
    }

    let rows = crate::db::block_on(async {
        sqlx::query(
            "SELECT id, title, status, context, decision, tags_json FROM adr"
        )
            .fetch_all(&mut *platform)
            .await
    })?;

    let mut migrated = 0;
    for row in rows {
        let id: String = row.try_get("id")?;
        let title: String = row.try_get("title")?;
        let status: String = row.try_get("status").unwrap_or_else(|_| "proposed".into());
        let context: String = row.try_get("context").unwrap_or_default();
        let decision: String = row.try_get("decision").unwrap_or_default();
        let tags_json: String = row.try_get("tags_json").unwrap_or_else(|_| "[]".into());

        let tags = serde_json::from_str::<serde_json::Value>(&tags_json)
            .unwrap_or_else(|_| serde_json::json!([]));

        let mut data = serde_json::Map::new();
        data.insert("title".into(), serde_json::Value::String(title));
        data.insert(
            "status".into(),
            serde_json::Value::String(map_adr_status(&status)),
        );
        if !context.is_empty() {
            data.insert("context".into(), serde_json::Value::String(context));
        }
        if !decision.is_empty() {
            data.insert("decision".into(), serde_json::Value::String(decision));
        }
        data.insert("tags".into(), tags);

        if item_exists(svc, &schema.id, &id)? {
            continue;
        }
        store::create_item(svc, schema, &id, &serde_json::Value::Object(data), None)?;
        migrated += 1;
    }

    Ok(migrated)
}

fn map_adr_status(raw: &str) -> String {
    match raw.to_ascii_lowercase().as_str() {
        "proposed" | "draft" => "proposed",
        "accepted" | "active" | "final" => "accepted",
        "deprecated" | "retired" => "deprecated",
        "superseded" | "replaced" => "superseded",
        _ => "proposed",
    }
    .to_string()
}

// ── Specs migration (target + capability) ───────────────────────────────────

fn migrate_specs(
    svc: &mut DataService,
    schema: &CollectionSchema,
    platform: &mut SqliteConnection,
) -> Result<usize> {
    if !table_exists(platform, "target")? {
        return Ok(0);
    }

    let rows = crate::db::block_on(async {
        sqlx::query(
            "SELECT id, title, description, status, body_markdown, file_scope_json
             FROM target",
        )
            .fetch_all(&mut *platform)
            .await
    })?;

    let has_capabilities = table_exists(platform, "capability")?;

    let mut migrated = 0;
    for row in rows {
        let id: String = row.try_get("id")?;
        let title: String = row.try_get("title")?;
        let description: Option<String> = row.try_get("description").ok();
        let status: String = row.try_get("status").unwrap_or_else(|_| "draft".into());
        let body_markdown: Option<String> = row.try_get("body_markdown").ok();
        let file_scope_json: Option<String> = row.try_get("file_scope_json").ok();

        let acceptance = if has_capabilities {
            read_capability_titles(platform, &id)?
        } else {
            vec![]
        };

        let scope: serde_json::Value = file_scope_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_else(|| serde_json::json!([]));

        let mut data = serde_json::Map::new();
        data.insert("title".into(), serde_json::Value::String(title));
        data.insert(
            "status".into(),
            serde_json::Value::String(map_spec_status(&status)),
        );

        let description_text = match (description, body_markdown) {
            (Some(d), Some(b)) if !d.is_empty() && !b.is_empty() => format!("{d}\n\n{b}"),
            (Some(d), _) if !d.is_empty() => d,
            (_, Some(b)) if !b.is_empty() => b,
            _ => String::new(),
        };
        if !description_text.is_empty() {
            data.insert(
                "description".into(),
                serde_json::Value::String(description_text),
            );
        }

        if !acceptance.is_empty() {
            let bullets = acceptance
                .iter()
                .map(|t| format!("- {t}"))
                .collect::<Vec<_>>()
                .join("\n");
            data.insert("acceptance".into(), serde_json::Value::String(bullets));
        }
        data.insert("scope".into(), scope);

        if item_exists(svc, &schema.id, &id)? {
            continue;
        }
        store::create_item(svc, schema, &id, &serde_json::Value::Object(data), None)?;
        migrated += 1;
    }

    Ok(migrated)
}

fn read_capability_titles(
    platform: &mut SqliteConnection,
    target_id: &str,
) -> Result<Vec<String>> {
    let rows = crate::db::block_on(async {
        sqlx::query("SELECT title FROM capability WHERE target_id = ? ORDER BY priority DESC, created_at ASC")
            .bind(target_id)
            .fetch_all(&mut *platform)
            .await
    })?;
    Ok(rows
        .iter()
        .filter_map(|r| r.try_get::<String, _>("title").ok())
        .collect())
}

fn map_spec_status(raw: &str) -> String {
    match raw.to_ascii_lowercase().as_str() {
        "draft" | "proposed" | "planned" => "draft",
        "active" | "in-progress" | "in_progress" | "doing" => "in-progress",
        "complete" | "completed" | "done" | "shipped" => "complete",
        "abandoned" | "cancelled" | "canceled" | "dropped" => "abandoned",
        _ => "draft",
    }
    .to_string()
}

fn item_exists(svc: &mut DataService, collection: &str, id: &str) -> Result<bool> {
    let sql = format!("SELECT id FROM `collection_{collection}` WHERE id = ?");
    let rows = svc.query(&sql, &[SqlParam::from(id)])?;
    Ok(!rows.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::builtin::{register_builtin_schemas, ADRS_ID, NOTES_ID, SPECS_ID};
    use crate::collections::query::QueryRequest;

    fn empty_query(collection: &str) -> QueryRequest {
        QueryRequest {
            collection: collection.to_string(),
            filter: Default::default(),
            sort: Default::default(),
            limit: 50,
            offset: 0,
        }
    }

    fn setup_data_service() -> (DataService, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        // Use open_raw to bypass the auto-startup — these tests exercise
        // migration directly rather than letting it run as a side effect.
        let mut svc = DataService::open_raw(&ship_dir).unwrap();
        register_builtin_schemas(&mut svc).unwrap();
        (svc, tmp)
    }

    fn seed_platform_db(path: &Path) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let url = format!("sqlite://{}?mode=rwc", path.display());
        let opts = SqliteConnectOptions::from_str(&url).unwrap().create_if_missing(true);
        let mut conn = crate::db::block_on(async {
            SqliteConnection::connect_with(&opts).await
        })
        .unwrap();

        crate::db::block_on(async {
            sqlx::query(
                "CREATE TABLE note (
                    id TEXT PRIMARY KEY, title TEXT NOT NULL,
                    content TEXT NOT NULL DEFAULT '',
                    tags_json TEXT NOT NULL DEFAULT '[]',
                    branch TEXT,
                    synced_at TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
            )
            .execute(&mut conn)
            .await
        })
        .unwrap();
        crate::db::block_on(async {
            sqlx::query(
                "CREATE TABLE adr (
                    id TEXT PRIMARY KEY, title TEXT NOT NULL,
                    status TEXT NOT NULL DEFAULT 'proposed',
                    date TEXT NOT NULL,
                    context TEXT NOT NULL DEFAULT '',
                    decision TEXT NOT NULL DEFAULT '',
                    tags_json TEXT NOT NULL DEFAULT '[]',
                    supersedes_id TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
            )
            .execute(&mut conn)
            .await
        })
        .unwrap();
        crate::db::block_on(async {
            sqlx::query(
                "CREATE TABLE target (
                    id TEXT PRIMARY KEY, kind TEXT NOT NULL,
                    title TEXT NOT NULL, description TEXT,
                    status TEXT NOT NULL DEFAULT 'active',
                    goal TEXT, phase TEXT, due_date TEXT,
                    body_markdown TEXT, file_scope_json TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
            )
            .execute(&mut conn)
            .await
        })
        .unwrap();
        crate::db::block_on(async {
            sqlx::query(
                "CREATE TABLE capability (
                    id TEXT PRIMARY KEY, target_id TEXT NOT NULL,
                    title TEXT NOT NULL,
                    status TEXT NOT NULL DEFAULT 'aspirational',
                    evidence TEXT, milestone_id TEXT, phase TEXT,
                    acceptance_criteria TEXT NOT NULL DEFAULT '[]',
                    assigned_to TEXT, priority INTEGER NOT NULL DEFAULT 0,
                    file_scope TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
            )
            .execute(&mut conn)
            .await
        })
        .unwrap();

        let now = chrono::Utc::now().to_rfc3339();
        crate::db::block_on(async {
            sqlx::query(
                "INSERT INTO note (id, title, content, tags_json, branch, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind("n1")
            .bind("First note")
            .bind("hello world")
            .bind(r#"["dev","todo"]"#)
            .bind("main")
            .bind(&now)
            .bind(&now)
            .execute(&mut conn)
            .await
        })
        .unwrap();
        crate::db::block_on(async {
            sqlx::query(
                "INSERT INTO adr (id, title, status, date, context, decision, tags_json, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind("a1")
            .bind("Use SQLite")
            .bind("accepted")
            .bind(&now)
            .bind("Need durable storage")
            .bind("SQLite with WAL")
            .bind("[]")
            .bind(&now)
            .bind(&now)
            .execute(&mut conn)
            .await
        })
        .unwrap();
        crate::db::block_on(async {
            sqlx::query(
                "INSERT INTO target (id, kind, title, description, status, body_markdown,
                                     file_scope_json, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind("t1")
            .bind("feature")
            .bind("Ship it")
            .bind("High-level desc")
            .bind("active")
            .bind("Body text")
            .bind(r#"["src/foo.rs"]"#)
            .bind(&now)
            .bind(&now)
            .execute(&mut conn)
            .await
        })
        .unwrap();
        crate::db::block_on(async {
            sqlx::query(
                "INSERT INTO capability (id, target_id, title, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind("c1")
            .bind("t1")
            .bind("Acceptance one")
            .bind(&now)
            .bind(&now)
            .execute(&mut conn)
            .await
        })
        .unwrap();
        crate::db::block_on(async {
            sqlx::query(
                "INSERT INTO capability (id, target_id, title, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind("c2")
            .bind("t1")
            .bind("Acceptance two")
            .bind(&now)
            .bind(&now)
            .execute(&mut conn)
            .await
        })
        .unwrap();

        crate::db::block_on(async { conn.close().await }).unwrap();
    }

    #[test]
    fn migration_without_platform_db_marks_complete() {
        let (mut svc, tmp) = setup_data_service();
        let missing = tmp.path().join("nope").join("platform.db");

        let report = migrate_platform_to_collections(&mut svc, &missing).unwrap();
        assert!(report.source_missing);
        assert!(!report.already_migrated);
        assert_eq!(report.notes_migrated, 0);

        // Second run short-circuits.
        let report2 = migrate_platform_to_collections(&mut svc, &missing).unwrap();
        assert!(report2.already_migrated);
    }

    #[test]
    fn migration_copies_rows_and_is_idempotent() {
        let (mut svc, tmp) = setup_data_service();
        let platform = tmp.path().join("platform.db");
        seed_platform_db(&platform);

        let report = migrate_platform_to_collections(&mut svc, &platform).unwrap();
        assert_eq!(report.notes_migrated, 1);
        assert_eq!(report.adrs_migrated, 1);
        assert_eq!(report.specs_migrated, 1);

        let notes = store::query_items(
            &mut svc,
            &builtin::notes_schema(),
            &empty_query(NOTES_ID),
        )
        .unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, "n1");
        assert_eq!(notes[0].data["title"], "First note");
        assert_eq!(notes[0].data["content"], "hello world");
        assert_eq!(notes[0].data["category"], "main");

        let adrs = store::query_items(
            &mut svc,
            &builtin::adrs_schema(),
            &empty_query(ADRS_ID),
        )
        .unwrap();
        assert_eq!(adrs.len(), 1);
        assert_eq!(adrs[0].data["status"], "accepted");

        let specs = store::query_items(
            &mut svc,
            &builtin::specs_schema(),
            &empty_query(SPECS_ID),
        )
        .unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].data["status"], "in-progress");
        let acceptance = specs[0].data["acceptance"].as_str().unwrap();
        assert!(acceptance.contains("Acceptance one"));
        assert!(acceptance.contains("Acceptance two"));

        // Running again should not duplicate anything.
        let report2 = migrate_platform_to_collections(&mut svc, &platform).unwrap();
        assert!(report2.already_migrated);

        let notes_again = store::query_items(
            &mut svc,
            &builtin::notes_schema(),
            &empty_query(NOTES_ID),
        )
        .unwrap();
        assert_eq!(notes_again.len(), 1, "no duplicates on second run");
    }

    #[test]
    fn migration_empty_tables_completes_cleanly() {
        let (mut svc, tmp) = setup_data_service();
        let platform = tmp.path().join("platform.db");

        // Create platform.db with the tables but no rows.
        let url = format!("sqlite://{}?mode=rwc", platform.display());
        let opts = SqliteConnectOptions::from_str(&url).unwrap().create_if_missing(true);
        let mut conn = crate::db::block_on(async {
            SqliteConnection::connect_with(&opts).await
        }).unwrap();
        for ddl in [
            "CREATE TABLE note (id TEXT PRIMARY KEY, title TEXT, content TEXT, tags_json TEXT, branch TEXT)",
            "CREATE TABLE adr (id TEXT PRIMARY KEY, title TEXT, status TEXT, context TEXT, decision TEXT, tags_json TEXT)",
            "CREATE TABLE target (id TEXT PRIMARY KEY, kind TEXT, title TEXT, description TEXT, status TEXT, body_markdown TEXT, file_scope_json TEXT)",
            "CREATE TABLE capability (id TEXT PRIMARY KEY, target_id TEXT, title TEXT, priority INTEGER, created_at TEXT)",
        ] {
            crate::db::block_on(async { sqlx::query(ddl).execute(&mut conn).await }).unwrap();
        }
        crate::db::block_on(async { conn.close().await }).unwrap();

        let report = migrate_platform_to_collections(&mut svc, &platform).unwrap();
        assert_eq!(report.notes_migrated, 0);
        assert_eq!(report.adrs_migrated, 0);
        assert_eq!(report.specs_migrated, 0);
        assert!(!report.source_missing);
    }

    #[test]
    fn status_mapping() {
        assert_eq!(map_adr_status("accepted"), "accepted");
        assert_eq!(map_adr_status("ACTIVE"), "accepted");
        assert_eq!(map_adr_status("retired"), "deprecated");
        assert_eq!(map_adr_status("replaced"), "superseded");
        assert_eq!(map_adr_status("whatever"), "proposed");

        assert_eq!(map_spec_status("active"), "in-progress");
        assert_eq!(map_spec_status("completed"), "complete");
        assert_eq!(map_spec_status("cancelled"), "abandoned");
        assert_eq!(map_spec_status("unknown"), "draft");
    }

    /// Realistic volume — 500 notes — catches N+1 cost in idempotency checks.
    #[test]
    fn migration_handles_realistic_volume() {
        let (mut svc, tmp) = setup_data_service();
        let platform = tmp.path().join("platform.db");
        seed_platform_db(&platform);

        // Append 500 notes on top of the base seed (which inserts 1).
        let url = format!("sqlite://{}?mode=rwc", platform.display());
        let opts = SqliteConnectOptions::from_str(&url).unwrap().create_if_missing(true);
        let mut conn = crate::db::block_on(async {
            SqliteConnection::connect_with(&opts).await
        })
        .unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        for i in 0..500 {
            let id = format!("bulk-{i:04}");
            crate::db::block_on(async {
                sqlx::query(
                    "INSERT INTO note (id, title, content, tags_json, branch, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&id)
                .bind(format!("bulk note {i}"))
                .bind("bulk content")
                .bind("[]")
                .bind("main")
                .bind(&now)
                .bind(&now)
                .execute(&mut conn)
                .await
            })
            .unwrap();
        }
        crate::db::block_on(async { conn.close().await }).unwrap();

        let report = migrate_platform_to_collections(&mut svc, &platform).unwrap();
        assert_eq!(report.notes_migrated, 501);

        // Re-run must short-circuit even at volume.
        let report2 = migrate_platform_to_collections(&mut svc, &platform).unwrap();
        assert!(report2.already_migrated);
    }

    /// Resume after a partial migration: some target rows already exist (as
    /// would happen if a prior run errored between migrate_notes and
    /// migrate_adrs). The re-run must not duplicate already-copied rows and
    /// must fill in the rest.
    #[test]
    fn migration_resumes_after_partial_failure() {
        let (mut svc, tmp) = setup_data_service();
        let platform = tmp.path().join("platform.db");
        seed_platform_db(&platform);

        // Simulate a prior partial run: migrate just notes successfully, then
        // "crash" before completion is marked.
        let notes_schema = builtin::notes_schema();
        let mut conn_ro = open_platform_readonly(&platform).unwrap().unwrap();
        let first_pass_notes = migrate_notes(&mut svc, &notes_schema, &mut conn_ro).unwrap();
        assert_eq!(first_pass_notes, 1);
        // Completion flag intentionally NOT set — mimics a partial run.
        crate::db::block_on(async { conn_ro.close().await }).unwrap();

        // Now run the full migration. Notes are already present; adrs/specs
        // still need copying. Nothing should duplicate.
        let report = migrate_platform_to_collections(&mut svc, &platform).unwrap();
        assert!(!report.already_migrated);
        assert_eq!(report.notes_migrated, 0, "notes already present — none re-copied");
        assert_eq!(report.adrs_migrated, 1);
        assert_eq!(report.specs_migrated, 1);

        let notes = store::query_items(
            &mut svc,
            &builtin::notes_schema(),
            &empty_query(NOTES_ID),
        )
        .unwrap();
        assert_eq!(notes.len(), 1, "no note duplicates after resume");
    }
}
