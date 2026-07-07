//! SQLite-backed [`JournalStore`] — the persisted durable-execution backend.
//!
//! Stores each run's journal as a JSON blob in the `durable_journals` table
//! (created by migration `202606100001_durable_journals.sql`). Save is a full
//! UPSERT — a journal is append-only so a later save always supersedes an
//! earlier one, making a full overwrite correct and simplest. Load returns
//! `None` for unknown run ids (never started / pruned), which is distinct from
//! a backend error.
//!
//! # Substrate choice (P1.0b decision: option A)
//!
//! The journal is run-scoped mutable-append state, not a domain event, so a
//! dedicated table is the honest home. The alternative (fold over a
//! `pipeline.journal.*` event family) matches the R13 design's literal wording
//! but adds an event family + projection for state that churns on every op.
//! Revisit if/when journals must replicate via the event mesh; the trait seam
//! makes swapping the backend non-breaking.

use std::path::Path;

use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{Connection, SqliteConnection};
use tokio::sync::Mutex;

use durable_execution::{Journal, JournalStore, JournalStoreError};

/// SQLite-backed [`JournalStore`].
///
/// Holds a single WAL-mode `SqliteConnection` (behind a `Mutex`) — NOT a pool.
/// The ship-substrate db layer is single-connection throughout; the journal is
/// the one direct-write store on `runtime.db`, so it matches that model instead
/// of standing up a private pool. The `durable_journals` table comes from the
/// ONE shared DDL ([`ship_substrate::db::scope_schema::DURABLE_JOURNALS_DDL`]),
/// so a fresh test db works without the full runtime migration and production
/// `runtime.db` (which already has the table via `RUNTIME_DDL`) is unaffected.
pub struct SqliteJournalStore {
    conn: Mutex<SqliteConnection>,
}

impl SqliteJournalStore {
    /// Open (or create) a journal store at `db_path`.
    ///
    /// Creates the parent directory and the db file if absent, in WAL mode with
    /// a 5-second busy timeout — the standard ship-substrate db-layer config.
    /// Ensures `durable_journals` from the shared DDL before returning.
    ///
    /// Pragma discipline (census §2):
    /// - `synchronous = NORMAL` (WAL-appropriate; sqlite's version-dependent
    ///   default is unstable so this is set explicitly).
    /// - `foreign_keys = ON` re-issued per connection (it is a per-connection
    ///   pragma in sqlite, NOT persisted in the file header like WAL).
    pub async fn open(db_path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let opts = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_secs(5))
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

        let mut conn = SqliteConnection::connect_with(&opts).await?;
        sqlx::query("PRAGMA foreign_keys = ON").execute(&mut conn).await?;
        sqlx::query(ship_substrate::db::scope_schema::DURABLE_JOURNALS_DDL)
            .execute(&mut conn)
            .await?;

        Ok(Self { conn: Mutex::new(conn) })
    }
}

#[async_trait]
impl JournalStore for SqliteJournalStore {
    /// Persist `journal` as the durable state for `run_id`, replacing any
    /// prior state. A journal is append-only, so a later save is always a
    /// superset of an earlier one — a full overwrite (UPSERT) is correct.
    async fn save(&self, run_id: &str, journal: &Journal) -> Result<(), JournalStoreError> {
        let json = serde_json::to_string(journal)
            .map_err(|e| JournalStoreError::Serde(e.to_string()))?;
        sqlx::query(
            "INSERT INTO durable_journals (run_id, journal_json, updated_at)
             VALUES (?, ?, datetime('now'))
             ON CONFLICT(run_id) DO UPDATE SET
               journal_json = excluded.journal_json,
               updated_at   = excluded.updated_at",
        )
        .bind(run_id)
        .bind(&json)
        .execute(&mut *self.conn.lock().await)
        .await
        .map_err(|e| JournalStoreError::Backend(e.to_string()))?;
        Ok(())
    }

    /// Load the persisted journal for `run_id`. Returns `None` if the run is
    /// unknown to this store (never started, or pruned) — distinct from a
    /// backend error.
    async fn load(&self, run_id: &str) -> Result<Option<Journal>, JournalStoreError> {
        let json: Option<String> =
            sqlx::query_scalar("SELECT journal_json FROM durable_journals WHERE run_id = ?")
                .bind(run_id)
                .fetch_optional(&mut *self.conn.lock().await)
                .await
                .map_err(|e| JournalStoreError::Backend(e.to_string()))?;
        match json {
            None => Ok(None),
            Some(json) => {
                let journal = serde_json::from_str(&json)
                    .map_err(|e| JournalStoreError::Serde(e.to_string()))?;
                Ok(Some(journal))
            }
        }
    }

    async fn load_all_incomplete(&self) -> Result<Vec<(String, Journal)>, JournalStoreError> {
        // Suspension is a property of the deserialized journal, not a column —
        // scan all rows and filter. Run counts are small (journals are pruned
        // with their runs); revisit with a `suspended` column if that changes.
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT run_id, journal_json FROM durable_journals ORDER BY run_id")
                .fetch_all(&mut *self.conn.lock().await)
                .await
                .map_err(|e| JournalStoreError::Backend(e.to_string()))?;
        let mut out = Vec::new();
        for (run_id, json) in rows {
            let journal: Journal = serde_json::from_str(&json)
                .map_err(|e| JournalStoreError::Serde(e.to_string()))?;
            if journal.is_suspended() {
                out.push((run_id, journal));
            }
        }
        Ok(out)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::{AtomicU32, Ordering};

    use serde_json::{json, Value};
    use durable_execution::{
        execute, resume, Dispatch, DispatchError, EngineError, ExecCtx, Journal, Program,
        RunStatus,
    };

    // ── Test infrastructure (mirrors durable-execution's counting-dispatch harness) ──

    /// Baseline host: echoes the op as a result value.
    struct Echo;
    #[async_trait]
    impl Dispatch for Echo {
        async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
            Ok(json!({ "op": op, "args": args, "ok": true }))
        }
    }

    /// Counts real dispatches — proves replay never re-executes completed ops.
    struct Counting {
        inner: Arc<dyn Dispatch>,
        calls: Arc<AtomicU32>,
        seen: Arc<Mutex<Vec<String>>>,
    }
    #[async_trait]
    impl Dispatch for Counting {
        async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.seen.lock().unwrap().push(op.to_string());
            self.inner.call(op, args).await
        }
    }

    fn counting() -> (Arc<Counting>, Arc<AtomicU32>, Arc<Mutex<Vec<String>>>) {
        let calls = Arc::new(AtomicU32::new(0));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let c = Arc::new(Counting {
            inner: Arc::new(Echo),
            calls: calls.clone(),
            seen: seen.clone(),
        });
        (c, calls, seen)
    }

    /// Program: two sequential ops.
    struct TwoReads;
    #[async_trait]
    impl Program for TwoReads {
        async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
            let a = ctx.op("collections.read", json!({})).await?;
            let b = ctx.op("collections.get", json!({})).await?;
            Ok(json!({ "a": a, "b": b }))
        }
    }

    /// Program: read → suspend → write.
    struct ReadApproveWrite;
    #[async_trait]
    impl Program for ReadApproveWrite {
        async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
            let _read = ctx.op("collections.read", json!({})).await?;
            let decision = ctx.suspend("approve the write?").await?;
            let wrote = ctx.op("collections.put", json!({ "approved": decision })).await?;
            Ok(wrote)
        }
    }

    /// Open a fresh journal store in a temporary directory.
    async fn temp_store() -> (tempfile::TempDir, SqliteJournalStore) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = dir.path().join("journals.db");
        let store = SqliteJournalStore::open(&db)
            .await
            .expect("SqliteJournalStore::open");
        (dir, store)
    }

    // ── Behavioral tests ──────────────────────────────────────────────────────

    // Save a completed run, load it back, replay — zero new dispatches. This is
    // the headline durability property: a journal that crosses the SQLite boundary
    // reproduces the exact same run without re-burning any work.
    #[tokio::test]
    async fn save_load_round_trip_replays_with_zero_dispatch() {
        let (_dir, store) = temp_store().await;
        let (gw, calls, _) = counting();

        // Live run: both ops execute.
        let (s1, journal) = execute(Journal::new(), gw.clone(), &TwoReads).await;
        assert!(matches!(s1, RunStatus::Completed(_)));
        assert_eq!(calls.load(Ordering::SeqCst), 2, "live run executes both ops");

        // Save across the SQLite boundary.
        store.save("run-1", &journal).await.expect("save");

        // Load from SQLite and replay — zero new dispatches, same result.
        let loaded = store.load("run-1").await.expect("load").expect("run-1 exists");
        let (s2, _) = execute(loaded, gw, &TwoReads).await;
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "loaded replay re-executes nothing"
        );
        assert_eq!(s1, s2, "replay reproduces the same result");
    }

    // A run PARKED on a suspend survives the store boundary. Save the suspended
    // journal, load it back in a "fresh process", resolve the checkpoint —
    // the pre-suspend op is NOT re-run, the post-suspend op runs exactly once.
    #[tokio::test]
    async fn suspended_run_survives_store_and_resumes_without_re_dispatch() {
        let (_dir, store) = temp_store().await;
        let (gw, calls, seen) = counting();

        // Run to the checkpoint.
        let (s1, journal) = execute(Journal::new(), gw.clone(), &ReadApproveWrite).await;
        assert!(matches!(s1, RunStatus::Suspended), "parks at the checkpoint");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "only the read ran before parking"
        );
        assert!(journal.is_suspended(), "journal records the parked suspend");

        // Persist the parked run and drop it.
        store.save("run-await", &journal).await.expect("save parked run");
        drop(journal);

        // "Restart": load the parked run from the store.
        let reloaded = store
            .load("run-await")
            .await
            .expect("load")
            .expect("exists");
        assert!(
            reloaded.is_suspended(),
            "the parked suspend survived SQLite persistence"
        );

        // Resolve + replay. The read is memoized across the restart; only the
        // write executes.
        let (s2, journal2) = resume(reloaded, json!(true), gw, &ReadApproveWrite).await;
        assert!(
            matches!(s2, RunStatus::Completed(_)),
            "completes after the reloaded resume"
        );
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "read NOT re-run across the restart; only the write executed on resume"
        );
        assert!(!journal2.is_suspended());
        assert_eq!(
            *seen.lock().unwrap(),
            vec!["collections.read", "collections.put"],
            "op order: read once (live), put once (post-resume)"
        );
    }

    // An unknown run id loads as None, not an error. "Never started" is distinct
    // from "backend failed" — the store distinguishes them via the Result<Option>
    // shape.
    #[tokio::test]
    async fn unknown_run_id_loads_none() {
        let (_dir, store) = temp_store().await;
        let result = store.load("never-started").await.expect("load ok");
        assert!(
            result.is_none(),
            "unknown run_id must return None, not an error"
        );
    }

    // A later, longer journal replaces an earlier, shorter one. The engine only
    // ever saves a superset journal (the log grows forward), so the UPSERT must
    // overwrite completely — a partial merge would corrupt replay.
    #[tokio::test]
    async fn longer_journal_overwrites_shorter() {
        let (_dir, store) = temp_store().await;
        let (gw, calls, _) = counting();

        // Run two ops live → 2-entry journal.
        let (_, long_journal) = execute(Journal::new(), gw.clone(), &TwoReads).await;
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "live run fires both ops"
        );
        assert_eq!(long_journal.entries().len(), 2);

        // Save only the FIRST entry as a shorter "prior snapshot".
        // We simulate a shorter journal by running a single-op program.
        struct OneRead;
        #[async_trait]
        impl Program for OneRead {
            async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
                ctx.op("collections.read", json!({})).await
            }
        }
        let (_, short_journal) = execute(Journal::new(), gw.clone(), &OneRead).await;
        assert_eq!(short_journal.entries().len(), 1);

        // Save the shorter journal first, then overwrite with the longer one.
        store.save("run-overwrite", &short_journal).await.expect("save short");
        store.save("run-overwrite", &long_journal).await.expect("save long");

        // Load — must get the LONGER journal (the shorter one is gone).
        let loaded = store
            .load("run-overwrite")
            .await
            .expect("load")
            .expect("exists");
        assert_eq!(
            loaded.entries().len(),
            2,
            "loaded journal must be the longer one (overwrite succeeded)"
        );
        assert_eq!(
            loaded.entries(),
            long_journal.entries(),
            "loaded entries match the long journal"
        );
    }

    // Optional bonus: end-to-end through the persisted store.
    // run → save → load → resume, asserting zero re-dispatch across the store
    // boundary. This exercises the real consumer shape (without needing
    // floki-Stage-2's live wiring).
    #[tokio::test]
    async fn end_to_end_run_save_load_resume_through_persisted_store() {
        let (_dir, store) = temp_store().await;
        let (gw, calls, seen) = counting();

        // 1. Execute to the suspend boundary.
        let (s1, journal) = execute(Journal::new(), gw.clone(), &ReadApproveWrite).await;
        assert!(matches!(s1, RunStatus::Suspended));
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        // 2. Persist via the store.
        store.save("e2e-run", &journal).await.expect("save");

        // 3. Simulate a daemon restart: reload from the persisted store.
        let persisted = store.load("e2e-run").await.expect("load").expect("e2e-run exists");
        assert!(persisted.is_suspended(), "persisted run is parked");

        // 4. External signal arrives — resolve and resume.
        let (s2, final_journal) = resume(persisted, json!("approved"), gw, &ReadApproveWrite).await;
        assert!(matches!(s2, RunStatus::Completed(_)));
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "only the write ran post-resume; the read was memoized across the restart"
        );
        assert!(!final_journal.is_suspended());
        assert_eq!(
            *seen.lock().unwrap(),
            vec!["collections.read", "collections.put"]
        );

        // 5. Save the completed journal.
        store.save("e2e-run", &final_journal).await.expect("save completed");

        // 6. Reload the completed journal — no more suspended state.
        let completed = store.load("e2e-run").await.expect("load").expect("exists");
        assert!(!completed.is_suspended(), "completed journal is not suspended");
        assert_eq!(
            completed.entries().len(),
            3,
            "3 entries: read, suspend-resolved, put"
        );
    }

    // Census §2: every sqlite open must re-issue `synchronous = NORMAL`
    // (WAL-appropriate, the version-dependent default is unstable) and
    // `foreign_keys = ON` (per-connection pragma, not persisted in the file
    // header like WAL). Query both back on the journal store's connection to
    // assert the live connection has them set.
    #[tokio::test]
    async fn open_sets_synchronous_normal_and_foreign_keys_on() {
        let (dir, store) = temp_store().await;
        let _ = dir;
        let mut guard = store.conn.lock().await;

        let synchronous: i64 = sqlx::query_scalar("PRAGMA synchronous")
            .fetch_one(&mut *guard)
            .await
            .expect("PRAGMA synchronous");
        assert_eq!(synchronous, 1, "synchronous must be NORMAL (1)");

        let fk: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(&mut *guard)
            .await
            .expect("PRAGMA foreign_keys");
        assert_eq!(fk, 1, "foreign_keys must be ON (1)");
    }
}
