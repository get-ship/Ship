//! `space_settings` projection — per-space settings overrides.
//!
//! Materialized from `space.setting.changed` events, this read model holds the
//! per-space overlay that layers OVER the global daemon settings
//! (`settings.state`). Space overrides win per-key; the global table stays the
//! base. Keyed by `(space_id, key)` — last write wins (UPSERT on apply).
//!
//! The space scope rides the event envelope (`space_id`, stamped via
//! `with_context`), NOT the payload — mirror `ConfigFileChanged`. The setting
//! key is the envelope's `entity_id`.
//!
//! See `.artifacts/specs/space/129-per-space-settings.md`.

use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;
use sqlx::{Row, SqliteConnection};

use ship_substrate::db::block_on;
use ship_substrate::projections::registry::Projection;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::SPACE_SETTING_CHANGED;
use ship_types::payload::SpaceSettingChanged;

pub struct SpaceSettingsProjection;

impl Projection for SpaceSettingsProjection {
    fn name(&self) -> &str {
        "space_settings"
    }

    fn event_types(&self) -> &[&str] {
        &[SPACE_SETTING_CHANGED]
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        if event.event_type != SPACE_SETTING_CHANGED {
            return Ok(());
        }
        // The space scope rides the envelope, not the payload. An event with no
        // space_id has no overlay to attach to — skip it (the global tier owns
        // unscoped settings).
        let Some(space_id) = event.space_id().map(str::to_string) else {
            return Ok(());
        };
        let payload: SpaceSettingChanged = serde_json::from_str(&event.payload_json)?;
        block_on(async {
            sqlx::query(
                "INSERT INTO space_settings (space_id, key, value_json, updated_at) \
                 VALUES (?, ?, ?, datetime('now')) \
                 ON CONFLICT(space_id, key) DO UPDATE SET \
                   value_json = excluded.value_json, \
                   updated_at = datetime('now')",
            )
            .bind(&space_id)
            .bind(&payload.key)
            .bind(&payload.value_json)
            .execute(conn)
            .await
        })?;
        Ok(())
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async { sqlx::query("DELETE FROM space_settings").execute(conn).await })?;
        Ok(())
    }
}

/// All overrides for a space, as a `key -> Value` map. The stored `value_json`
/// is parsed back to a `Value` so callers overlay typed JSON onto the global
/// settings object. A row whose `value_json` fails to parse is skipped (the
/// emit path validates JSON, so this is defensive).
pub fn get_all(space_id: &str) -> Result<HashMap<String, Value>> {
    // space.setting.* is Space-scoped → space_settings lives in the bounded
    // spaces/<space_id>/space.db; the `space_id` arg IS the scope key.
    let mut conn = ship_substrate::db::open_space_db(
        &ship_substrate::project::get_global_dir()?,
        space_id,
    )?;
    let rows = block_on(async {
        sqlx::query("SELECT key, value_json FROM space_settings WHERE space_id = ?")
            .bind(space_id)
            .fetch_all(&mut conn)
            .await
    })?;
    let mut map = HashMap::new();
    for row in &rows {
        let key: String = row.get(0);
        let raw: String = row.get(1);
        if let Ok(value) = serde_json::from_str::<Value>(&raw) {
            map.insert(key, value);
        }
    }
    Ok(map)
}

/// A single override for `(space_id, key)`, parsed back to a `Value`. `None`
/// when the key has no space override.
pub fn get(space_id: &str, key: &str) -> Result<Option<Value>> {
    let mut conn = ship_substrate::db::open_space_db(
        &ship_substrate::project::get_global_dir()?,
        space_id,
    )?;
    let row = block_on(async {
        sqlx::query("SELECT value_json FROM space_settings WHERE space_id = ? AND key = ?")
            .bind(space_id)
            .bind(key)
            .fetch_optional(&mut conn)
            .await
    })?;
    Ok(row.and_then(|r| {
        let raw: String = r.get(0);
        serde_json::from_str::<Value>(&raw).ok()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

     fn fresh() -> (tempfile::TempDir, sqlx::SqliteConnection) {
         let tmp = tempfile::tempdir().unwrap();
         let path = tmp.path().join("scope.db");
         ship_substrate::db::ensure_events_db_at_path(&path).unwrap();
         let conn = ship_substrate::db::open_db_at(&path).unwrap();
         (tmp, conn)
     }

    /// Build a `space.setting.changed` event with the space riding the envelope
    /// (`with_context`), the key as `entity_id`, and the value in the payload.
    fn changed(space_id: &str, key: &str, value_json: &str) -> EventEnvelope {
        let payload = SpaceSettingChanged { key: key.to_string(), value_json: value_json.to_string() };
        EventEnvelope::new(SPACE_SETTING_CHANGED, key, &payload)
            .unwrap()
            .with_context(Some(space_id), None)
    }

    /// Direct read of a stored override, off the test's own connection (the
    /// `get`/`get_all` helpers read the ambient thread-local DB, not this temp
    /// one — so the projection's effect is asserted on `conn` directly).
    fn stored(conn: &mut sqlx::SqliteConnection, space_id: &str, key: &str) -> Option<String> {
        block_on(async {
            sqlx::query("SELECT value_json FROM space_settings WHERE space_id = ? AND key = ?")
                .bind(space_id)
                .bind(key)
                .fetch_optional(conn)
                .await
        })
        .unwrap()
        .map(|r| r.get::<String, _>(0))
    }

    #[test]
    fn upsert_last_write_wins_for_same_space_key() {
        let (_tmp, mut conn) = fresh();
        let proj = SpaceSettingsProjection;
        proj.apply(&changed("space-a", "theme", "\"light\""), &mut conn).unwrap();
        proj.apply(&changed("space-a", "theme", "\"dark\""), &mut conn).unwrap();
        // Two events for the same (space, key) collapse to one row, last wins.
        assert_eq!(stored(&mut conn, "space-a", "theme"), Some("\"dark\"".to_string()));
        let count: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM space_settings WHERE space_id = ? AND key = ?",
            )
            .bind("space-a")
            .bind("theme")
            .fetch_one(&mut conn)
            .await
        })
        .unwrap();
        assert_eq!(count, 1, "upsert keeps a single row per (space, key)");
    }

    #[test]
    fn different_spaces_are_isolated() {
        let (_tmp, mut conn) = fresh();
        let proj = SpaceSettingsProjection;
        proj.apply(&changed("space-a", "theme", "\"dark\""), &mut conn).unwrap();
        proj.apply(&changed("space-b", "theme", "\"light\""), &mut conn).unwrap();
        assert_eq!(stored(&mut conn, "space-a", "theme"), Some("\"dark\"".to_string()));
        assert_eq!(stored(&mut conn, "space-b", "theme"), Some("\"light\"".to_string()));
    }

    #[test]
    fn event_without_space_id_is_skipped() {
        let (_tmp, mut conn) = fresh();
        // No `with_context` → no space on the envelope → no overlay row.
        let payload = SpaceSettingChanged {
            key: "theme".to_string(),
            value_json: "\"dark\"".to_string(),
        };
        let event = EventEnvelope::new(SPACE_SETTING_CHANGED, "theme", &payload).unwrap();
        SpaceSettingsProjection.apply(&event, &mut conn).unwrap();
        let count: i64 = block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM space_settings")
                .fetch_one(&mut conn)
                .await
        })
        .unwrap();
        assert_eq!(count, 0, "an unscoped event has no space overlay to attach to");
    }
}
