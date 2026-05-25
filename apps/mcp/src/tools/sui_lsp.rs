//! SUI language tools for agents — `sui_diagnostics`, `sui_hover`,
//! `sui_dependencies`.
//!
//! These call the `sui-compiler` analysis facade **in-process** — no `bun`,
//! no Node, no TypeScript. The same Rust intelligence the LSP server uses
//! (`sui_compiler::analyze` / `hover_at`) answers agent queries directly, so
//! the tool works anywhere the daemon runs and adds no external runtime.

use anyhow::{anyhow, Result};
use ship_substrate::env::{self, Var};
use std::path::{Path, PathBuf};

/// Resolve the workspace root — the base that workspace-relative file args
/// resolve against. `SHIP_REPO_DIR` wins; otherwise walk up from CWD to the
/// `bun.lock` marker at the repo root.
pub fn resolve_workspace_root() -> Result<PathBuf> {
    if let Some(repo) = env::read(Var::RepoDir) {
        let root = PathBuf::from(repo);
        if root.join("bun.lock").is_file() {
            return Ok(root);
        }
    }
    let cwd = std::env::current_dir().map_err(|e| anyhow!("cwd unavailable: {e}"))?;
    find_workspace_root(&cwd)
        .ok_or_else(|| anyhow!("workspace root not found; set SHIP_REPO_DIR or run inside the repo"))
}

fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    loop {
        if cur.join("bun.lock").is_file() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

/// Resolve a user-supplied path against the workspace root if it isn't absolute.
fn resolve_file_arg(file: &str, workspace_root: &Path) -> PathBuf {
    let p = PathBuf::from(file);
    if p.is_absolute() { p } else { workspace_root.join(p) }
}

/// Resolve + existence-check a file arg, returning its absolute path and the
/// workspace root it resolved against.
fn validate_file(file: &str) -> Result<(PathBuf, PathBuf)> {
    let workspace_root = resolve_workspace_root()?;
    let abs = resolve_file_arg(file, &workspace_root);
    if !abs.exists() {
        return Err(anyhow!("file not found: {} (resolved from {file})", abs.display()));
    }
    Ok((abs, workspace_root))
}

/// Read the `.sui` source, resolving its path against the workspace root.
fn read_source(file: &str) -> Result<(PathBuf, String)> {
    let (abs, _) = validate_file(file)?;
    let source = std::fs::read_to_string(&abs)
        .map_err(|e| anyhow!("failed to read {}: {e}", abs.display()))?;
    Ok((abs, source))
}

fn pretty(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

/// All diagnostics for a `.sui` file (parse + semantic checks + lint),
/// collected — not stopped at the first.
pub async fn sui_diagnostics(file: &str) -> String {
    match read_source(file) {
        Ok((abs, source)) => {
            let analysis = sui_compiler::analyze(&source, Some(abs.as_path()));
            pretty(&serde_json::json!({ "diagnostics": analysis.diagnostics }))
        }
        Err(e) => format!("Error: {e}"),
    }
}

/// Hover info for the symbol under a 0-based `(line, col)` cursor — kind,
/// origin section, and the controller declaration position.
pub async fn sui_hover(file: &str, line: u32, col: u32) -> String {
    match read_source(file) {
        Ok((abs, source)) => {
            // Agents pass 0-based LSP coordinates; the analysis facade is
            // 1-based, matching the compiler + CLI.
            let (l, c) = (line as usize + 1, col as usize + 1);
            match sui_compiler::hover_at(&source, Some(abs.as_path()), l, c) {
                Some(h) => pretty(&serde_json::json!({ "hover": h })),
                None => pretty(&serde_json::json!({ "hover": null })),
            }
        }
        Err(e) => format!("Error: {e}"),
    }
}

/// The dependency surface of a `.sui` view: custom-element tags it
/// instantiates, `on:` events it handles, and the paired controller path.
pub async fn sui_dependencies(file: &str) -> String {
    match read_source(file) {
        Ok((abs, source)) => {
            let analysis = sui_compiler::analyze(&source, Some(abs.as_path()));
            pretty(&serde_json::json!({
                "tags": analysis.tags,
                "events": analysis.events,
                "controller": analysis.controller_path,
            }))
        }
        Err(e) => format!("Error: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn workspace_root_resolves_to_repo_not_packages() {
        // Regression: must resolve to the repo root, and a workspace-relative
        // view path must resolve to a real file (no `packages/packages/…`).
        let root = resolve_workspace_root().expect("workspace root resolves");
        assert!(!root.ends_with("packages"), "root wrongly in packages/: {}", root.display());
        let (abs, _) = validate_file("views/agents/page.sui").expect("known view resolves");
        assert!(abs.exists(), "resolved path should exist: {}", abs.display());
        assert!(!abs.to_string_lossy().contains("packages/packages"));
    }

    #[test]
    fn resolve_file_arg_keeps_absolute_paths() {
        let ws = PathBuf::from("/tmp/ws");
        assert_eq!(resolve_file_arg("/etc/hosts", &ws), PathBuf::from("/etc/hosts"));
    }

    #[test]
    fn resolve_file_arg_joins_relative_to_workspace() {
        let ws = PathBuf::from("/tmp/ws");
        assert_eq!(
            resolve_file_arg("views/x/page.sui", &ws),
            PathBuf::from("/tmp/ws/views/x/page.sui"),
        );
    }

    #[test]
    fn validate_file_errors_clearly_on_missing() {
        let err = validate_file("definitely/does/not/exist.sui").expect_err("missing errors");
        assert!(format!("{err}").contains("file not found"));
    }

    #[tokio::test]
    async fn diagnostics_returns_json_in_process() {
        // sui-button.sui ships in the workspace; analysis returns a
        // diagnostics array (possibly empty) — never an error string.
        let out = sui_diagnostics("packages/components/primitives/src/sui-button/sui-button.sui").await;
        assert!(!out.starts_with("Error:"), "expected JSON, got: {out}");
        let v: Value = serde_json::from_str(&out).expect("parses as JSON");
        assert!(v.get("diagnostics").map(|d| d.is_array()).unwrap_or(false), "missing diagnostics array: {out}");
    }

    #[tokio::test]
    async fn dependencies_returns_tags_and_events_in_process() {
        let out = sui_dependencies("views/agents/page.sui").await;
        assert!(!out.starts_with("Error:"), "expected JSON, got: {out}");
        let v: Value = serde_json::from_str(&out).expect("parses as JSON");
        let tags = v.get("tags").and_then(|t| t.as_array()).expect("tags array");
        assert!(
            tags.iter().any(|t| t.as_str() == Some("sui-button")),
            "expected sui-button in tags: {out}"
        );
        let events = v.get("events").and_then(|e| e.as_array()).expect("events array");
        assert!(
            events.iter().any(|e| e.as_str() == Some("click")),
            "expected click in events: {out}"
        );
    }
}
