//! `.deck/asset-graph.json` — the asset SSOT seed (Step 2 of the asset-graph
//! plan). `build_all` derives it from the artifacts it actually produced:
//! every built target, keyed by its logical name, with the on-disk path
//! (relative to the deck output root), a content hash, and the byte size.
//!
//! Derived-from-the-build is the whole point — the graph cannot disagree with
//! what was bundled, the way an independent filesystem walk (the old catalog,
//! the islands scan, the per-handler probe) can. Additive today: nothing reads
//! it yet. Next, the serve layer routes every asset request through it (one
//! resolver, one root); then the hash becomes the basis for content-addressed,
//! fresh-or-fail serving — no stale-asset window by construction.

use std::hash::{Hash, Hasher};

use crate::targets::{BundleLayout, BundleTarget};

/// Write `.deck/asset-graph.json` for the targets `build_all` successfully
/// produced. Best-effort: a target whose output can't be read is skipped (it
/// already surfaced as a build failure); a serialize/write error is logged,
/// not fatal — the graph is additive and must never break a bake.
pub(crate) fn write_asset_graph(layout: &BundleLayout, built: &[BundleTarget]) {
    let mut assets = serde_json::Map::new();
    for t in built {
        let out_path = layout.assets_dir_for(&t.name).join(&t.out_file);
        let Ok(bytes) = std::fs::read(&out_path) else { continue };
        let rel = out_path
            .strip_prefix(&layout.out_root)
            .unwrap_or(&out_path)
            .to_string_lossy()
            .into_owned();
        assets.insert(
            t.name.clone(),
            serde_json::json!({
                "kind": asset_kind(&t.name),
                "path": rel,
                "hash": content_hash(&bytes),
                "bytes": bytes.len(),
            }),
        );
    }
    let count = assets.len();
    let graph = serde_json::json!({ "version": 0, "assets": assets });
    let dir = layout.repo_root.join(".deck");
    let _ = std::fs::create_dir_all(&dir);
    match serde_json::to_string_pretty(&graph) {
        Ok(json) => {
            if std::fs::write(dir.join("asset-graph.json"), json).is_ok() {
                tracing::info!(assets = count, "bundler: asset graph written");
            }
        }
        Err(e) => tracing::warn!(error = %e, "bundler: asset graph serialize failed"),
    }
}

/// Deterministic content hash for an asset blob. Stable within a toolchain
/// version; a Rust upgrade rehashes everything, which is a rebuild — fine.
fn content_hash(bytes: &[u8]) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut h);
    format!("{:016x}", h.finish())
}

/// Classify a target by its logical name — the same buckets the serve layer
/// routes on. The bare `sui-*` singletons are framework; `views/*` are views;
/// anything else is an app bundle. (Per-tag `sui-components/<name>` chunks
/// are `framework` too — they share the `sui-` prefix.)
fn asset_kind(name: &str) -> &'static str {
    if name.starts_with("views/") {
        "view"
    } else if name.starts_with("sui-") {
        "framework"
    } else {
        "bundle"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::targets::TargetEntry;
    use std::path::PathBuf;

    fn target(name: &str, out_file: &str) -> BundleTarget {
        BundleTarget {
            name: name.into(),
            entry: TargetEntry::Real(PathBuf::from("src/x.ts")),
            externals: vec![],
            out_file: out_file.into(),
        }
    }

    #[test]
    fn graph_lists_built_artifacts_with_hash_and_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let layout = BundleLayout::new(root.clone(), None);

        // Fake two produced artifacts where `build_all` would have written them.
        let rt = layout.assets_dir_for("sui-runtime");
        std::fs::create_dir_all(&rt).unwrap();
        std::fs::write(rt.join("sui-runtime.js"), b"export const v=1\n").unwrap();
        let vdir = layout.assets_dir_for("views/git/view");
        std::fs::create_dir_all(vdir.join("views/git")).unwrap();
        std::fs::write(vdir.join("views/git/view.js"), b"export default 1\n").unwrap();

        write_asset_graph(
            &layout,
            &[
                target("sui-runtime", "sui-runtime.js"),
                target("views/git/view", "views/git/view.js"),
            ],
        );

        let raw = std::fs::read_to_string(root.join(".deck/asset-graph.json")).unwrap();
        let g: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let assets = &g["assets"];
        assert_eq!(assets["sui-runtime"]["kind"], "framework");
        assert_eq!(assets["views/git/view"]["kind"], "view");
        let h = assets["sui-runtime"]["hash"].as_str().unwrap();
        assert_eq!(h.len(), 16, "16-hex content hash");
        let p = assets["sui-runtime"]["path"].as_str().unwrap();
        assert!(p.ends_with("sui-runtime.js"));
        assert!(!p.starts_with('/'), "path is relative to the out root, not absolute");
    }

    #[test]
    fn graph_hash_tracks_content() {
        let a = content_hash(b"export const v=1");
        assert_ne!(a, content_hash(b"export const v=2"), "different content → different hash");
        assert_eq!(a, content_hash(b"export const v=1"), "same content → same hash");
    }

    #[test]
    fn unreadable_target_is_skipped_not_fatal() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let layout = BundleLayout::new(root.clone(), None);
        // No file on disk for this target — must not panic, just omit it.
        write_asset_graph(&layout, &[target("sui-runtime", "sui-runtime.js")]);
        let raw = std::fs::read_to_string(root.join(".deck/asset-graph.json")).unwrap();
        let g: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(g["assets"].as_object().unwrap().len(), 0);
    }
}
