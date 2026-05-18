use axum::Extension;
use axum::extract::Path;
use axum::http::header;

use crate::DaemonMode;

fn dev() -> Option<Extension<DaemonMode>> { Some(Extension(DaemonMode::Development)) }
fn prod() -> Option<Extension<DaemonMode>> { Some(Extension(DaemonMode::Production)) }

// ── Asset handler cache headers ───────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn asset_dev_mode_no_cache() {
    let resp = crate::assets::serve_sui_runtime(dev()).await;
    let cc = resp.headers().get(header::CACHE_CONTROL).unwrap().to_str().unwrap();
    assert_eq!(cc, "no-cache");
}

#[tokio::test(flavor = "multi_thread")]
async fn asset_prod_mode_immutable() {
    let resp = crate::assets::serve_sui_runtime(prod()).await;
    let cc = resp.headers().get(header::CACHE_CONTROL).unwrap().to_str().unwrap();
    assert!(cc.contains("immutable"), "prod asset must be immutable; got: {cc}");
    assert!(cc.contains("max-age=31536000"), "prod asset must have max-age; got: {cc}");
}

#[tokio::test(flavor = "multi_thread")]
async fn asset_prod_mode_has_etag() {
    let resp = crate::assets::serve_sui_runtime(prod()).await;
    assert!(resp.headers().contains_key(header::ETAG), "prod asset must have ETag");
}

#[tokio::test(flavor = "multi_thread")]
async fn asset_dev_mode_no_etag() {
    let resp = crate::assets::serve_sui_runtime(dev()).await;
    assert!(!resp.headers().contains_key(header::ETAG), "dev asset must not have ETag");
}

#[tokio::test(flavor = "multi_thread")]
async fn island_prod_mode_immutable() {
    let resp = crate::assets::serve_sui_island(prod(), Path("sui-terminal".to_string())).await;
    let cc = resp.headers().get(header::CACHE_CONTROL).unwrap().to_str().unwrap();
    assert!(cc.contains("immutable"), "prod island must be immutable; got: {cc}");
}

#[tokio::test(flavor = "multi_thread")]
async fn island_dev_mode_no_cache() {
    let resp = crate::assets::serve_sui_island(dev(), Path("sui-terminal".to_string())).await;
    let cc = resp.headers().get(header::CACHE_CONTROL).unwrap().to_str().unwrap();
    assert_eq!(cc, "no-cache");
}

// ── View template/setup cache headers ────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
#[allow(
    clippy::await_holding_lock,
    reason = "test serializes env-var mutation across the await — the whole point of this lock"
)]
async fn view_template_prod_must_revalidate_and_etag() {
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
    let _lock = crate::test_lock::lock();
    let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
    unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", repo_root()); }
    let resp = crate::view::serve_sui_view_template(prod(), Path("agents".to_string())).await;
    match prev {
        Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
        None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
    }
    let cc = resp.headers().get(header::CACHE_CONTROL).unwrap().to_str().unwrap();
    assert!(cc.contains("must-revalidate"), "prod template must have must-revalidate; got: {cc}");
    assert!(resp.headers().contains_key(header::ETAG), "prod template must have ETag");
}

#[tokio::test(flavor = "multi_thread")]
#[allow(
    clippy::await_holding_lock,
    reason = "test serializes env-var mutation across the await — the whole point of this lock"
)]
async fn view_template_dev_no_cache_no_etag() {
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
    let _lock = crate::test_lock::lock();
    let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
    unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", repo_root()); }
    let resp = crate::view::serve_sui_view_template(dev(), Path("agents".to_string())).await;
    match prev {
        Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
        None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
    }
    let cc = resp.headers().get(header::CACHE_CONTROL).unwrap().to_str().unwrap();
    assert_eq!(cc, "no-cache");
    assert!(!resp.headers().contains_key(header::ETAG));
}
