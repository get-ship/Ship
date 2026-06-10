use crate::DaemonMode;

// ── Asset handler cache headers ───────────────────────────────────────────────

#[test]
fn asset_dev_mode_no_cache() {
    let resp = crate::assets::serve_sui_runtime(DaemonMode::Development);
    assert_eq!(resp.cache_control, "no-cache");
}

#[test]
fn asset_prod_mode_immutable() {
    let resp = crate::assets::serve_sui_runtime(DaemonMode::Production);
    assert!(resp.cache_control.contains("immutable"), "prod asset must be immutable; got: {}", resp.cache_control);
    assert!(resp.cache_control.contains("max-age=31536000"), "prod asset must have max-age; got: {}", resp.cache_control);
}

#[test]
fn asset_prod_mode_has_etag() {
    let resp = crate::assets::serve_sui_runtime(DaemonMode::Production);
    assert!(resp.etag.is_some(), "prod asset must have ETag");
}

#[test]
fn asset_dev_mode_no_etag() {
    let resp = crate::assets::serve_sui_runtime(DaemonMode::Development);
    assert!(resp.etag.is_none(), "dev asset must not have ETag");
}

// ── Shared chunk serving (#267 per-view resolver, multi-entry build) ──────────

#[test]
fn chunk_serves_from_deck_root_and_rejects_bad_input() {
    use crate::test_lock::ProjectRootGuard;
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(".deck")).unwrap();
    std::fs::write(tmp.path().join(".deck/shared-abc123.js"), "export const x=1\n").unwrap();
    let _root = ProjectRootGuard::new(tmp.path());
    let ok = crate::assets::serve_sui_chunk(DaemonMode::Production, "shared-abc123.js");
    let traversal = crate::assets::serve_sui_chunk(DaemonMode::Production, "../etc/passwd");
    let nested = crate::assets::serve_sui_chunk(DaemonMode::Production, "sub/x.js");
    let nonjs = crate::assets::serve_sui_chunk(DaemonMode::Production, "evil.txt");
    let missing = crate::assets::serve_sui_chunk(DaemonMode::Production, "nope.js");
    assert_eq!(ok.status, 200, "a real shared chunk serves");
    assert!(ok.cache_control.contains("immutable"), "content-addressed → immutable; got {}", ok.cache_control);
    assert_eq!(traversal.status, 404, "path traversal rejected");
    assert_eq!(nested.status, 404, "nested path rejected (single segment only)");
    assert_eq!(nonjs.status, 404, "non-.js rejected");
    assert_eq!(missing.status, 404, "missing chunk → 404");
}

