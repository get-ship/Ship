//! Integration test for `SpaceService.{InstallStack, UnregisterStack,
//! ListInstalled}` over the live wire.
//!
//! Mirrors `compile.rs`: in-process ConnectRPC router + mocked
//! `ship_store::Registry` so we exercise the full wire without standing up a
//! gateway server.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use parking_lot::Mutex;
use ship_ops_rpc::SpaceServiceExt;
use ship_space::{LibraryProjection, SpaceStacksProjection, SpaceWire, SpacesProjection};
use ship_store::{FetchedBundle, Registry, RegistryVersion, TrustedKey};
use stack_model::ArchiveHeader;
use tower::ServiceExt;

const TEST_KID: &str = "test-publisher-1";
const TEST_PRIV_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
    MC4CAQAwBQYDK2VwBCIEIAFISiAspm6fXMD6Gg3dlW8qtiRFxoiHbiCjCuys/rg5\n\
    -----END PRIVATE KEY-----\n";
const TEST_PUB_X: &str = "mZAmshQk4uLsAab8Rhqqt9kJGbP9F6hZZ39eyfUncAQ";

fn trusted() -> Vec<TrustedKey> {
    vec![TrustedKey {
        kid: TEST_KID.to_string(),
        x_base64url: TEST_PUB_X.to_string(),
    }]
}

/// Install the domain's projection set on the substrate projection bus.
/// Production `helmd`/`shipd` does this at startup; tests do it inline.
fn install_projections() {
    ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
        vec![
            Box::new(SpacesProjection),
            Box::new(SpaceStacksProjection),
            Box::new(LibraryProjection),
        ]
    }));
}

/// Mock that returns the CANONICAL tree hash from `write_signed_bundle` —
/// the install pipeline's belt-and-braces check requires the registry's
/// stated hash to match the extracted tree hash.
struct MockRegistry {
    state: Arc<Mutex<Option<(String, String, Vec<u8>, String, String)>>>,
}

#[async_trait]
impl Registry for MockRegistry {
    async fn resolve_version(
        &self,
        path: &str,
        _constraint: &str,
    ) -> anyhow::Result<RegistryVersion> {
        let g = self.state.lock();
        let (p, version, _, _, sha) = g.as_ref().expect("fixture set");
        assert_eq!(p, path);
        Ok(RegistryVersion {
            version: version.clone(),
            sha256: sha.clone(),
        })
    }

    async fn fetch_bundle(&self, path: &str, version: &str) -> anyhow::Result<FetchedBundle> {
        let g = self.state.lock();
        let (p, v, bytes, jwt, sha) = g.as_ref().expect("fixture set").clone();
        assert_eq!(p, path);
        assert_eq!(v, version);
        Ok(FetchedBundle {
            bytes,
            attestation_jwt: jwt,
            sha256: sha,
        })
    }
}

fn write_signed_bundle() -> (Vec<u8>, String, String) {
    let src = tempfile::tempdir().unwrap();
    std::fs::write(src.path().join("stack.config.ts"), "export default {};\n").unwrap();
    std::fs::create_dir_all(src.path().join("skills/review")).unwrap();
    std::fs::write(src.path().join("skills/review/SKILL.md"), "# Review\n").unwrap();
    let header = ArchiveHeader {
        path: "github.com/ship/test-stack".into(),
        version: "1.2.3".into(),
        commit: "0123456789abcdef0123456789abcdef01234567".into(),
        hash: String::new(),
        dependencies: Default::default(),
        recommended_agents: vec![],
        recommended_skills: vec![],
    };

    let (tar_bytes, header) =
        ship_store::pack_stack(src.path(), header).unwrap();
    let tree_hash = header.hash.clone();

    let mut jwt_header = Header::new(Algorithm::EdDSA);
    jwt_header.kid = Some(TEST_KID.to_string());
    let claim = ship_store::BundleClaim {
        path: "github.com/ship/test-stack".into(),
        version: "1.2.3".into(),
        sha256: tree_hash.clone(),
        kid: Some(TEST_KID.to_string()),
        iat: Some(1_700_000_000),
    };
    let key = EncodingKey::from_ed_pem(TEST_PRIV_PEM.as_bytes()).unwrap();
    let jwt = jsonwebtoken::encode(&jwt_header, &claim, &key).unwrap();

    (tar_bytes, jwt, tree_hash)
}

async fn call_rpc(
    app: &axum::Router,
    method: &str,
    body: serde_json::Value,
) -> (u16, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri(format!("/ship.ops.v1.SpaceService/{method}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.expect("in-process RPC");
    let status = resp.status().as_u16();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("response not valid JSON: {e}; bytes={:?}", bytes));
    (status, parsed)
}

/// Build a router with a fixed `SpaceContext::for_space` factory — the wire
/// only ever sees the seeded active space. This mirrors the production
/// helmd mount but with install handles attached.
fn build_install_router(
    kernel: Arc<ship_substrate::ports::KernelLog>,
    space_id: String,
    data_dir: PathBuf,
    registry: Arc<dyn Registry + Send + Sync>,
) -> axum::Router {
    let connect = connectrpc::Router::new();
    let data_dir_for_wire = data_dir.clone();
    let space_id_for_wire = space_id;
    let wire = SpaceWire::with_install(
        kernel,
        move |_rctx: &connectrpc::RequestContext| {
            ship_substrate::ports::SpaceContext::for_space(
                &data_dir_for_wire,
                ship_substrate::ports::SpaceId::new(space_id_for_wire.clone()),
            )
            .map(|ctx| Arc::new(ctx))
            .map_err(|e| connectrpc::ConnectError::internal(e.to_string()))
        },
        registry,
        trusted(),
        data_dir,
    );
    let connect = Arc::new(wire).register(connect);
    axum::Router::new().fallback_service(connect.into_axum_service())
}

/// The helmd wire-level install RPC: the daemon resolves, fetches, verifies,
/// unpacks into the managed library dir under `<data>/library/<enc-path>/<enc-ver>/`,
/// and returns the resolved install metadata in the proto response. The
/// projection-fold into the `library` SQL table is exercised in
/// ship-space/tests/install_lifecycle.rs (separate file because it talks to
/// the projection bus directly).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn install_stack_wire_round_trip() {
    install_projections();
    let ship_home = tempfile::tempdir().unwrap();
    let _guard = ship_substrate::testutil::EnvGuard::set(
        ship_substrate::env::Var::Home,
        ship_home.path().to_str().unwrap(),
    );
    let data_dir = ship_substrate::ShipHome::from_env().data_dir().to_path_buf();
    let _ = ship_substrate::db::open_runtime_db(&data_dir);

    // Seed an active space so resolve_active_space succeeds.
    let runtime = ship_substrate::events::new_isolated_for_test(data_dir.clone()).unwrap();
    let kernel = Arc::new(ship_substrate::ports::KernelLog::new(runtime));
    let repo_tmp = tempfile::tempdir().unwrap();
    let repo_path = repo_tmp.path().to_path_buf();
    std::fs::write(repo_path.join("stack.config.ts"), "export default {};\n").unwrap();
    let canonical = std::fs::canonicalize(&repo_path).unwrap();
    let space_record = ship_space::identity::ensure_identity_for_root(&kernel, &canonical, None)
        .await
        .expect("seed space");
    let space_id = space_record.space_id.clone();

    let (bytes, jwt, tree_hash) = write_signed_bundle();
    let registry = MockRegistry {
        state: Arc::new(Mutex::new(Some((
            "github.com/ship/test-stack".into(),
            "1.2.3".into(),
            bytes,
            jwt,
            tree_hash.clone(),
        )))),
    };

    let app = build_install_router(
        kernel,
        space_id,
        data_dir.clone(),
        Arc::new(registry) as Arc<dyn Registry + Send + Sync>,
    );

    // install_stack → asserts on status + body fields.
    let install_body = serde_json::json!({
        "space_id": "",
        "path": "github.com/ship/test-stack",
        "version_constraint": "^1.0",
    });
    let (status, body) = call_rpc(&app, "InstallStack", install_body).await;
    assert_eq!(status, 200, "install must succeed; body={body}");
    let root_path = body["rootPath"].as_str().expect("rootPath string");
    assert_eq!(body["version"].as_str(), Some("1.2.3"));
    assert_eq!(body["contentHash"].as_str(), Some(tree_hash.as_str()));
    assert!(
        root_path.ends_with("github.com%2Fship%2Ftest-stack/1.2.3"),
        "library root ends with url-encoded path/version: {root_path}"
    );

    // list_installed → endpoint reachable; projection-fold assertion is in
    // the ship-space integration test. (The empty-vec serialisation rule
    // means an empty library serialises as `{}`, so we accept both shapes.)
    let list_body = serde_json::json!({ "space_id": "" });
    let (status, body) = call_rpc(&app, "ListInstalled", list_body).await;
    assert_eq!(status, 200, "list_installed must succeed; status={status} body={body}");
    // Either `{"stacks": [...]}` (when rows exist) or `{}` (empty vec
    // serialised) — both are valid for the wire contract.
    let _ = body;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn install_stack_rejects_empty_path() {
    install_projections();
    let ship_home = tempfile::tempdir().unwrap();
    let _guard = ship_substrate::testutil::EnvGuard::set(
        ship_substrate::env::Var::Home,
        ship_home.path().to_str().unwrap(),
    );
    let data_dir = ship_substrate::ShipHome::from_env().data_dir().to_path_buf();
    let _ = ship_substrate::db::open_runtime_db(&data_dir);

    let runtime = ship_substrate::events::new_isolated_for_test(data_dir.clone()).unwrap();
    let kernel = Arc::new(ship_substrate::ports::KernelLog::new(runtime));
    let repo_tmp = tempfile::tempdir().unwrap();
    let repo_path = repo_tmp.path().to_path_buf();
    std::fs::write(repo_path.join("stack.config.ts"), "export default {};\n").unwrap();
    let canonical = std::fs::canonicalize(&repo_path).unwrap();
    let space_record = ship_space::identity::ensure_identity_for_root(&kernel, &canonical, None)
        .await
        .expect("seed space");
    let space_id = space_record.space_id.clone();

    let (bytes, jwt, tree_hash) = write_signed_bundle();
    let registry = MockRegistry {
        state: Arc::new(Mutex::new(Some((
            "github.com/ship/test-stack".into(),
            "1.2.3".into(),
            bytes,
            jwt,
            tree_hash,
        )))),
    };

    let app = build_install_router(
        kernel,
        space_id,
        data_dir,
        Arc::new(registry) as Arc<dyn Registry + Send + Sync>,
    );

    let install_body = serde_json::json!({
        "space_id": "",
        "path": "",
        "version_constraint": "^1.0",
    });
    let (status, _body) = call_rpc(&app, "InstallStack", install_body).await;
    assert_eq!(status, 400, "empty path rejected as bad args");
}
