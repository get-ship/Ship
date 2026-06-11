//! Behavioral tests for DecisionService — decisions built-in collection.
//!
//! All tests use in-memory / temp-dir ports — no network, no real daemon.
//! Tests map directly to the design tiers in
//! `.artifacts/prep-decisions-collection-design.md`.

use std::sync::Arc;
use std::time::Duration;

use ship_collection::builtin::{all_builtin_schemas, decisions_schema};
use ship_collection::decisions::{DecisionService, RecordInput, RecordResult, COLLECTION};
use ship_collection::error::CollectionError;
use ship_collection::projection::{SchemaRegistry, ensure_collection_table};
use ship_collection::service::CollectionService;
use ship_collection::types::RESERVED_IDS;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::home::ShipHome;
use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

mod common;

// ── Fixtures ─────────────────────────────────────────────────────────────────

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    common::install_collection_projection();
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

fn make_ctx(tmp: &tempfile::TempDir, kernel: Arc<KernelLog>) -> Arc<SpaceContext> {
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    let space_db = ship_substrate::db::space_db_path(&ship_dir, "test");
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, "test"));
    Arc::new(SpaceContext::new(SpaceId::new("test"), store, kernel))
}

fn make_decision_service(ctx: Arc<SpaceContext>) -> DecisionService {
    DecisionService {
        inner: CollectionService { ctx, schemas: Arc::new(SchemaRegistry::new()) },
    }
}

fn ri<'a>(kind: &'a str, title: &'a str) -> RecordInput<'a> {
    RecordInput { kind, title, rationale: None, conviction: None, tags: &[] }
}

fn ri_conviction<'a>(kind: &'a str, title: &'a str, conviction: &'a str) -> RecordInput<'a> {
    RecordInput { kind, title, rationale: None, conviction: Some(conviction), tags: &[] }
}

// ── Tier 1: Schema and Registry ───────────────────────────────────────────────

/// decisions schema is present in all_builtin_schemas and has system=true.
#[test]
fn decisions_schema_in_list() {
    let schemas = all_builtin_schemas();
    let d = schemas.iter().find(|s| s.id == "decisions").expect("decisions must be in all_builtin_schemas");
    assert!(d.system, "decisions must have system=true");
    assert_eq!(d.title.as_deref(), Some("Decisions"));
}

/// Creating a collection named "decisions" must be rejected (reserved id).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn decisions_id_is_reserved() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = CollectionService { ctx, schemas: Arc::new(SchemaRegistry::new()) };

    let schema_json = serde_json::to_vec(&serde_json::json!({
        "id": "decisions",
        "fields": { "title": { "type": "string" } }
    }))
    .unwrap();

    let result = svc.create_schema("decisions", &schema_json).await;
    assert!(result.is_err(), "reserved id 'decisions' must be rejected");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("reserved"), "err: {err}");
}

/// decisions is in the RESERVED_IDS constant.
#[test]
fn decisions_in_reserved_ids() {
    assert!(RESERVED_IDS.contains(&"decisions"), "'decisions' must be in RESERVED_IDS");
}

/// Decisions schema has exactly the 9 declared fields.
#[test]
fn decisions_schema_field_set() {
    let schema = decisions_schema();
    let expected = ["kind", "status", "title", "rationale", "conviction", "tags", "confirmations", "supersedes", "superseded_by"];
    for name in &expected {
        assert!(schema.fields.contains_key(*name), "decisions schema must have field '{name}'");
    }
    assert_eq!(schema.fields.len(), expected.len(), "decisions schema must have exactly {} fields", expected.len());
}

// ── Tier 2: record semantics ──────────────────────────────────────────────────

/// A new record returns Created and the item is retrievable.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn record_creates_item_and_returns_created() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let result = svc.record(ri("operational", "Use WAL mode")).await.unwrap();
    let id = match result {
        RecordResult::Created { id } => id,
        RecordResult::Reinforced { .. } => panic!("expected Created"),
    };
    let item = svc.get(&id).await.unwrap();
    assert_eq!(item.data["title"].as_str(), Some("Use WAL mode"));
    assert_eq!(item.data["status"].as_str(), Some("active"));
    assert_eq!(item.data["kind"].as_str(), Some("operational"));
}

/// Second record with same kind+title reinforces (confirmations=2).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn record_reinforces_on_exact_title_match() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let _ = svc.record(ri("operational", "Use WAL mode")).await.unwrap();
    let result = svc.record(ri("operational", "Use WAL mode")).await.unwrap();

    let (id, confirmations) = match result {
        RecordResult::Reinforced { id, confirmations } => (id, confirmations),
        RecordResult::Created { .. } => panic!("expected Reinforced"),
    };
    assert_eq!(confirmations, 2, "confirmations must be 2 on first reinforce");

    let item = svc.get(&id).await.unwrap();
    let stored_conf = item.data["confirmations"].as_f64().unwrap_or(0.0);
    assert_eq!(stored_conf as u64, 2, "stored confirmations must be 2");
}

/// Reinforce is case-insensitive on title.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn record_reinforces_case_insensitive() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let _ = svc.record(ri("operational", "Use WAL mode")).await.unwrap();
    let result = svc.record(ri("operational", "use wal mode")).await.unwrap();

    match result {
        RecordResult::Reinforced { confirmations, .. } => {
            assert_eq!(confirmations, 2, "case-insensitive match must reinforce");
        }
        RecordResult::Created { .. } => panic!("expected Reinforced for case-insensitive match"),
    }
}

/// Conviction advances one tier per reinforce and is capped at "high".
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn record_advances_conviction_on_reinforce() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    // Start at "low".
    let first = svc.record(ri_conviction("operational", "WAL mode", "low")).await.unwrap();
    let id = match first { RecordResult::Created { id } => id, _ => panic!("expected Created") };

    // Reinforce → conviction should advance to "medium".
    svc.record(ri("operational", "WAL mode")).await.unwrap();
    let item = svc.get(&id).await.unwrap();
    assert_eq!(item.data["conviction"].as_str(), Some("medium"), "low → medium after first reinforce");

    // Reinforce again → "high".
    svc.record(ri("operational", "WAL mode")).await.unwrap();
    let item = svc.get(&id).await.unwrap();
    assert_eq!(item.data["conviction"].as_str(), Some("high"), "medium → high after second reinforce");

    // Reinforce again → still "high" (cap).
    svc.record(ri("operational", "WAL mode")).await.unwrap();
    let item = svc.get(&id).await.unwrap();
    assert_eq!(item.data["conviction"].as_str(), Some("high"), "high stays high (capped)");
}

/// Same title with different kind does NOT dedup — two Created results.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn record_does_not_dedup_across_kinds() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let r1 = svc.record(ri("operational", "Use index")).await.unwrap();
    let r2 = svc.record(ri("tooling", "Use index")).await.unwrap();

    assert!(
        matches!(r1, RecordResult::Created { .. }),
        "first record must be Created"
    );
    assert!(
        matches!(r2, RecordResult::Created { .. }),
        "different kind must create a new record, not reinforce"
    );
}

/// A superseded record does not trigger reinforce on the same title.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn record_does_not_dedup_across_status() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    // Create then supersede the original.
    let r1 = svc.record(ri("operational", "Rule A")).await.unwrap();
    let old_id = match r1 { RecordResult::Created { id } => id, _ => panic!() };
    svc.supersede(&old_id, ri("operational", "Rule A v2")).await.unwrap();

    // Recording the same title+kind should create a new active item, not reinforce.
    let r3 = svc.record(ri("operational", "Rule A")).await.unwrap();
    assert!(
        matches!(r3, RecordResult::Created { .. }),
        "must create new item when matching superseded title (not reinforce)"
    );
}

/// Unknown kind is rejected with a validation error.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn record_rejects_unknown_kind() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let result = svc.record(ri("unknown_kind", "Some title")).await;
    assert!(result.is_err(), "unknown kind must be rejected");
    assert!(
        matches!(result.unwrap_err(), CollectionError::Validation(_)),
        "expected Validation error for unknown kind"
    );
}

/// Invalid conviction is rejected with a validation error.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn record_rejects_invalid_conviction() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let result = svc
        .record(RecordInput { kind: "operational", title: "Some title", rationale: None, conviction: Some("extreme"), tags: &[] })
        .await;
    assert!(result.is_err(), "invalid conviction must be rejected");
    assert!(
        matches!(result.unwrap_err(), CollectionError::Validation(_)),
        "expected Validation error for invalid conviction"
    );
}

/// Empty title is rejected with a validation error.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn record_rejects_missing_title() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let result = svc.record(ri("operational", "")).await;
    assert!(result.is_err(), "empty title must be rejected");
    assert!(
        matches!(result.unwrap_err(), CollectionError::Validation(_)),
        "expected Validation error for empty title"
    );
}

// ── Tier 3: supersede semantics ───────────────────────────────────────────────

/// supersede creates a new record and marks the old one as superseded.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn supersede_creates_new_and_marks_old() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_decision_service(ctx);

    let r = svc.record(ri("correction", "Old rule")).await.unwrap();
    let old_id = match r { RecordResult::Created { id } => id, _ => panic!() };

    // Events + rows together (verifier condition): subscribe pre-act.
    let mut rx = kernel.subscribe();
    let result = svc
        .supersede(&old_id, ri("correction", "New rule"))
        .await
        .unwrap();

    let new_id = result.new_id.clone();
    assert_eq!(result.superseded_id, old_id);

    // Old record: status=superseded, superseded_by=new_id.
    let old_item = svc.get(&old_id).await.unwrap();
    assert_eq!(old_item.data["status"].as_str(), Some("superseded"), "old must be superseded");
    assert_eq!(
        old_item.data["superseded_by"].as_str(),
        Some(new_id.as_str()),
        "old must have superseded_by=new_id"
    );

    // New record: status=active, supersedes=old_id.
    let new_item = svc.get(&new_id).await.unwrap();
    assert_eq!(new_item.data["status"].as_str(), Some("active"), "new must be active");
    assert_eq!(
        new_item.data["supersedes"].as_str(),
        Some(old_id.as_str()),
        "new must have supersedes=old_id"
    );
    // The supersede rode exactly the two collection events.
    let mut got = (false, false);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < deadline && !(got.0 && got.1) {
        match tokio::time::timeout(Duration::from_millis(200), rx.recv()).await {
            Ok(Ok(e)) => {
                if e.event_type == "collection.item_created" { got.0 = true; }
                if e.event_type == "collection.item_updated" { got.1 = true; }
            }
            _ => break,
        }
    }
    assert!(got.0, "supersede must emit collection.item_created");
    assert!(got.1, "supersede must emit collection.item_updated");

}

/// Superseding an unknown id returns NotFound.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn supersede_rejects_unknown_old_id() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let result = svc.supersede("does-not-exist", ri("correction", "New")).await;
    assert!(result.is_err(), "unknown old_id must be rejected");
    assert!(
        matches!(result.unwrap_err(), CollectionError::NotFound(_)),
        "expected NotFound for unknown old_id"
    );
}

/// Superseding an already-superseded record returns Validation error.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn supersede_rejects_already_superseded() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let r = svc.record(ri("operational", "First")).await.unwrap();
    let old_id = match r { RecordResult::Created { id } => id, _ => panic!() };
    svc.supersede(&old_id, ri("operational", "Second")).await.unwrap();

    // Try to supersede the now-superseded record again.
    let result = svc.supersede(&old_id, ri("operational", "Third")).await;
    assert!(result.is_err(), "already-superseded must be rejected");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("superseded"),
        "error must mention 'superseded': {err}"
    );
}

/// The new record produced by supersede has a different id from the old one.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn supersede_new_record_distinct_from_old() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let r = svc.record(ri("product", "Old design")).await.unwrap();
    let old_id = match r { RecordResult::Created { id } => id, _ => panic!() };

    let result = svc.supersede(&old_id, ri("product", "New design")).await.unwrap();
    assert_ne!(result.new_id, result.superseded_id, "new and old ids must differ");
}

// ── Tier 4: query and audit ───────────────────────────────────────────────────

/// query filters by kind.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn query_filters_by_kind() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    svc.record(ri("correction", "Fix A")).await.unwrap();
    svc.record(ri("operational", "Op B")).await.unwrap();
    svc.record(ri("correction", "Fix C")).await.unwrap();

    let items = svc.query(Some("correction"), None, None, None, 50).await.unwrap();
    assert_eq!(items.len(), 2, "query by kind='correction' must return 2");
    assert!(items.iter().all(|i| i.data["kind"].as_str() == Some("correction")));

    let items2 = svc.query(Some("operational"), None, None, None, 50).await.unwrap();
    assert_eq!(items2.len(), 1, "query by kind='operational' must return 1");
}

/// query defaults to status='active'; superseded records are excluded.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn query_filters_by_status_active() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let r1 = svc.record(ri("operational", "Active one")).await.unwrap();
    let r2 = svc.record(ri("operational", "To supersede")).await.unwrap();
    let id2 = match r2 { RecordResult::Created { id } => id, _ => panic!() };
    let _ = match r1 { RecordResult::Created { .. } => {} _ => panic!() };

    svc.supersede(&id2, ri("operational", "Replacement")).await.unwrap();

    // Default status='active' query: active items only.
    let items = svc.query(None, None, None, None, 50).await.unwrap();
    assert!(
        items.iter().all(|i| i.data["status"].as_str() == Some("active")),
        "query must only return active items by default"
    );
    // "To supersede" is gone; "Active one" and "Replacement" remain.
    assert_eq!(items.len(), 2, "must have 2 active items");
}

/// query filters by tag.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn query_filters_by_tag() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let db_tags = vec!["db".to_string()];
    let auth_tags = vec!["auth".to_string()];

    svc.record(RecordInput { kind: "operational", title: "DB rule", rationale: None, conviction: None, tags: &db_tags })
        .await
        .unwrap();
    svc.record(RecordInput { kind: "operational", title: "Auth rule", rationale: None, conviction: None, tags: &auth_tags })
        .await
        .unwrap();

    let db_items = svc.query(None, None, None, Some("db"), 50).await.unwrap();
    assert_eq!(db_items.len(), 1, "query tag='db' must return 1 item");
    assert_eq!(db_items[0].data["title"].as_str(), Some("DB rule"));

    let auth_items = svc.query(None, None, None, Some("auth"), 50).await.unwrap();
    assert_eq!(auth_items.len(), 1, "query tag='auth' must return 1 item");

    let empty = svc.query(None, None, None, Some("network"), 50).await.unwrap();
    assert!(empty.is_empty(), "query tag='network' must return empty");
}

/// min_conviction filter keeps items at or above the threshold.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn query_min_conviction_filters_below() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    svc.record(ri_conviction("operational", "Low one", "low")).await.unwrap();
    svc.record(ri_conviction("operational", "Med one", "medium")).await.unwrap();
    svc.record(ri_conviction("operational", "High one", "high")).await.unwrap();

    let high_only = svc.query(None, None, Some("high"), None, 50).await.unwrap();
    assert_eq!(high_only.len(), 1, "min_conviction=high must return only high");
    assert_eq!(high_only[0].data["conviction"].as_str(), Some("high"));

    let med_up = svc.query(None, None, Some("medium"), None, 50).await.unwrap();
    assert_eq!(med_up.len(), 2, "min_conviction=medium must return medium + high");
    assert!(med_up.iter().all(|i| {
        let c = i.data["conviction"].as_str().unwrap_or("low");
        c == "medium" || c == "high"
    }));

    let all = svc.query(None, None, Some("low"), None, 50).await.unwrap();
    assert_eq!(all.len(), 3, "min_conviction=low must return all");
}

/// query on an empty collection returns empty.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn query_returns_empty_on_no_match() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let items = svc.query(None, None, None, None, 50).await.unwrap();
    assert!(items.is_empty(), "empty collection must return empty result");
}

/// audit ranks active records by confirmations descending.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn audit_ranks_by_confirmations_desc() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    // Alpha: reinforce 3 times total (confirmations=3).
    svc.record(ri("operational", "Alpha")).await.unwrap();
    svc.record(ri("operational", "Alpha")).await.unwrap();
    svc.record(ri("operational", "Alpha")).await.unwrap();

    // Beta: one record (confirmations=1).
    svc.record(ri("operational", "Beta")).await.unwrap();

    // Gamma: reinforce 2 times total (confirmations=2).
    svc.record(ri("operational", "Gamma")).await.unwrap();
    svc.record(ri("operational", "Gamma")).await.unwrap();

    let items = svc.audit(10).await.unwrap();
    assert_eq!(items.len(), 3, "audit must return all 3 active items");

    let titles: Vec<&str> = items.iter().map(|i| i.data["title"].as_str().unwrap_or("")).collect();
    assert_eq!(titles[0], "Alpha", "Alpha (confirmations=3) must be first");
    assert_eq!(titles[1], "Gamma", "Gamma (confirmations=2) must be second");
    assert_eq!(titles[2], "Beta", "Beta (confirmations=1) must be last");
}

/// audit excludes superseded records.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn audit_excludes_superseded() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_decision_service(ctx);

    let r = svc.record(ri("operational", "Old one")).await.unwrap();
    let old_id = match r { RecordResult::Created { id } => id, _ => panic!() };
    svc.supersede(&old_id, ri("operational", "New one")).await.unwrap();

    let items = svc.audit(50).await.unwrap();
    assert!(
        items.iter().all(|i| i.data["status"].as_str() == Some("active")),
        "audit must not include superseded records"
    );
    // "New one" is active.
    assert_eq!(items.len(), 1, "audit must return exactly 1 active item");
    assert_eq!(items[0].data["title"].as_str(), Some("New one"));
}

// ── Tier 5: event-sourcing integrity ─────────────────────────────────────────

/// record emits collection.item_created with collection_id == "decisions".
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn record_emits_item_created_event() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_decision_service(ctx);

    let mut rx = kernel.subscribe();

    svc.record(ri("operational", "Emit test")).await.unwrap();

    let found = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match rx.recv().await {
                Ok(e) if e.event_type == "collection.item_created" => return true,
                Ok(_) => continue,
                Err(_) => return false,
            }
        }
    })
    .await
    .expect("timeout waiting for collection.item_created");

    assert!(found, "record must emit collection.item_created");
}

/// supersede emits collection.item_created AND collection.item_updated.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn supersede_emits_two_collection_events() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_decision_service(ctx);

    let r = svc.record(ri("operational", "Old decision")).await.unwrap();
    let old_id = match r { RecordResult::Created { id } => id, _ => panic!() };

    // Subscribe immediately before the act: broadcast receivers never see
    // events sent before subscription, so the record() events cannot leak in —
    // deterministic, no timing-based drain (verifier condition).
    let mut rx = kernel.subscribe();

    let result = svc.supersede(&old_id, ri("operational", "New decision")).await.unwrap();

    let mut got_created = false;
    let mut got_updated = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);

    while tokio::time::Instant::now() < deadline && !(got_created && got_updated) {
        match tokio::time::timeout(Duration::from_millis(200), rx.recv()).await {
            Ok(Ok(e)) => {
                if e.event_type == "collection.item_created" { got_created = true; }
                if e.event_type == "collection.item_updated" { got_updated = true; }
            }
            _ => break,
        }
    }

    assert!(got_created, "supersede must emit collection.item_created");
    assert!(got_updated, "supersede must emit collection.item_updated");

    // Rows + events together (verifier condition): the same operation's final
    // state must match what the events claim happened.
    let old_item = svc.get(&old_id).await.unwrap();
    assert_eq!(old_item.data["status"].as_str(), Some("superseded"));
    assert_eq!(old_item.data["superseded_by"].as_str(), Some(result.new_id.as_str()));
    let new_item = svc.get(&result.new_id).await.unwrap();
    assert_eq!(new_item.data["status"].as_str(), Some("active"));
    assert_eq!(new_item.data["supersedes"].as_str(), Some(old_id.as_str()));
}

/// Items created in one service instance are readable from a new instance on the
/// same DB files (simulates projection persistence across daemon restarts).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn decisions_survives_projection_rebuild() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_decision_service(ctx);

    let r = svc.record(ri("operational", "Durable decision")).await.unwrap();
    let id = match r { RecordResult::Created { id } => id, _ => panic!() };

    // Open a fresh DataService (new connection) on the same DB file.
    let ctx2 = make_ctx(&tmp, kernel);
    // Ensure the decisions table schema on the new DataService connection.
    ensure_collection_table(ctx2.data(), &decisions_schema()).await.unwrap();
    let svc2 = make_decision_service(ctx2);

    let item = svc2.get(&id).await.unwrap();
    assert_eq!(
        item.data["title"].as_str(),
        Some("Durable decision"),
        "item must be readable via new DataService connection"
    );
}

// ── COLLECTION constant ───────────────────────────────────────────────────────

/// The COLLECTION constant is "decisions".
#[test]
fn collection_constant_is_decisions() {
    assert_eq!(COLLECTION, "decisions");
}
