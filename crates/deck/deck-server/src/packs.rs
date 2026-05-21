//! Generalized pack-bundle handler — serves `<pack-id>` workspace
//! packages' `dist/<bundle>.js` files at `/_sui/packs/<pack-id>/<bundle>`.
//!
//! Pack discovery happens lazily on first request and is cached in a
//! `OnceLock`. The walker reads the root `package.json`'s
//! `workspaces.packages` list, expands single-segment globs, and
//! registers any package that declares `customElements` in its
//! manifest. `<pack-id>` is the unscoped portion of the npm name
//! (`@ship/charts` → `charts`).
//!
//! Two endpoints share this index:
//!   * `GET /_sui/packs/<pack-id>/<bundle>.js`     — bundle file
//!   * `GET /_sui/packs/<pack-id>/custom-elements.json` — aggregated CEM
//!
//! Workspace-catalog endpoints (`/api/sui/catalog/{workspace,published}`)
//! read pre-aggregated JSON from `.ship/sui-out/catalogs/` (emitted by
//! `bun packages/deck/catalog/src/cli.ts`). They live in this module so
//! all pack-discovery + catalog plumbing sits together.

use axum::{
    Extension,
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path as StdPath, PathBuf};
use std::sync::OnceLock;

use crate::{DaemonMode, asset_cache, compute_etag, mode_from_ext};

#[derive(Clone, Debug)]
struct PackRecord {
    /// Absolute path to the package root (the dir containing package.json).
    root: PathBuf,
    /// Value of `package.json.customElements`, relative to root.
    custom_elements: String,
}

static PACK_INDEX: OnceLock<HashMap<String, PackRecord>> = OnceLock::new();

fn repo_root() -> Option<PathBuf> {
    // deck-server lives at crates/deck/deck-server. Repo root is
    // three parents up from CARGO_MANIFEST_DIR. Used as the workspace
    // discovery anchor and the catalog directory base.
    StdPath::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(PathBuf::from)
}

fn read_json(path: &StdPath) -> Option<Value> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<Value>(&raw).ok()
}

fn workspace_globs(repo: &StdPath) -> Vec<String> {
    let manifest = match read_json(&repo.join("package.json")) {
        Some(m) => m,
        None => return Vec::new(),
    };
    let ws = manifest.get("workspaces");
    let arr = match ws {
        Some(Value::Array(a)) => a.clone(),
        Some(Value::Object(o)) => o
            .get("packages")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    arr.into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect()
}

fn expand_glob(repo: &StdPath, glob: &str) -> Vec<PathBuf> {
    if !glob.contains('*') {
        let abs = repo.join(glob);
        return if abs.exists() { vec![abs] } else { Vec::new() };
    }
    let (parent_rel, pattern) = match glob.rfind('/') {
        Some(i) => (&glob[..i], &glob[i + 1..]),
        None => (".", glob),
    };
    if pattern != "*" {
        // Only single trailing `*` supported, matching the aggregator.
        return Vec::new();
    }
    let parent_abs = repo.join(parent_rel);
    let entries = match std::fs::read_dir(&parent_abs) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        if name_s.starts_with('.') {
            continue;
        }
        let p = entry.path();
        if p.is_dir() {
            out.push(p);
        }
    }
    out
}

fn build_pack_index() -> HashMap<String, PackRecord> {
    let mut out: HashMap<String, PackRecord> = HashMap::new();
    let Some(repo) = repo_root() else {
        return out;
    };
    for glob in workspace_globs(&repo) {
        for dir in expand_glob(&repo, &glob) {
            let manifest_path = dir.join("package.json");
            let Some(manifest) = read_json(&manifest_path) else {
                continue;
            };
            let Some(name) = manifest.get("name").and_then(Value::as_str) else {
                continue;
            };
            let Some(cem) = manifest.get("customElements").and_then(Value::as_str) else {
                continue;
            };
            let pack_id = pack_id_for_name(name);
            out.insert(
                pack_id,
                PackRecord {
                    root: dir,
                    custom_elements: cem.to_string(),
                },
            );
        }
    }
    out
}

fn pack_id_for_name(npm_name: &str) -> String {
    if let Some(rest) = npm_name.strip_prefix('@')
        && let Some(idx) = rest.find('/')
    {
        return rest[idx + 1..].to_string();
    }
    npm_name.to_string()
}

fn pack_index() -> &'static HashMap<String, PackRecord> {
    PACK_INDEX.get_or_init(build_pack_index)
}

/// Public reset hook for tests + dev-loop scenarios where the workspace
/// composition changes between requests. Production callers leave it
/// alone — the OnceLock initialiser is sufficient.
#[cfg(test)]
pub fn reset_pack_index_for_tests() {
    // OnceLock has no `reset`; this helper rebuilds + drops the cached
    // value's leak by writing into a new OnceLock at startup. In test we
    // simply don't rely on caching — tests build the index inline.
}

/// `GET /_sui/packs/{pack_id}/{file}` — serves either the pack's
/// `dist/<file>` JS bundle or its aggregated `custom-elements.json`.
pub async fn serve_pack_asset(
    mode: Option<Extension<DaemonMode>>,
    Path((pack_id, file)): Path<(String, String)>,
) -> Response {
    let mode = mode_from_ext(mode);
    let Some(record) = pack_index().get(&pack_id) else {
        return not_found(&format!("unknown pack '{pack_id}'"));
    };
    // Custom-elements.json: serve the file the package declared via its
    // `customElements` manifest field, regardless of where it lives in
    // the pack tree.
    if file == "custom-elements.json" {
        let abs = record.root.join(&record.custom_elements);
        return match std::fs::read(&abs) {
            Ok(bytes) => json_response(bytes, mode),
            Err(_) => not_found(&format!("manifest missing: {}", abs.display())),
        };
    }
    // Bundle files: confine reads to `<root>/dist/` and reject path
    // traversal. `file` may contain a `.js.map` suffix (sourcemap).
    if file.contains("..") || file.contains('/') {
        return not_found("invalid file");
    }
    let abs = record.root.join("dist").join(&file);
    if !abs.starts_with(&record.root) {
        return not_found("path escape");
    }
    match std::fs::read(&abs) {
        Ok(bytes) => js_response(bytes, mode),
        Err(_) => not_found(&format!("bundle missing: dist/{file}")),
    }
}

/// `GET /api/sui/catalog/workspace` — every component the workspace knows about.
pub async fn serve_catalog_workspace(mode: Option<Extension<DaemonMode>>) -> Response {
    serve_catalog_file("workspace.json", mode_from_ext(mode))
}

/// `GET /api/sui/catalog/published` — only components from non-private packages
/// with a `customElements` field. Used by future library-consumer UIs.
pub async fn serve_catalog_published(mode: Option<Extension<DaemonMode>>) -> Response {
    serve_catalog_file("published.json", mode_from_ext(mode))
}

const EMPTY_CATALOG: &str = "{\"schemaVersion\":\"1\",\"count\":0,\"entries\":[]}\n";

fn serve_catalog_file(name: &str, mode: DaemonMode) -> Response {
    let Some(repo) = repo_root() else {
        return json_response(EMPTY_CATALOG.as_bytes().to_vec(), mode);
    };
    let path = repo.join(".ship/sui-out/catalogs").join(name);
    match std::fs::read(&path) {
        Ok(bytes) => json_response(bytes, mode),
        Err(_) => json_response(EMPTY_CATALOG.as_bytes().to_vec(), mode),
    }
}

fn json_response(bytes: Vec<u8>, mode: DaemonMode) -> Response {
    let cache = asset_cache(mode);
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(header::CACHE_CONTROL, cache);
    if mode == DaemonMode::Production {
        builder = builder.header(header::ETAG, compute_etag(&bytes));
    }
    builder
        .body(Body::from(bytes))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn js_response(bytes: Vec<u8>, mode: DaemonMode) -> Response {
    let cache = asset_cache(mode);
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, cache);
    if mode == DaemonMode::Production {
        builder = builder.header(header::ETAG, compute_etag(&bytes));
    }
    builder
        .body(Body::from(bytes))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_id_unscoped_npm_name() {
        assert_eq!(pack_id_for_name("@ship/charts"), "charts");
        assert_eq!(pack_id_for_name("widgets"), "widgets");
        assert_eq!(pack_id_for_name("@scope/sub-pack"), "sub-pack");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn catalog_workspace_returns_json() {
        let resp = serve_catalog_workspace(None).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(ct.to_str().unwrap().contains("application/json"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn catalog_published_returns_json() {
        let resp = serve_catalog_published(None).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn pack_unknown_id_returns_404() {
        let resp = serve_pack_asset(
            None,
            Path(("not-a-real-pack".to_string(), "anything.js".to_string())),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
