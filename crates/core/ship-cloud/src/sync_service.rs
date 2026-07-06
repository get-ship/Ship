//! Sync service — cursor-based push of elevated events to Ship Cloud.
//!
//! Push cycle: fires every `push_interval_secs`. Reads elevated events from
//! the kernel store since the push cursor, ships them to cloud, advances the
//! cursor.
//!
//! Pull (routing received events to subscribed actors) is Phase 2 — it requires
//! kernel access from within the service task, which doesn't exist yet.
use std::time::Duration;

use anyhow::Result;

use ship_substrate::db::cursors;
use ship_substrate::events::{EventEnvelope, SqliteEventStore, query_events_since};
use ship_substrate::services::ServiceHandler;
use ship_types::payload::SyncPushCompleted;
use crate::{PushResponse, SyncClient};

const PUSH_SCOPE_PREFIX: &str = "push:platform";

/// The sync service's writer binding. `sync.*` are Runtime-scoped daemon
/// service events with no space binding (the service task has no session), so
/// the daemon-internal binding is the authoritative non-space binding
/// `enforce_scope` stamps the Runtime key from.
fn sync_binding() -> ship_types::ScopeBinding {
    ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk)
}

/// Transport abstraction over [`SyncClient`] — enables injection in tests.
pub(crate) trait SyncTransport: Send + 'static {
    fn push_platform_events(
        &self,
        space_id: &str,
        events: &[EventEnvelope],
    ) -> Result<PushResponse>;
}

impl SyncTransport for SyncClient {
    fn push_platform_events(
        &self,
        space_id: &str,
        events: &[EventEnvelope],
    ) -> Result<PushResponse> {
        self.push_platform_events(space_id, events)
    }
}

/// Configuration for the sync service, read from `ship.jsonc` `sync` block.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub endpoint: String,
    pub space_id: String,
    pub push_interval_secs: u64,
    /// Emit a push immediately once this many elevated events have accumulated.
    pub push_threshold: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.getship.dev".to_string(),
            space_id: String::new(),
            push_interval_secs: 30,
            push_threshold: 50,
        }
    }
}

/// Headless sync service implementing [`ServiceHandler`].
///
/// Push cycle fires on the tick interval and on threshold accumulation.
pub struct SyncServiceHandler {
    config: SyncConfig,
    client: Box<dyn SyncTransport>,
    elevated_since_push: usize,
}

impl SyncServiceHandler {
    pub fn new(config: SyncConfig) -> Self {
        let client = Box::new(SyncClient::new(&config.endpoint));
        Self { config, client, elevated_since_push: 0 }
    }

    #[cfg(test)]
    pub(crate) fn with_transport(
        config: SyncConfig,
        client: Box<dyn SyncTransport>,
    ) -> Self {
        Self { config, client, elevated_since_push: 0 }
    }

    fn push_scope(&self) -> String {
        format!("{PUSH_SCOPE_PREFIX}:{}", self.config.space_id)
    }

    fn push_cycle(&mut self, store: &SqliteEventStore) -> Result<()> {
        let push_start = std::time::Instant::now();
        let scope = self.push_scope();
        let cursor = cursors::get(&scope)?;
        let events = query_events_since(cursor.as_deref(), true)?;
        if events.is_empty() {
            return Ok(());
        }
        let resp = self.client.push_platform_events(&self.config.space_id, &events)?;
        cursors::set(&scope, &resp.cursor)?;
        self.elevated_since_push = 0;
        let payload = SyncPushCompleted {
            events_pushed: resp.accepted as u64,
            duration_ms: push_start.elapsed().as_millis() as u64,
        };
        store.append_enforced(
            &EventEnvelope::new(
                "sync.push.completed",
                "sync",
                &serde_json::to_value(&payload)?,
            )?,
            &sync_binding(),
        )?;
        Ok(())
    }
}

impl ServiceHandler for SyncServiceHandler {
    fn name(&self) -> &str {
        "sync"
    }

    fn handle(&mut self, event: &EventEnvelope, store: &SqliteEventStore) -> Result<()> {
        if event.event_type == "sync.trigger.push" {
            return self.push_cycle(store);
        }
        if event.elevated {
            self.elevated_since_push += 1;
            if self.elevated_since_push >= self.config.push_threshold {
                self.push_cycle(store)?;
            }
        }
        Ok(())
    }

    fn on_start(&mut self, store: &SqliteEventStore) -> Result<()> {
        store.append_enforced(
            &EventEnvelope::new(
                "sync.started",
                "sync",
                &serde_json::json!({"space_id": &self.config.space_id}),
            )?,
            &sync_binding(),
        )?;
        Ok(())
    }

    fn on_stop(&mut self, store: &SqliteEventStore) -> Result<()> {
        store.append_enforced(
            &EventEnvelope::new(
                "sync.stopped",
                "sync",
                &serde_json::json!({}),
            )?,
            &sync_binding(),
        )?;
        Ok(())
    }

    fn tick_interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(self.config.push_interval_secs))
    }

    fn on_tick(&mut self, store: &SqliteEventStore) -> Result<()> {
        if let Err(e) = self.push_cycle(store) {
            tracing::warn!("sync push_cycle error: {e}");
            let payload = serde_json::json!({"error": e.to_string()});
            let envelope = EventEnvelope::new("sync.push.failed", "sync", &payload)?;
            if let Err(persist_err) = store.append_enforced(&envelope, &sync_binding()) {
                tracing::error!("sync: failed to persist sync.push.failed event: {persist_err}");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::db::{block_on, open_db_at, open_runtime_db, runtime_db_path};
    use ship_substrate::testutil::init_project_minimal;
    use tempfile::tempdir;

    /// The runtime.db path under the test's thread-local global ship-dir. The
    /// sync test fixtures persist Runtime-scoped events, so they back their
    /// `ActorStore` with `runtime.db` rather than the dead `platform.db`.
    fn test_runtime_db() -> std::path::PathBuf {
        runtime_db_path(&ship_substrate::project::get_global_dir().unwrap())
    }

    struct MockTransport {
        response_cursor: String,
    }

    impl SyncTransport for MockTransport {
        fn push_platform_events(
            &self,
            _space_id: &str,
            _events: &[EventEnvelope],
        ) -> anyhow::Result<PushResponse> {
            Ok(PushResponse {
                accepted: 1,
                cursor: self.response_cursor.clone(),
            })
        }
    }

    struct FailTransport;

    impl SyncTransport for FailTransport {
        fn push_platform_events(
            &self,
            _space_id: &str,
            _events: &[EventEnvelope],
        ) -> anyhow::Result<PushResponse> {
            anyhow::bail!("simulated network error")
        }
    }

    fn setup_global_db() -> tempfile::TempDir {
        let tmp = tempdir().unwrap();
        init_project_minimal(tmp.path()).unwrap();
        // Ensure the runtime.db events schema exists at the thread-local global dir.
        let _ = open_runtime_db(&ship_substrate::project::get_global_dir().unwrap()).unwrap();
        tmp
    }

    fn insert_elevated_event(id: &str) {
        use chrono::Utc;
        let mut conn = open_db_at(&test_runtime_db()).unwrap();
        let now = Utc::now().to_rfc3339();
        block_on(async {
            sqlx::query(
                "INSERT INTO events \
                 (id, event_type, entity_id, actor, payload_json, elevated, created_at) \
                 VALUES (?, 'actor.created', 'ws-1', 'ship', '{}', 1, ?)",
            )
            .bind(id)
            .bind(&now)
            .execute(&mut conn)
            .await
        })
        .unwrap();
    }

    fn make_store(_tmp: &tempfile::TempDir) -> SqliteEventStore {
        // Backed by the already-initialised global runtime.db (setup_global_db).
        // ActorStore is gone (job/actorstore-deletion, 2026-07-06) — the
        // service now writes through `SqliteEventStore::append_enforced`,
        // which runs the same stamp/scope/schema gate every other writer
        // uses.
        SqliteEventStore::new_at_db_path(&test_runtime_db()).unwrap()
    }

    #[test]
    fn push_cycle_advances_cursor_on_success() {
        let _global = setup_global_db();
        insert_elevated_event("01JAAAAAAAAAAAAAAAAAAAAAAA");

        let store_tmp = tempdir().unwrap();
        let store = make_store(&store_tmp);
        let config = SyncConfig {
            space_id: "proj-cursor".to_string(),
            ..SyncConfig::default()
        };
        let mut handler = SyncServiceHandler::with_transport(
            config.clone(),
            Box::new(MockTransport {
                response_cursor: "01JBBBBBBBBBBBBBBBBBBBBBBBB".to_string(),
            }),
        );

        handler.push_cycle(&store).unwrap();

        let scope = format!("push:platform:{}", config.space_id);
        assert_eq!(
            cursors::get(&scope).unwrap(),
            Some("01JBBBBBBBBBBBBBBBBBBBBBBBB".to_string())
        );
    }

    #[test]
    fn push_cycle_is_noop_when_no_events() {
        let _global = setup_global_db();
        // No events inserted — push_cycle must return without touching cursor.

        let store_tmp = tempdir().unwrap();
        let store = make_store(&store_tmp);
        let config = SyncConfig {
            space_id: "proj-empty".to_string(),
            ..SyncConfig::default()
        };
        let mut handler = SyncServiceHandler::with_transport(
            config.clone(),
            Box::new(MockTransport {
                response_cursor: "should-not-appear".to_string(),
            }),
        );

        handler.push_cycle(&store).unwrap();

        let scope = format!("push:platform:{}", config.space_id);
        assert!(cursors::get(&scope).unwrap().is_none());
    }

    #[test]
    fn push_cycle_cursor_not_updated_on_transport_error() {
        let _global = setup_global_db();
        insert_elevated_event("01JCCCCCCCCCCCCCCCCCCCCCCCC");

        let store_tmp = tempdir().unwrap();
        let store = make_store(&store_tmp);
        let config = SyncConfig {
            space_id: "proj-fail".to_string(),
            ..SyncConfig::default()
        };
        let mut handler =
            SyncServiceHandler::with_transport(config.clone(), Box::new(FailTransport));

        assert!(handler.push_cycle(&store).is_err());

        let scope = format!("push:platform:{}", config.space_id);
        assert!(
            cursors::get(&scope).unwrap().is_none(),
            "cursor must not advance on transport failure"
        );
    }
}
