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

// `sui-terminal` is a real deck-built island: its `.sui` carries
// `<bundle separate />` and `packages/components/sui/terminal/src` is a
// SEPARATE_BUNDLE_ROOT, so `cargo xtask build-assets` writes
// `.deck/sui-components/sui-terminal.js`. These tests resolve the project root
// to the repo (same as the view-template tests below) so `serve_sui_island`
// reads that built bundle — cache-header coverage on a tag deck actually
// produces, never an orphan stale artifact.
#[test]
fn island_prod_mode_immutable() {
    let _lock = crate::test_lock::lock();
    let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
    unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", repo_root()); }
    let resp = crate::assets::serve_sui_island(DaemonMode::Production, "sui-terminal");
    match prev {
        Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
        None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
    }
    assert_eq!(resp.status, 200, "deck-built sui-terminal island must be served (run `cargo xtask build-assets`)");
    assert!(resp.cache_control.contains("immutable"), "prod island must be immutable; got: {}", resp.cache_control);
}

#[test]
fn island_dev_mode_no_cache() {
    let _lock = crate::test_lock::lock();
    let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
    unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", repo_root()); }
    let resp = crate::assets::serve_sui_island(DaemonMode::Development, "sui-terminal");
    match prev {
        Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
        None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
    }
    assert_eq!(resp.status, 200, "deck-built sui-terminal island must be served (run `cargo xtask build-assets`)");
    assert_eq!(resp.cache_control, "no-cache");
}

// ── View template/setup cache headers ────────────────────────────────────────

use std::path::PathBuf;
use std::sync::OnceLock;

fn repo_root() -> &'static std::path::Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let mut cur = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        loop {
            if cur.join("Cargo.toml").is_file()
                && std::fs::read_to_string(cur.join("Cargo.toml"))
                    .map(|s| s.contains("[workspace]"))
                    .unwrap_or(false)
            {
                return cur;
            }
            if !cur.pop() { panic!("no workspace root"); }
        }
    });
    ROOT.get().unwrap()
}

#[test]
fn view_template_prod_must_revalidate_and_etag() {
    let _lock = crate::test_lock::lock();
    let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
    unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", repo_root()); }
    let resp = crate::view::serve_sui_view_template(DaemonMode::Production, "git");
    match prev {
        Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
        None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
    }
    assert!(resp.cache_control.contains("must-revalidate"), "prod template must have must-revalidate; got: {}", resp.cache_control);
    assert!(resp.etag.is_some(), "prod template must have ETag");
}

#[test]
fn view_template_dev_no_cache_no_etag() {
    let _lock = crate::test_lock::lock();
    let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
    unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", repo_root()); }
    let resp = crate::view::serve_sui_view_template(DaemonMode::Development, "git");
    match prev {
        Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
        None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
    }
    assert_eq!(resp.cache_control, "no-cache");
    assert!(resp.etag.is_none());
}
