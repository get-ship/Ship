//! Workspace-catalog endpoint — serves `.deck/catalogs/workspace.json` at
//! `/_sui/catalog/workspace`. Emitted by `deck-bundler`'s `build_all`
//! → `refresh_catalog` (bake) and used by devtools/component-picker.
//!
//! The published catalog and pack-asset serving endpoints are removed (#267):
//! the per-view tree-shaking resolver subsumes them.

use crate::{DaemonMode, DeckResponse};

/// `GET /_sui/catalog/workspace` — every component the workspace knows about.
pub fn serve_catalog_workspace(mode: DaemonMode) -> DeckResponse {
    serve_catalog_file("workspace.json", mode)
}

const EMPTY_CATALOG: &str = "{\"schemaVersion\":\"1\",\"count\":0,\"entries\":[]}\n";

fn serve_catalog_file(name: &str, mode: DaemonMode) -> DeckResponse {
    // deck-core lives at crates/deck/deck-core. Repo root is three parents up.
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(std::path::PathBuf::from);
    let Some(repo) = repo else {
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
    fn catalog_workspace_returns_json() {
        let resp = serve_catalog_workspace(DaemonMode::Development);
        assert_eq!(resp.status, 200);
        assert!(resp.content_type.contains("application/json"));
    }
}
