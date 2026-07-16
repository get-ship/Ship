//! Wire-layer behavior tests for `ArtifactsService`.
//!
//! These exercise the ConnectRPC wire methods (`create` / `get_blob`) directly
//! against an `ArtifactsWire` instance — the service layer is already covered
//! by `behavior.rs`. The wire layer's job is thin: enforce the byte cap, parse
//! the scope, and convert the typed error into a `ConnectError`.

use std::sync::Arc;

use connectrpc::RequestContext;
use ship_artifacts::wire::ArtifactsWire;
use ship_ops_rpc::ArtifactsService;
use ship_ops_rpc::{
    CreateArtifactRequest, GetArtifactBlobRequest, OwnedCreateArtifactRequestView,
    OwnedGetArtifactBlobRequestView,
};
use ship_substrate::data::DataService;
use ship_substrate::home::ShipHome;
use ship_substrate::{new_isolated_for_test, KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

mod common;

const SCOPE_SPACE: &str = "proj1";

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    common::install_artifact_projection();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

fn make_ctx(tmp: &tempfile::TempDir, kernel: Arc<KernelLog>) -> Arc<SpaceContext> {
    let ship_dir = tmp.path().join(".ship");
    // I-2: register the fabricated space id so `PartitionedSink::route()`
    // doesn't refuse this suite's Space-scoped `artifact.*` events.
    ship_substrate::testutil::register_space(&ship_dir, SCOPE_SPACE).unwrap();
    let space_db = ship_substrate::db::space_db_path(&ship_dir, SCOPE_SPACE);
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, SCOPE_SPACE));
    Arc::new(SpaceContext::new(SpaceId::new(SCOPE_SPACE), store, kernel))
}

fn make_wire(
    tmp: &tempfile::TempDir,
    ctx: Arc<SpaceContext>,
) -> ArtifactsWire<impl Fn(&RequestContext) -> Arc<SpaceContext> + Send + Sync + 'static> {
    let ctx_for_factory = ctx.clone();
    let kernel = ctx.kernel.clone();
    let make = move |_rctx: &RequestContext| -> Arc<SpaceContext> { ctx_for_factory.clone() };
    ArtifactsWire::new(kernel, tmp.path().to_path_buf(), make)
}

fn owned_create(
    name: &str,
    mime: &str,
    data: Vec<u8>,
    scope: &str,
    run_id: &str,
    expires_at: &str,
) -> OwnedCreateArtifactRequestView {
    let msg = CreateArtifactRequest {
        name: name.to_string(),
        mime: mime.to_string(),
        data,
        scope: scope.to_string(),
        run_id: run_id.to_string(),
        expires_at: expires_at.to_string(),
        ..Default::default()
    };
    OwnedCreateArtifactRequestView::from_owned(&msg).expect("encode view")
}

fn owned_get_blob(id: &str) -> OwnedGetArtifactBlobRequestView {
    let msg = GetArtifactBlobRequest { id: id.to_string(), ..Default::default() };
    OwnedGetArtifactBlobRequestView::from_owned(&msg).expect("encode view")
}

// ── 1. create round-trips through the wire layer ─────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wire_create_then_get_blob_round_trips_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let wire = make_wire(&tmp, ctx);

    let payload = b"hello, wire layer".to_vec();
    let req = owned_create("hi.txt", "text/plain", payload.clone(), "project", "", "");
    let resp = wire.create(RequestContext::default(), req).await.expect("create");
    let record = resp.body;
    assert_eq!(record.name, "hi.txt");
    assert_eq!(record.mime, "text/plain");
    assert_eq!(record.size as usize, payload.len());
    assert_eq!(record.scope, "project");

    let id = record.id.clone();
    let resp = wire.get_blob(RequestContext::default(), owned_get_blob(&id)).await.expect("get_blob");
    let got_record = resp.body.record.into_option().expect("blob response carries record");
    assert_eq!(got_record.id, id);
    // sha256 of `payload` should ride the record.
    let expected_hash = ship_types::sha256_hex(&payload);
    assert_eq!(got_record.content_hash.as_deref(), Some(expected_hash.as_str()));
}

// ── 2. create dedups metadata for identical bytes within a scope ─────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wire_create_dedups_metadata_for_identical_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let wire = make_wire(&tmp, ctx);

    let payload = b"same bytes".to_vec();
    let r1 = wire
        .create(
            RequestContext::default(),
            owned_create("a.txt", "text/plain", payload.clone(), "project", "", ""),
        )
        .await
        .expect("create 1")
        .body;
    let r2 = wire
        .create(
            RequestContext::default(),
            owned_create("a-renamed.txt", "text/plain", payload.clone(), "project", "", ""),
        )
        .await
        .expect("create 2")
        .body;
    // Two distinct metadata rows — but the underlying content-hash is identical.
    assert_ne!(r1.id, r2.id, "dedup emits a new metadata row, not a new id");
    assert_eq!(r1.content_hash, r2.content_hash, "same bytes → same hash");

    // The blob is shared: GetBlob for each id returns the same bytes.
    let b1 = wire
        .get_blob(RequestContext::default(), owned_get_blob(&r1.id))
        .await
        .expect("get 1")
        .body;
    let b2 = wire
        .get_blob(RequestContext::default(), owned_get_blob(&r2.id))
        .await
        .expect("get 2")
        .body;
    assert_eq!(b1.data, payload);
    assert_eq!(b2.data, payload);
}

// ── 3. create rejects payloads over the cap with a typed invalid-argument ────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wire_create_rejects_payload_over_8_mib() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let wire = make_wire(&tmp, ctx);

    // 8 MiB + 1 byte — the cap is on `>` not `>=`, so this must reject.
    let oversize = vec![0u8; ship_artifacts::error::MAX_PAYLOAD_BYTES + 1];
    let req = owned_create("big.bin", "application/octet-stream", oversize, "project", "", "");
    let err = wire
        .create(RequestContext::default(), req)
        .await
        .expect_err("over-cap create must error");
    // The wire layer surfaces the typed cap message via `ConnectError::invalid_argument`.
    let msg = format!("{err:?}");
    assert!(msg.contains("exceeds cap"), "expected cap message, got: {msg}");

    // The boundary: a payload exactly at the cap is allowed.
    let at_cap = vec![0u8; ship_artifacts::error::MAX_PAYLOAD_BYTES];
    let req = owned_create("edge.bin", "application/octet-stream", at_cap, "project", "", "");
    let r = wire
        .create(RequestContext::default(), req)
        .await
        .expect("at-cap create succeeds");
    assert_eq!(r.body.size as usize, ship_artifacts::error::MAX_PAYLOAD_BYTES);
}

// ── 4. create with run-scope carries expires_at through to the record ─────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wire_create_run_scope_carries_expires_at() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let wire = make_wire(&tmp, ctx);

    let exp = "2030-01-01T00:00:00Z";
    let req = owned_create("ephemeral.txt", "text/plain", b"gone soon".to_vec(), "run", "run-42", exp);
    let record = wire.create(RequestContext::default(), req).await.expect("create run").body;
    assert_eq!(record.scope, "run");
    assert_eq!(record.run_id.as_deref(), Some("run-42"));
    assert_eq!(record.expires_at.as_deref(), Some(exp));
}

// ── 5. create rejects an unknown scope as a typed invalid-argument ────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wire_create_rejects_unknown_scope() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let wire = make_wire(&tmp, ctx);

    let req = owned_create("x.txt", "text/plain", b"x".to_vec(), "account", "", "");
    let err = wire
        .create(RequestContext::default(), req)
        .await
        .expect_err("unknown scope must error");
    let msg = format!("{err:?}");
    assert!(msg.contains("target_scope"), "scope parse error mentions target_scope: {msg}");
}

// ── 6. get_blob on a missing id surfaces as a typed not-found ────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wire_get_blob_missing_id_is_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let wire = make_wire(&tmp, ctx);

    let err = wire
        .get_blob(RequestContext::default(), owned_get_blob("does-not-exist"))
        .await
        .expect_err("missing id must error");
    // `not_found` is the typed code — the message is human-meaningful.
    let msg = format!("{err:?}");
    assert!(msg.contains("not found") || msg.contains("not_found"), "got: {msg}");
}
