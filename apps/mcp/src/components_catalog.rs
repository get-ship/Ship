//! SUI component catalog exposed as `ship://components` MCP resources.
//!
//! Reads `.ship/sui-out/catalogs/workspace.json` (emitted by
//! `@ship/sui-catalog`) at runtime. Replaces the build-time
//! `include_str!("…/sui-custom-elements.json")` cut-over that pinned the
//! MCP server to a stale aggregate baked at the last `cargo build`.
//!
//! Two reads happen per request: full catalog + per-tag filter. The
//! workspace.json file is small (≪ 1 MB even for a large monorepo) and
//! gets cached at the filesystem layer by the OS, so re-parsing on each
//! call keeps the implementation simple. Switch to a OnceLock cache if
//! ever called on a hot path.

use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const CATALOG_REL: &str = ".ship/sui-out/catalogs/workspace.json";

fn repo_root() -> PathBuf {
    // apps/mcp lives at apps/mcp. Repo root is two parents up from
    // CARGO_MANIFEST_DIR. Distinct from deck-server's three-level
    // walk-up (it's nested under crates/sui/).
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_workspace_catalog() -> Option<Value> {
    let path = repo_root().join(CATALOG_REL);
    let raw = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str::<Value>(&raw).ok()
}

/// Return the workspace catalog as a JSON string. Falls back to an empty
/// document when the on-disk catalog is missing (fresh clone with no
/// build artefacts) so callers can still parse the response.
pub fn full_catalog() -> String {
    let catalog =
        read_workspace_catalog().unwrap_or_else(|| json!({"schemaVersion": "1", "count": 0, "entries": []}));
    serde_json::to_string(&catalog).unwrap_or_else(|_| "{}".to_string())
}

/// Return a single-entry catalog for `tag`, or `None` if the tag is
/// unknown. Output keeps the top-level catalog shape (schemaVersion +
/// count + entries) so consumers can use one parser for both projections.
pub fn component_catalog(tag: &str) -> Option<String> {
    let catalog = read_workspace_catalog()?;
    let entries = catalog.get("entries")?.as_array()?;
    let matched: Vec<Value> = entries
        .iter()
        .filter(|e| e.get("tagName").and_then(Value::as_str) == Some(tag))
        .cloned()
        .collect();
    if matched.is_empty() {
        return None;
    }
    let out = json!({
        "schemaVersion": catalog.get("schemaVersion").cloned().unwrap_or_else(|| json!("1")),
        "count": matched.len(),
        "entries": matched,
    });
    serde_json::to_string(&out).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_catalog_returns_parseable_json() {
        let s = full_catalog();
        let v: Value = serde_json::from_str(&s).expect("must parse");
        assert!(v.get("entries").and_then(Value::as_array).is_some());
    }

    #[test]
    fn component_catalog_unknown_tag_returns_none() {
        assert!(component_catalog("sui-definitely-not-real-zzz").is_none());
    }
}
