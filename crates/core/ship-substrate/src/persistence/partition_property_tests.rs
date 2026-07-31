//! Partition-isolation properties for the scope-routed event store.
//!
//! Two layers:
//!   (2) `PartitionedSink` route exclusivity — over ARBITRARY `(scope,
//!       scope_key)`, a persisted event lands in exactly ONE per-scope db and
//!       is absent from every other. Driven by proptest.
//!   e2e — the SAME isolation property through the REAL `KernelRouter::emit`
//!       chokepoint (validators → author/scope stamp → sink persist → project),
//!       for Space(A) / Space(B) / Runtime / Account. Broadens the prior
//!       hand-rolled partition-isolation tests into the full emit path.
//!
//! These prove existing invariants — a failure is a real partitioning bug.

use std::path::Path;

use agent_schema::Authority;
use proptest::prelude::*;
use ship_kernel::EventPersist;
use ship_types::envelope::EventEnvelope;
use ship_types::Stamped;
use ship_types::event_registry::EventScope;
use ship_types::{AccountId, CallerKind, DeviceId, EmitContext, ScopeBinding, SpaceId};
use tempfile::tempdir;

use super::event_store::{EventStore, SqliteEventStore};
use super::partitioned_sink::PartitionedSink;
use crate::db;

// ── shared helpers ──────────────────────────────────────────────────────────

/// All per-scope db files that could exist under `root` for the space-key set
/// we exercise. Used to assert an event is present in exactly one.
fn all_scope_paths(root: &Path, space_keys: &[&str]) -> Vec<std::path::PathBuf> {
    let mut v = vec![db::runtime_db_path(root), db::account_db_path(root)];
    for k in space_keys {
        v.push(db::space_db_path(root, k));
    }
    v
}

/// Whether event `id` is present in the db at `path` (missing file → absent).
fn has_event(path: &Path, id: &str) -> bool {
    if !path.exists() {
        return false;
    }
    SqliteEventStore::new_at_db_path(path)
        .unwrap()
        .get(id)
        .unwrap()
        .is_some()
}

/// The db file an event with `(scope, scope_key)` is expected to route to.
fn expected_path(root: &Path, scope: EventScope, scope_key: &str) -> std::path::PathBuf {
    match scope {
        EventScope::Space => db::space_db_path(root, scope_key),
        EventScope::Account => db::account_db_path(root),
        EventScope::Runtime => db::runtime_db_path(root),
    }
}

// ── (2) PartitionedSink route exclusivity (proptest) ──────────────────────────

fn scope() -> impl Strategy<Value = EventScope> {
    prop_oneof![
        Just(EventScope::Runtime),
        Just(EventScope::Account),
        Just(EventScope::Space),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// For an arbitrary `(scope, scope_key)`, persisting through the sink lands
    /// the event in EXACTLY the db its scope maps to — present there, absent
    /// from every other scope db. Space keys are drawn from a small alphabet so
    /// the "other spaces" cross-check has siblings to be absent from.
    #[test]
    fn route_lands_in_exactly_one_db(
        scope in scope(),
        key in "[a-z][a-z0-9-]{0,12}",
    ) {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let sink = PartitionedSink::new(root.to_path_buf());

        // I-2: route() refuses a Space event whose key has no `spaces`
        // registry row. Register it for the Space case only — Account/
        // Runtime never consult the registry (route ignores scope_key for
        // them).
        if scope == EventScope::Space {
            crate::testutil::register_space(root, &key).unwrap();
        }

        // Space carries the generated key; Account/Runtime carry one too (the
        // sink ignores it for routing — proves routing is scope-keyed, not
        // scope_key-keyed, for the non-Space scopes).
        let ev = EventEnvelope::new("test.route", "ent-1", &serde_json::json!({"k": 1}))
            .unwrap()
            .with_scope(scope, Some(&key));
        sink.persist(&Stamped::unchecked(ev.clone())).unwrap();

        let want = expected_path(root, scope, &key);
        prop_assert!(has_event(&want, &ev.id), "event must be in its own scope db");

        // Absent from every db that is NOT the target — including a sibling
        // space key the event never belonged to.
        let sibling = format!("{key}-other");
        for path in all_scope_paths(root, &[&key, &sibling]) {
            if path == want {
                continue;
            }
            prop_assert!(
                !has_event(&path, &ev.id),
                "event leaked into {}", path.display()
            );
        }
    }

    /// Two distinct Space keys are independently addressable: each event is in
    /// its own space.db and absent from the other's — for arbitrary key pairs.
    #[test]
    fn distinct_spaces_are_isolated(
        a in "[a-z][a-z0-9-]{0,12}",
        b in "[a-z][a-z0-9-]{0,12}",
    ) {
        prop_assume!(a != b);
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let sink = PartitionedSink::new(root.to_path_buf());

        // I-2: register both fabricated space keys before routing events
        // under them.
        crate::testutil::register_space(root, &a).unwrap();
        crate::testutil::register_space(root, &b).unwrap();

        let mk = |k: &str| {
            EventEnvelope::new("test.route", "ent-1", &serde_json::json!({}))
                .unwrap()
                .with_scope(EventScope::Space, Some(k))
        };
        let ea = mk(&a);
        let eb = mk(&b);
        sink.persist(&Stamped::unchecked(ea.clone())).unwrap();
        sink.persist(&Stamped::unchecked(eb.clone())).unwrap();

        prop_assert!(has_event(&db::space_db_path(root, &a), &ea.id));
        prop_assert!(has_event(&db::space_db_path(root, &b), &eb.id));
        prop_assert!(!has_event(&db::space_db_path(root, &b), &ea.id));
        prop_assert!(!has_event(&db::space_db_path(root, &a), &eb.id));
    }
}

// ── e2e: isolation through the real KernelRouter::emit ────────────────────────

/// Build a Space-bound EmitContext (Runtime caller, daemon-internal trusted
/// binding that names space `id`). `enforce_scope` stamps that space's key.
fn space_ctx(id: &str) -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ScopeBinding::from_session(
            DeviceId::new("dev-e2e"),
            Some(AccountId::new("acct-e2e")),
            Some(SpaceId::new(id)),
            Authority::Space {
                space_id: id.to_string(),
            },
        ),
        session_id: None,
    }
}

/// Account-bound context: binding carries the account id; `enforce_scope`
/// stamps `key_for(Account)`.
fn account_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ScopeBinding::from_session(
            DeviceId::new("dev-e2e"),
            Some(AccountId::new("acct-e2e")),
            None,
            Authority::Sdk,
        ),
        session_id: None,
    }
}

/// Runtime-bound context: any binding; `enforce_scope` stamps the device id.
fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ScopeBinding::daemon_internal(None, Authority::Sdk),
        session_id: None,
    }
}

#[test]
fn cross_scope_emit_persists_and_isolates_per_db() {
    // emit() is async and the sink persists via block_in_place, which requires
    // a multi-thread runtime. proptest can't host async, so this is the fixed
    // Space(A)/Space(B)/Runtime/Account matrix the spec names — driven through
    // the REAL emit chokepoint.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        // I-2: register both fixture space ids before routing Space-scoped
        // events under them.
        crate::testutil::register_space(&root, "A").unwrap();
        crate::testutil::register_space(&root, "B").unwrap();
        let router = crate::events::kernel_router::KernelRouter::new(root.clone()).unwrap();

        // One event per scope, each emitted with its scope chosen on the
        // envelope and the matching binding in its context.
        let space_a = EventEnvelope::new("test.cross", "ent-a", &serde_json::json!({}))
            .unwrap()
            .with_scope(EventScope::Space, None);
        let space_b = EventEnvelope::new("test.cross", "ent-b", &serde_json::json!({}))
            .unwrap()
            .with_scope(EventScope::Space, None);
        let account = EventEnvelope::new("test.cross", "ent-acct", &serde_json::json!({}))
            .unwrap()
            .with_scope(EventScope::Account, None);
        // Runtime event is `actor.created` so the substrate ActorProjection
        // also materializes into runtime.db — proving "persists AND projects
        // into its own db", not merely the raw events row.
        let runtime = EventEnvelope::new(
            "actor.created",
            "actor-e2e",
            &serde_json::json!({
                "kind": "test-worker",
                "environment_type": "local",
                "restart_count": 0
            }),
        )
        .unwrap()
        .with_actor_id("actor-e2e")
        .with_scope(EventScope::Runtime, None);

        let a_id = space_a.id.clone();
        let b_id = space_b.id.clone();
        let acct_id = account.id.clone();
        let rt_id = runtime.id.clone();

        router.emit(space_a, &space_ctx("A")).await.unwrap();
        router.emit(space_b, &space_ctx("B")).await.unwrap();
        router.emit(account, &account_ctx()).await.unwrap();
        router.emit(runtime, &runtime_ctx()).await.unwrap();

        let space_a_db = db::space_db_path(&root, "A");
        let space_b_db = db::space_db_path(&root, "B");
        let account_db = db::account_db_path(&root);
        let runtime_db = db::runtime_db_path(&root);

        // Each event persisted into EXACTLY its own scope db.
        let placements = [
            (&a_id, &space_a_db),
            (&b_id, &space_b_db),
            (&acct_id, &account_db),
            (&rt_id, &runtime_db),
        ];
        let all_dbs = [&space_a_db, &space_b_db, &account_db, &runtime_db];
        for (id, home) in placements {
            assert!(
                has_event(home, id),
                "event {id} must be in {}",
                home.display()
            );
            for other in all_dbs {
                if *other == *home {
                    continue;
                }
                assert!(
                    !has_event(other, id),
                    "event {id} leaked into {}",
                    other.display()
                );
            }
        }

        // The Runtime event PROJECTED into runtime.db (actors row) and that
        // projection row is isolated to runtime.db.
        let actors_count = |path: &Path| -> i64 {
            if !path.exists() {
                return 0;
            }
            let mut conn = db::open_db_at(path).unwrap();
            db::block_on(async {
                sqlx::query_scalar("SELECT COUNT(*) FROM actors WHERE id = ?")
                    .bind("actor-e2e")
                    .fetch_one(&mut conn)
                    .await
            })
            .unwrap_or(0)
        };
        assert_eq!(
            actors_count(&runtime_db),
            1,
            "actors row must project into runtime.db"
        );
        assert_eq!(
            actors_count(&account_db),
            0,
            "actors row must NOT be in account.db"
        );
        assert_eq!(
            actors_count(&space_a_db),
            0,
            "actors row must NOT be in space A db"
        );

        // The kernel emit path routes per-scope into the per-scope dbs; it
        // never creates a global single-store file.
    });
}
