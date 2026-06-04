//! Read side of `.deck/asset-graph.json` — the serve layer's view of the asset
//! SSOT the bundler emits (`deck-bundler::asset_graph`). One resolver, one
//! root: the graph file is located through `resolve_project_root()` (the same
//! root the producer wrote against), so the serve layer and the build agree on
//! where assets live by construction, not convention.
//!
//! Reloaded whenever the file's mtime advances — a rebuild/HMR cycle rewrites
//! the graph, so a served path is never stale relative to the last build (the
//! property the whole asset-graph plan exists to guarantee). Absent graph
//! (older bake, fresh checkout, on-demand-only project) → `None`, and callers
//! fall back to the conventional on-disk probe. Additive: today only
//! `resolve_view_js` consults it; Step 3 routes the remaining handlers through
//! it and lets the probing fall out as dead code.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use serde::Deserialize;

/// One built artifact, as recorded by the bundler. `path` is relative to the
/// deck output root (the project root the producer built under), so the serve
/// layer resolves it against `resolve_project_root()`.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetNode {
    pub kind: String,
    pub path: String,
    pub hash: String,
    pub bytes: usize,
}

#[derive(Debug, Default, Deserialize)]
pub struct AssetGraph {
    #[serde(default)]
    pub assets: HashMap<String, AssetNode>,
}

impl AssetGraph {
    pub fn get(&self, name: &str) -> Option<&AssetNode> {
        self.assets.get(name)
    }
}

struct Cached {
    mtime: SystemTime,
    graph: Arc<AssetGraph>,
}

static CACHE: RwLock<Option<Cached>> = RwLock::new(None);

fn graph_path() -> Option<PathBuf> {
    crate::resolver::deck_out_root().map(|d| d.join("asset-graph.json"))
}

/// Parse a graph file. Pure (no env, no cache) — the unit-testable core.
fn load_from(path: &std::path::Path) -> Option<AssetGraph> {
    let raw = std::fs::read(path).ok()?;
    serde_json::from_slice(&raw).ok()
}

/// The current asset graph, reloaded whenever the file's mtime advances.
/// `None` when no graph has been produced (callers fall back to probing).
pub fn graph() -> Option<Arc<AssetGraph>> {
    let path = graph_path()?;
    let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok()?;
    if let Ok(guard) = CACHE.read()
        && let Some(c) = guard.as_ref()
        && c.mtime == mtime
    {
        return Some(c.graph.clone());
    }
    let arc = Arc::new(load_from(&path)?);
    if let Ok(mut guard) = CACHE.write() {
        *guard = Some(Cached { mtime, graph: arc.clone() });
    }
    Some(arc)
}

/// Resolve a logical asset name (e.g. `sui-runtime`, `views/git/view`) to its
/// node in the current graph, or `None` if the graph is absent or lacks it.
pub fn lookup(name: &str) -> Option<AssetNode> {
    graph()?.get(name).cloned()
}

/// Absolute on-disk path of a node's artifact: its `path` (relative to the
/// project/out root) resolved against `resolve_project_root()`. `None` when no
/// project root is known.
pub fn resolve(node: &AssetNode) -> Option<PathBuf> {
    crate::resolver::resolve_project_root().map(|r| r.join(&node.path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_graph(dir: &std::path::Path, body: &str) -> PathBuf {
        let p = dir.join("asset-graph.json");
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        p
    }

    #[test]
    fn parses_nodes_and_looks_them_up() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_graph(
            tmp.path(),
            r#"{"version":0,"assets":{
                "sui-runtime":{"kind":"framework","path":"crates/deck/deck-core/assets/sui-runtime.js","hash":"00ff00ff00ff00ff","bytes":42},
                "views/git/view":{"kind":"view","path":".deck/views/git/view.js","hash":"abcdabcdabcdabcd","bytes":7}
            }}"#,
        );
        let g = load_from(&p).expect("valid graph parses");
        let rt = g.get("sui-runtime").expect("framework node present");
        assert_eq!(rt.kind, "framework");
        assert_eq!(rt.bytes, 42);
        assert_eq!(rt.hash, "00ff00ff00ff00ff");
        let v = g.get("views/git/view").expect("view node present");
        assert_eq!(v.path, ".deck/views/git/view.js");
        assert!(g.get("nope").is_none());
    }

    #[test]
    fn malformed_graph_is_none_not_panic() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_graph(tmp.path(), "{ not json");
        assert!(load_from(&p).is_none());
    }

    #[test]
    fn missing_assets_key_defaults_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_graph(tmp.path(), r#"{"version":0}"#);
        let g = load_from(&p).expect("parses with no assets key");
        assert!(g.get("anything").is_none());
    }
}
