//! Install-stack round-trip — mocked registry + bundled attestation, exercising
//! the resolve→fetch→extract-verify→unpack→library-move pipeline without
//! standing up a Connect-RPC server.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use parking_lot::Mutex;
use ship_store::{
    BundleFetcher, FetchedBundle, InstallError, InstalledStack, Registry, RegistryVersion,
    TrustedKey, install_stack, remove_installed,
};
use stack_model::ArchiveHeader;
use stack_model::manifest::ManifestExports;

const TEST_KID: &str = "test-publisher-1";
// Offline-generated Ed25519 keypair (test-only). The private half signs the
// attestation JWT; the public half populates `TrustedKey`.
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

/// In-memory registry — both methods return the canned `version` + the
/// canonical tree hash the install pipeline will recompute, so the
/// registry/attestation/extracted-tree hashes all agree.
struct MockRegistry {
    path: String,
    version: String,
    hash: String,
    bytes: Vec<u8>,
    jwt: String,
    fetch_count: Arc<Mutex<u32>>,
}

#[async_trait]
impl Registry for MockRegistry {
    async fn resolve_version(
        &self,
        path: &str,
        _constraint: &str,
    ) -> anyhow::Result<RegistryVersion> {
        assert_eq!(path, self.path);
        Ok(RegistryVersion {
            version: self.version.clone(),
            sha256: self.hash.clone(),
        })
    }

    async fn fetch_bundle(&self, path: &str, version: &str) -> anyhow::Result<FetchedBundle> {
        assert_eq!(path, self.path);
        assert_eq!(version, self.version);
        *self.fetch_count.lock() += 1;
        Ok(FetchedBundle {
            bytes: self.bytes.clone(),
            attestation_jwt: self.jwt.clone(),
            sha256: self.hash.clone(),
        })
    }
}

fn write_stack_tree(root: &std::path::Path) {
    // Files that mirror a real stack: a stack marker + a skill. The header
    // is written separately below — pack_stack requires a `ship.json`
    // carrying the publisher-stated version/commit; the install pipeline
    // re-hashes the extracted tree and verifies the attestation binds that
    // hash (not the header's `hash`, which is overwritten by pack_stack).
    std::fs::write(root.join("stack.config.ts"), "export default {};\n").unwrap();
    std::fs::create_dir_all(root.join("skills/review")).unwrap();
    std::fs::write(root.join("skills/review/SKILL.md"), "# Review skill\n").unwrap();

    let header = ArchiveHeader {
        path: "github.com/ship/test-stack".into(),
        version: "1.2.3".into(),
        commit: "0123456789abcdef0123456789abcdef01234567".into(),
        // pack_stack overwrites this with the canonical tree hash; the
        // publisher writes a placeholder and lets the pack step set it.
        hash: String::new(),
        exports: ManifestExports::default(),
        dependencies: Default::default(),
        recommended_agents: vec!["reviewer".into()],
        recommended_skills: vec![],
    };
    std::fs::write(
        root.join("ship.json"),
        serde_json::to_vec_pretty(&header).unwrap(),
    )
    .unwrap();
}

/// Build a gzip-tarball containing a valid stack tree + a publisher-signed
/// attestation JWT over the tree hash. Returns `(bytes, attestation_jwt,
/// canonical_hash)`.
fn build_signed_bundle() -> (Vec<u8>, String, String) {
    let src = tempfile::tempdir().unwrap();
    write_stack_tree(src.path());

    let (tar_bytes, header) = ship_store::pack_stack(src.path()).unwrap();
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

fn fresh_registry(bytes: Vec<u8>, jwt: String, hash: String) -> MockRegistry {
    MockRegistry {
        path: "github.com/ship/test-stack".into(),
        version: "1.2.3".into(),
        hash,
        bytes,
        jwt,
        fetch_count: Arc::new(Mutex::new(0)),
    }
}

#[tokio::test]
async fn install_stack_unpacks_into_managed_library_dir() {
    let (bytes, jwt, hash) = build_signed_bundle();
    let registry = fresh_registry(bytes, jwt, hash.clone());

    let tmp = tempfile::tempdir().unwrap();
    let data_dir: PathBuf = tmp.path().to_path_buf();

    let result: InstalledStack = install_stack(
        &registry,
        &data_dir,
        "github.com/ship/test-stack",
        "^1.0",
        &trusted(),
    )
    .await
    .expect("install succeeds");

    // The library row landed at <data>/library/<encoded-path>/<version>/.
    let expected_root = ship_store::library_root(
        &data_dir,
        "github.com/ship/test-stack",
        "1.2.3",
    );
    assert_eq!(result.root_path, expected_root);
    assert!(result.root_path.is_dir());
    assert!(result.root_path.join("stack.config.ts").is_file());
    assert!(result.root_path.join("skills/review/SKILL.md").is_file());
    assert_eq!(result.content_hash, hash);
    assert_eq!(result.version, "1.2.3");
    assert_eq!(*registry.fetch_count.lock(), 1, "fetch called exactly once");
}

#[tokio::test]
async fn install_stack_is_idempotent_when_library_row_already_matches_hash() {
    let (bytes, jwt, hash) = build_signed_bundle();
    let registry = fresh_registry(bytes, jwt, hash);

    let tmp = tempfile::tempdir().unwrap();
    let data_dir: PathBuf = tmp.path().to_path_buf();

    let first = install_stack(
        &registry,
        &data_dir,
        "github.com/ship/test-stack",
        "^1.0",
        &trusted(),
    )
    .await
    .expect("first install succeeds");

    // Second call: the library row is already at the canonical hash — the
    // pipeline must short-circuit. `fetch_bundle` MUST NOT be called.
    let second = install_stack(
        &registry,
        &data_dir,
        "github.com/ship/test-stack",
        "^1.0",
        &trusted(),
    )
    .await
    .expect("second install succeeds (idempotent)");
    assert_eq!(first.root_path, second.root_path);
    assert_eq!(first.content_hash, second.content_hash);
    assert_eq!(
        *registry.fetch_count.lock(),
        1,
        "fetch NOT called on idempotent re-install"
    );
}

#[tokio::test]
async fn install_stack_rejects_empty_path() {
    let (bytes, jwt, hash) = build_signed_bundle();
    let registry = fresh_registry(bytes, jwt, hash);

    let tmp = tempfile::tempdir().unwrap();
    let err = install_stack(&registry, tmp.path(), "", "^1.0", &trusted())
        .await
        .expect_err("empty path rejected");
    assert!(matches!(err, InstallError::BadArgs(_)));
}

#[tokio::test]
async fn install_stack_rejects_bad_attestation() {
    let (bytes, _jwt, hash) = build_signed_bundle();

    // Forge a JWT that claims a DIFFERENT tree hash → verify fails.
    let mut jwt_header = Header::new(Algorithm::EdDSA);
    jwt_header.kid = Some(TEST_KID.to_string());
    let claim = ship_store::BundleClaim {
        path: "github.com/ship/test-stack".into(),
        version: "1.2.3".into(),
        sha256: "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
        kid: Some(TEST_KID.to_string()),
        iat: Some(1_700_000_000),
    };
    let key = EncodingKey::from_ed_pem(TEST_PRIV_PEM.as_bytes()).unwrap();
    let bad_jwt = jsonwebtoken::encode(&jwt_header, &claim, &key).unwrap();

    let registry = fresh_registry(bytes, bad_jwt, hash);
    let tmp = tempfile::tempdir().unwrap();
    let err = install_stack(
        &registry,
        tmp.path(),
        "github.com/ship/test-stack",
        "^1.0",
        &trusted(),
    )
    .await
    .expect_err("tampered bundle rejected");
    assert!(
        matches!(err, InstallError::Verify(_)),
        "expected verify error, got {err:?}"
    );
}

#[tokio::test]
async fn remove_installed_clears_managed_library_row() {
    let (bytes, jwt, hash) = build_signed_bundle();
    let registry = fresh_registry(bytes, jwt, hash);

    let tmp = tempfile::tempdir().unwrap();
    let data_dir: PathBuf = tmp.path().to_path_buf();

    let installed = install_stack(
        &registry,
        &data_dir,
        "github.com/ship/test-stack",
        "^1.0",
        &trusted(),
    )
    .await
    .expect("install ok");
    assert!(installed.root_path.is_dir());

    remove_installed(&data_dir, "github.com/ship/test-stack", "1.2.3").unwrap();
    assert!(!installed.root_path.exists());

    // Idempotent: a second remove is a no-op.
    remove_installed(&data_dir, "github.com/ship/test-stack", "1.2.3").unwrap();
}
