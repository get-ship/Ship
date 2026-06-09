//! Workspace-catalog endpoint — serves `.deck/catalogs/workspace.json` at
//! `/_deck/catalog/workspace`. Emitted by `deck-bundler`'s `build_all`
//! → `refresh_catalog` (bake) and used by devtools/component-picker.
//!
//! The published catalog and pack-asset serving endpoints are removed (#267):
//! the per-view tree-shaking resolver subsumes them.

use crate::{DaemonMode, DeckResponse};

/// `GET /_deck/catalog/workspace` — every component the workspace knows about.
pub fn serve_catalog_workspace(mode: DaemonMode) -> DeckResponse {
    serve_catalog_file("workspace.json", mode)
}

const EMPTY_CATALOG: &str = "{\"schemaVersion\":\"1\",\"count\":0,\"entries\":[]}\n";

fn serve_catalog_file(name: &str, mode: DaemonMode) -> DeckResponse {
    // The bundler bakes catalogs to <project>/.deck/catalogs (deck-bundler
    // build_all). Resolve the SAME runtime root every other asset endpoint uses
    // (deck_out_root = <project>/.deck) — never the build machine's
    // CARGO_MANIFEST_DIR, which an installed or external-workspace daemon would
    // resolve to a nonexistent path, always returning EMPTY_CATALOG.
    let Some(path) = crate::resolver::deck_out_root().map(|root| root.join("catalogs").join(name))
    else {
        return DeckResponse::json_asset(EMPTY_CATALOG.as_bytes().to_vec(), mode);
    };
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

    /// Contract: the catalog is read from the RUNTIME-resolved `.deck/catalogs`
    /// root (deck_out_root), not a build-time path — so an installed/external
    /// daemon serves the real baked catalog. Fails with the old
    /// CARGO_MANIFEST_DIR resolution.
    #[test]
    fn catalog_served_from_resolved_deck_root() {
        use crate::test_lock::ProjectRootGuard;
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".deck/catalogs")).unwrap();
        std::fs::write(
            tmp.path().join(".deck/catalogs/workspace.json"),
            "{\"schemaVersion\":\"1\",\"count\":1,\"entries\":[\"sentinel-tag\"]}",
        )
        .unwrap();
        let _root = ProjectRootGuard::new(tmp.path());
        let body = String::from_utf8(serve_catalog_workspace(DaemonMode::Development).body).unwrap();
        assert!(
            body.contains("sentinel-tag"),
            "catalog must be read from the resolved .deck/catalogs root; got: {body}"
        );
    }
}
