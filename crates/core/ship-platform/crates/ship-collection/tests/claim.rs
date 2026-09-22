//! Round-trip test for the `claim` collection (Slice 1a — Helm cutover).
//!
//! Mirrors `tests/projection.rs` for the isolated harness
//! (`init_project_minimal` + `new_isolated_for_test` — never touches the operator's real ship home)
//! and asserts that a Helm-shaped claim — typed scalar fields plus a nested
//! `reconciliation` object — round-trips through `CollectionService::create_item`
//! → projection → `query_items_by_query` exactly. Typed columns are read back as
//! their native types; the nested object is stored as JSON-serialized text by
//! the projection (`item_projection::scalar_text` for `Value::Object`) and the
//! substrate's read path re-parses text columns as JSON first
//! (`ship_substrate::data::DataService::query`), so the nested object
//! re-emerges as a `serde_json::Value::Object` — i.e. it survives the
//! round-trip without flattening its sub-fields into columns.
//!
//! Two tests cover the two lifecycle shapes:
//!   1. Reconciled claim — every typed scalar + nested reconciliation object.
//!   2. Archived claim (unreconciled) — `archived_at` set, no reconciliation,
//!      proving the schema accepts the optional-field shape too.

use std::sync::Arc;

use serde_json::json;
use ship_collection::projection::SchemaRegistry;
use ship_collection::service::CollectionService;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test_with;
use ship_substrate::testutil::init_project_minimal;
use ship_substrate::{KernelLog, Query, SingleBackendDataStore, SpaceContext, SpaceId};

mod common;

const COLL: &str = "claim";

/// Build a `CollectionService` whose data store reads from the SAME per-scope
/// `space.db` the projection writes into. Mirrors `projection.rs::make_service`.
fn make_service(ship_dir: &std::path::Path, space_id: &str) -> CollectionService {
    // I-2: register the fabricated `space_id` so `PartitionedSink::route()`
    // doesn't refuse this suite's Space-scoped `collection.*` events.
    ship_substrate::testutil::register_space(ship_dir, space_id).unwrap();
    let router = new_isolated_for_test_with(
        ship_dir.to_path_buf(),
        common::collection_projection_factory(),
    )
    .unwrap();
    let kernel = Arc::new(KernelLog::new(router));

    let space_db = ship_substrate::db::space_db_path(ship_dir, space_id);
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, space_id));
    let ctx = Arc::new(SpaceContext::new(SpaceId::new(space_id), store, kernel));

    CollectionService {
        ctx,
        schemas: Arc::new(SchemaRegistry::new()),
    }
}

/// The Helm claim shape, expressed in the JSON schema format the
/// `.ship/collections/claim.jsonc` file uses. Kept inline (vs. read from the
/// file) so the test fails closed if the schema is edited in a way that
/// invalidates the contract the Helm cutover depends on.
///
/// Field-name note: Helm's source uses `createdAt` (epoch ms). The substrate
/// reserves `created_at` / `updated_at` / `id` / `space_id` as base columns on
/// every collection table and `ship_collection::helpers::row_to_item` filters
/// those keys out of the read-back data, so a user-declared `created_at` is
/// silently dropped on read. We use `claimed_at` here (same type & meaning)
/// to keep the epoch-ms column round-trippable. The Helm→Ship transform at
/// write time maps `createdAt` → `claimed_at`.
fn claim_schema() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "id": COLL,
        "name": "Claims",
        "fields": {
            "kind":          { "type": "enum",   "values": ["bug", "request", "observation", "hitl"], "required": true },
            "by":            { "type": "enum",   "values": ["user", "agent", "external"], "required": true },
            "target":        { "type": "string", "required": true },
            "version":       { "type": "string", "required": true },
            "body":          { "type": "string", "required": true },
            "status":        { "type": "enum",   "values": ["open", "reconciled", "archived"], "required": true },
            "claimed_at":    { "type": "number", "required": true },
            "archived_at":   { "type": "string" },
            "reconciliation": { "type": "object" }
        }
    }))
    .unwrap()
}

fn numeric_column(value: Option<&serde_json::Value>) -> Option<i64> {
    value.and_then(|v| match v {
        // serde_json's `as_i64` returns None when the number is internally
        // f64 (even for integer values); use `as_f64` and round-trip.
        serde_json::Value::Number(n) => n.as_i64().or_else(|| {
            n.as_f64()
                .and_then(|f| if f.is_finite() { Some(f as i64) } else { None })
        }),
        serde_json::Value::String(s) => s.parse::<i64>().ok(),
        _ => None,
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn claim_round_trip_with_nested_reconciliation() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = init_project_minimal(tmp.path()).unwrap();
    let svc = make_service(&ship_dir, "space-claims");

    // 1. Register the claim schema.
    svc.create_schema(COLL, &claim_schema()).await.unwrap();

    // 2. Create a representative Helm-shaped claim: all six typed scalars
    //    + the nested reconciliation object (the bit the spec calls out).
    let claimed_at: i64 = 1_716_000_000_000; // arbitrary epoch ms
    let item = svc
        .create_item(
            COLL,
            &serde_json::to_vec(&json!({
                "kind":    "bug",
                "by":      "agent",
                "target":  "/codebase",
                "version": "main",
                "body":    "ship-collection claim schema diverges from Helm data shape",
                "status":  "reconciled",
                "claimed_at": claimed_at,
                "reconciliation": {
                    "verdict":   "accepted",
                    "at":        "2026-06-30T12:00:00Z",
                    "note":      "Schema mirrors helm/data/claims/*.json — see Slice 1a.",
                    "intentRef": "ship://claim-collection"
                }
            }))
            .unwrap(),
        )
        .await
        .unwrap();

    // 3. Read it back via the projection. Read-after-write is immediate
    //    (projection runs in the same append transaction — see ADR D6/D7).
    let q = Query::new("collection_claim");
    let (items, _) = svc.query_items_by_query(q).await.unwrap();

    assert_eq!(
        items.len(),
        1,
        "projection must materialize exactly one row"
    );
    let row = &items[0];
    assert_eq!(row.id, item.id, "row id must match the generated item id");
    assert_eq!(row.collection, COLL);

    // 4. Typed scalars — every declared typed column survived the round-trip.
    assert_eq!(row.data.get("kind").and_then(|v| v.as_str()), Some("bug"));
    assert_eq!(row.data.get("by").and_then(|v| v.as_str()), Some("agent"));
    assert_eq!(
        row.data.get("target").and_then(|v| v.as_str()),
        Some("/codebase")
    );
    assert_eq!(
        row.data.get("version").and_then(|v| v.as_str()),
        Some("main")
    );
    assert_eq!(
        row.data.get("body").and_then(|v| v.as_str()),
        Some("ship-collection claim schema diverges from Helm data shape")
    );
    assert_eq!(
        row.data.get("status").and_then(|v| v.as_str()),
        Some("reconciled")
    );
    assert_eq!(
        numeric_column(row.data.get("claimed_at")),
        Some(claimed_at),
        "claimed_at (epoch ms) must round-trip; got {:?}",
        row.data.get("claimed_at")
    );

    // 5. Optional `archived_at` left unset — not declared `required`, so the
    //    validator accepts its absence. The projection materializes absent
    //    optional fields as SQL NULL, which the substrate's read path
    //    re-emits as `Value::Null` (the column IS present — the lazy
    //    `ensure_table_for_data` adds it on the first `item_created`).
    let archived_at = row.data.get("archived_at");
    assert!(
        archived_at.map_or(true, |v| v.is_null()),
        "archived_at is optional; absence must round-trip as null, got {:?}",
        archived_at
    );

    // 6. Nested reconciliation object — the explicit Slice 1a requirement.
    //    It is stored as JSON-serialized text in a TEXT column by the
    //    projection and re-parsed by the read path, so it re-emerges as a
    //    JSON object (not a string), with all sub-fields intact.
    let reconciliation = row
        .data
        .get("reconciliation")
        .expect("reconciliation must be present on the read-back row")
        .as_object()
        .expect("reconciliation must round-trip as a JSON object, not a string");
    assert_eq!(
        reconciliation.get("verdict").and_then(|v| v.as_str()),
        Some("accepted")
    );
    assert_eq!(
        reconciliation.get("at").and_then(|v| v.as_str()),
        Some("2026-06-30T12:00:00Z")
    );
    assert_eq!(
        reconciliation.get("note").and_then(|v| v.as_str()),
        Some("Schema mirrors helm/data/claims/*.json — see Slice 1a.")
    );
    assert_eq!(
        reconciliation.get("intentRef").and_then(|v| v.as_str()),
        Some("ship://claim-collection")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn claim_round_trip_with_archived_at_and_unreconciled() {
    // The opposite shape: a still-archived claim with `archived_at` set but
    // no reconciliation. Confirms optional-field and required-field
    // bookkeeping matches Helm's "open / archived / reconciled" lifecycle.
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = init_project_minimal(tmp.path()).unwrap();
    let svc = make_service(&ship_dir, "space-claims");

    svc.create_schema(COLL, &claim_schema()).await.unwrap();

    let claimed_at: i64 = 1_716_000_999_000;
    svc.create_item(
        COLL,
        &serde_json::to_vec(&json!({
            "kind":       "observation",
            "by":         "user",
            "target":     "/codebase",
            "version":    "release/2026.06",
            "body":       "Workflow note — no reconciliation needed.",
            "status":     "archived",
            "claimed_at": claimed_at,
            "archived_at": "2026-06-30T13:30:00Z"
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    let (items, _) = svc
        .query_items_by_query(Query::new("collection_claim"))
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    let row = &items[0];

    assert_eq!(
        row.data.get("status").and_then(|v| v.as_str()),
        Some("archived")
    );
    assert_eq!(
        row.data.get("archived_at").and_then(|v| v.as_str()),
        Some("2026-06-30T13:30:00Z")
    );
    assert_eq!(numeric_column(row.data.get("claimed_at")), Some(claimed_at));

    // No reconciliation on this claim — the field is omitted from the
    // item JSON, but the projection's `ensure_table_for_data` adds a
    // column for it on first write, so the column is present in the
    // table and read back as SQL NULL → `Value::Null`. Either absent
    // or null is the correct substrate contract.
    let reconciliation = row.data.get("reconciliation");
    assert!(
        reconciliation.map_or(true, |v| v.is_null()),
        "reconciliation must be absent or null when not supplied; got {:?}",
        reconciliation
    );
}
