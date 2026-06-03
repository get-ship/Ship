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
//! read pre-aggregated JSON from `.deck/catalogs/` (emitted by
//! `bun packages/deck/catalog/src/cli.ts`). They live in this module so
//! all pack-discovery + catalog plumbing sits together.

use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path as StdPath, PathBuf};
use std::sync::OnceLock;

use crate::{DaemonMode, DeckResponse};

#[derive(Clone, Debug)]
struct PackRecord {
    /// Absolute path to the package root (the dir containing package.json).
    root: PathBuf,
    /// Value of `package.json.customElements`, relative to root.
    custom_elements: String,
}

static PACK_INDEX: OnceLock<HashMap<String, PackRecord>> = OnceLock::new();

fn repo_root() -> Option<PathBuf> {
    // deck-core lives at crates/deck/deck-core. Repo root is
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

/// `GET /_sui/packs/{pack_id}/{file}` — serves either the pack's
/// `dist/<file>` JS bundle or its aggregated `custom-elements.json`.
pub fn serve_pack_asset(mode: DaemonMode, pack_id: &str, file: &str) -> DeckResponse {
    let Some(record) = pack_index().get(pack_id) else {
        return DeckResponse::not_found(format!("unknown pack '{pack_id}'"), mode);
    };
    // Custom-elements.json: serve the file the package declared via its
    // `customElements` manifest field, regardless of where it lives in
    // the pack tree.
    if file == "custom-elements.json" {
        let abs = record.root.join(&record.custom_elements);
        return match std::fs::read(&abs) {
            Ok(bytes) => DeckResponse::json_asset(bytes, mode),
            Err(_) => DeckResponse::not_found(format!("manifest missing: {}", abs.display()), mode),
        };
    }
    // Bundle files: confine reads to `<root>/dist/` and reject path
    // traversal. `file` may contain a `.js.map` suffix (sourcemap).
    if file.contains("..") || file.contains('/') {
        return DeckResponse::not_found("invalid file", mode);
    }
    let abs = record.root.join("dist").join(file);
    if !abs.starts_with(&record.root) {
        return DeckResponse::not_found("path escape", mode);
    }
    match std::fs::read(&abs) {
        Ok(bytes) => DeckResponse::js_asset(bytes, mode),
        Err(_) => DeckResponse::not_found(format!("bundle missing: dist/{file}"), mode),
    }
}

/// `GET /api/sui/catalog/workspace` — every component the workspace knows about.
pub fn serve_catalog_workspace(mode: DaemonMode) -> DeckResponse {
    serve_catalog_file("workspace.json", mode)
}

/// `GET /api/sui/catalog/published` — only components from non-private packages
/// with a `customElements` field. Used by future library-consumer UIs.
pub fn serve_catalog_published(mode: DaemonMode) -> DeckResponse {
    serve_catalog_file("published.json", mode)
}

const EMPTY_CATALOG: &str = "{\"schemaVersion\":\"1\",\"count\":0,\"entries\":[]}\n";

fn serve_catalog_file(name: &str, mode: DaemonMode) -> DeckResponse {
    let Some(repo) = repo_root() else {
        return DeckResponse::json_asset(EMPTY_CATALOG.as_bytes().to_vec(), mode);
    };
    let path = repo.join(".deck/catalogs").join(name);
    match std::fs::read(&path) {
        Ok(bytes) => DeckResponse::json_asset(bytes, mode),
        Err(_) => DeckResponse::json_asset(EMPTY_CATALOG.as_bytes().to_vec(), mode),
    }
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

    #[test]
    fn catalog_workspace_returns_json() {
        let resp = serve_catalog_workspace(DaemonMode::Development);
        assert_eq!(resp.status, 200);
        assert!(resp.content_type.contains("application/json"));
    }

    #[test]
    fn catalog_published_returns_json() {
        let resp = serve_catalog_published(DaemonMode::Development);
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn pack_unknown_id_returns_404() {
        let resp = serve_pack_asset(DaemonMode::Development, "not-a-real-pack", "anything.js");
        assert_eq!(resp.status, 404);
    }
}
