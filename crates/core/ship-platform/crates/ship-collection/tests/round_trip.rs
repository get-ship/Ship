//! `round_trip` — proves the carved `ship-substrate` serves collections
//! end-to-end WITHOUT a daemon, without a network port, and without any of
//! the heavy substrate machinery (object_store / portable-pty / notify).
//!
//! Mirrors the patterns `apps/shipd::build_space_db_context_arc` exercise in
//! production. (`SpaceContext::for_space` is also valid, but it re-initializes
//! the global router, which races with parallel tests in the same cargo-test
//! process; this test pins an isolated router via `new_isolated_for_test` so
//! each invocation owns its kernel — see the helper below.)
//!
//! 1. Pass `CollectionItemProjection` via `new_isolated_for_test_with` so the
//!    kernel's per-scope bus materializes `collection_<id>` tables in the
//!    same SQLite transaction as the event append — see ADR D6/D7.
//! 2. Build the per-scope `SpaceContext` rooted at `<tmp>/.ship/spaces/<id>/`,
//!    the SAME file the kernel router writes Space-scoped events into — so
//!    the `CollectionItemProjection`'s same-TX writes are visible to the
//!    ctx's reads.
//! 3. Round-trip the full collection flow:
//!    - `create_schema` for a tiny 2-field schema
//!    - `create_item`, `get_item` — assert the read-back matches what we wrote
//!    - `query_items_ast` with a WHERE + ORDER BY — assert the filtered set
//!    - Open a collection-scope `Watch`, assert the first frame is a `set`
//!      with the snapshot, then a second `create_item` produces a non-empty
//!      patch frame
//!
//! If any of the above regresses (e.g. a carved-core dep gets reintroduced
//! that pulls in object_store/notify/portable-pty), this test fails — that's
//! the point.

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use ship_collection::projection::SchemaRegistry;
use ship_collection::service::CollectionService;
use ship_reactive::{PatchOp, ReactiveRegistry, ResolverSpec, WatchKey};
use ship_substrate::events::EventFilter;
use ship_substrate::ports::{SpaceContext, SpaceId};

mod common;

const COLL: &str = "rt_notes";

/// Build a fresh per-test fixture rooted at `tmp.path()`. Returns the kernel
/// handle and a `SpaceContext` whose `KernelLog` was bound to the SAME
/// injected router — required so a later `ReactiveRegistry::watch` on
/// `kernel` sees the events `CollectionService::create_item` emits through the
/// ctx. The test holds one `KernelLog` over one isolated router.
fn make_kernel_and_ctx(
    tmp: &tempfile::TempDir,
) -> (Arc<ship_substrate::ports::KernelLog>, Arc<SpaceContext>) {
    // Pass the collection projection into the isolated router via the
    // factory seam (constructor DI — the only legal way to plug an L1
    // projection into the per-scope bus). Isolated builder keeps each
    // test's router off any shared ambient state.
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = ship_substrate::events::new_isolated_for_test_with(
        ship_dir.clone(),
        common::collection_projection_factory(),
    )
    .unwrap();
    let kernel = Arc::new(ship_substrate::ports::KernelLog::new(router));
    // The SpaceContext must bind its `DataService` to the SAME per-scope
    // `spaces/<id>/space.db` the kernel routes `collection.*` (Space-scoped)
    // events to, so the `CollectionItemProjection`'s same-TX writes are
    // visible to the ctx's reads. We mirror the shipd
    // `build_space_db_context_arc` pattern — open `space_db_path` explicitly
    // and wrap with `SingleBackendDataStore` rather than `for_space` (which
    // would open a fresh store and miss our isolated kernel).
    // I-2: register the fabricated "rt" space id so `PartitionedSink::
    // route()` doesn't refuse this suite's Space-scoped `collection.*` events.
    ship_substrate::testutil::register_space(&ship_dir, "rt").unwrap();
    let space_db = ship_substrate::db::space_db_path(&ship_dir, "rt");
    let svc = ship_substrate::data::DataService::open_at(&space_db).unwrap();
    let store = std::sync::Arc::new(ship_substrate::SingleBackendDataStore::new(svc, "rt"));
    let ctx = SpaceContext::new(SpaceId::new("rt"), store, kernel.clone());
    (kernel, Arc::new(ctx))
}

fn round_trip_helper_schema() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "id": COLL,
        "title": "Round-trip Notes",
        "system": false,
        "fields": {
            "title": { "type": "string", "required": true },
            "rank": { "type": "number" }
        }
    }))
    .unwrap()
}

/// Build an inline Kysely AST: `SELECT * FROM <COLL> WHERE rank <= 50
/// ORDER BY rank ASC`. Mirrors the AST shape `behavior.rs::query_inline_*`
/// uses (Kysely SelectQueryNode + BinaryOperationNode + OrderByNode).
fn build_inline_query_ast() -> Vec<u8> {
    let ast = json!({
        "kind": "SelectQueryNode",
        "from": {
            "kind": "FromNode",
            "froms": [{
                "kind": "TableNode",
                "table": {
                    "kind": "SchemableIdentifierNode",
                    "identifier": { "kind": "IdentifierNode", "name": COLL }
                }
            }]
        },
        "where": {
            "kind": "WhereNode",
            "where": {
                "kind": "BinaryOperationNode",
                "op": { "kind": "OperatorNode", "operator": "<=" },
                "leftOperand": {
                    "kind": "ReferenceNode",
                    "column": {
                        "kind": "ColumnNode",
                        "column": { "kind": "IdentifierNode", "name": "rank" }
                    }
                },
                "rightOperand": {
                    "kind": "ValueNode",
                    "value": {
                        "kind": "PrimitiveValueNode",
                        "value": 50,
                        "type": "number"
                    }
                }
            }
        },
        "orderBy": {
            "kind": "OrderByNode",
            "items": [{
                "orderBy": {
                    "kind": "OrderByItemNode",
                    "column": {
                        "kind": "ColumnNode",
                        "column": { "kind": "IdentifierNode", "name": "rank" }
                    }
                },
                "direction": "asc"
            }]
        }
    });
    serde_json::to_vec(&ast).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn round_trip_no_daemon_no_ports() {
    let tmp = tempfile::tempdir().unwrap();
    let (kernel, ctx) = make_kernel_and_ctx(&tmp);

    let svc = CollectionService {
        ctx: ctx.clone(),
        schemas: Arc::new(SchemaRegistry::new()),
    };

    // ── create_schema (2 fields) ───────────────────────────────────────────
    svc.create_schema(COLL, &round_trip_helper_schema())
        .await
        .expect("create_schema");

    // ── create_item + get_item round-trip ──────────────────────────────────
    let first = svc
        .create_item(
            COLL,
            &serde_json::to_vec(&json!({ "title": "alpha", "rank": 10 })).unwrap(),
        )
        .await
        .expect("create_item alpha");
    svc.create_item(
        COLL,
        &serde_json::to_vec(&json!({ "title": "beta", "rank": 80 })).unwrap(),
    )
    .await
    .expect("create_item beta");
    let third = svc
        .create_item(
            COLL,
            &serde_json::to_vec(&json!({ "title": "gamma", "rank": 40 })).unwrap(),
        )
        .await
        .expect("create_item gamma");

    let alpha = svc.get_item(COLL, &first.id).await.expect("get_item alpha");
    assert_eq!(
        alpha.data.get("title").and_then(|v| v.as_str()),
        Some("alpha")
    );
    // `rank` is stored as REAL via the projection. Accept any numeric form
    // (integer, float, or text-encoded) so this test stays robust if the
    // column type changes.
    let rank = alpha
        .data
        .get("rank")
        .and_then(|v| match v {
            serde_json::Value::Number(n) => n.as_f64().map(|f| f as i64),
            serde_json::Value::String(s) => s.parse::<i64>().ok(),
            _ => None,
        })
        .unwrap_or(-1);
    assert_eq!(
        rank,
        10,
        "alpha rank should be 10, got {:?}",
        alpha.data.get("rank")
    );

    // ── query_items_ast (WHERE rank <= 50 ORDER BY rank ASC) ───────────────
    let ast = build_inline_query_ast();
    let (items, _cursor) = svc
        .query_items_ast(&ast, None, 32)
        .await
        .expect("query_items_ast");

    // Only alpha (rank 10) and gamma (rank 40) qualify; beta (rank 80) excluded.
    assert_eq!(
        items.len(),
        2,
        "expected 2 items with rank <= 50, got {}: {:?}",
        items.len(),
        items
    );
    assert_eq!(
        items[0].id, first.id,
        "ORDER BY rank ASC should put rank 10 first"
    );
    assert_eq!(
        items[1].id, third.id,
        "ORDER BY rank ASC should put rank 40 second"
    );

    // ── collection-scope Watch: first frame is a `set` snapshot, then a patch
    let ctx_for_snapshot = ctx.clone();
    let spec = ResolverSpec {
        prefilter: Box::new(|env| env.event_type.starts_with("collection.")),
        snapshot: Box::new(move || {
            let c = ctx_for_snapshot.clone();
            Box::pin(async move {
                let q = ship_substrate::ports::Query::new(format!("collection_{COLL}"));
                let rows = c
                    .data()
                    .query(q)
                    .await
                    .ok()
                    .map(|r| r.rows)
                    .unwrap_or_default();
                let items: Vec<serde_json::Value> = rows
                    .into_iter()
                    .map(|r| {
                        // `Row` exposes `fields` (the per-column serde_json map).
                        let title = r
                            .fields
                            .get("title")
                            .cloned()
                            .unwrap_or(serde_json::Value::Null);
                        json!({ "id": r.id, "title": title })
                    })
                    .collect();
                json!({ "notes": { "items": items } })
            })
        }),
    };

    let registry = ReactiveRegistry::new(kernel.clone());
    let key = WatchKey {
        space_id: "rt".into(),
        scope: format!("collection:{COLL}"),
        query_hash: 0,
    };
    let mut rx = registry.watch(key, EventFilter::default(), spec);

    // First frame: snapshot set at root.
    let frame1 = tokio::time::timeout(Duration::from_secs(3), rx.recv())
        .await
        .expect("snapshot frame timeout")
        .expect("snapshot frame closed");
    assert_eq!(
        frame1.patches.len(),
        1,
        "snapshot frame must be exactly one patch"
    );
    assert_eq!(
        frame1.patches[0].op,
        PatchOp::Set,
        "snapshot frame must be Set, not Patch"
    );
    assert_eq!(frame1.patches[0].path, "", "snapshot frame must be at root");

    // Trigger a patch by adding a new item.
    svc.create_item(
        COLL,
        &serde_json::to_vec(&json!({ "title": "delta", "rank": 20 })).unwrap(),
    )
    .await
    .expect("create_item delta");

    // Second frame: a non-empty patch frame (the resolver diffed the snapshot).
    let frame2 = tokio::time::timeout(Duration::from_secs(3), rx.recv())
        .await
        .expect("patch frame timeout")
        .expect("patch frame closed");
    assert!(
        !frame2.patches.is_empty(),
        "expected non-empty patch frame after create_item"
    );

    // Sanity: three items qualify after delta (rank 20) joins.
    let (items_after, _) = svc
        .query_items_ast(&ast, None, 32)
        .await
        .expect("query_items_ast post-watch");
    assert_eq!(
        items_after.len(),
        3,
        "expected 3 items with rank <= 50 after delta insert"
    );
}
