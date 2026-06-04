//! Workspace discovery for the catalog aggregator (#163 catalog port).
//!
//! Port of `@ship/sui-catalog`'s `workspace.ts`: read the root `package.json`
//! `workspaces.packages` globs, expand each (single trailing `*` only), and
//! return every package dir with a parsable `package.json` that has a `name`.

use std::path::{Path, PathBuf};

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct WorkspacePackage {
    /// npm name (`@ship/charts`).
    pub name: String,
    /// Absolute path to the package root.
    pub root: PathBuf,
    /// `package.json.private === true`.
    pub private: bool,
    /// `package.json.customElements` (relative to root), if any.
    pub custom_elements: Option<String>,
}

pub(super) fn read_json(path: &Path) -> Option<Value> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// The `workspaces.packages` glob list (array or `{ packages: [...] }` form).
fn workspace_globs(repo_root: &Path) -> Vec<String> {
    let Some(manifest) = read_json(&repo_root.join("package.json")) else {
        return Vec::new();
    };
    let ws = &manifest["workspaces"];
    if let Some(arr) = ws.as_array() {
        return arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
    }
    ws["packages"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

/// Expand one glob against the repo root. Supports a single trailing `*`
/// (`packages/*`); literal paths return `[self]`. A stricter pattern (`pkg-*`)
/// is an explicit error so the aggregator never silently drops packages.
fn expand_glob(repo_root: &Path, glob: &str) -> Result<Vec<PathBuf>, String> {
    if !glob.contains('*') {
        let abs = repo_root.join(glob);
        return Ok(if abs.exists() { vec![abs] } else { vec![] });
    }
    let (parent_rel, pattern) = match glob.rfind('/') {
        Some(i) => (&glob[..i], &glob[i + 1..]),
        None => (".", glob),
    };
    if pattern != "*" {
        return Err(format!(
            "workspace glob \"{glob}\" uses unsupported pattern \"{pattern}\""
        ));
    }
    let parent_abs = repo_root.join(parent_rel);
    let Ok(entries) = std::fs::read_dir(&parent_abs) else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        if e.path().is_dir() {
            out.push(e.path());
        }
    }
    Ok(out)
}

/// Walk `workspaces.packages` and return every package dir with a parsable
/// `package.json` carrying a `name`. Sorted by name (byte-stable).
pub fn discover_workspace_packages(repo_root: &Path) -> Result<Vec<WorkspacePackage>, String> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for g in workspace_globs(repo_root) {
        for dir in expand_glob(repo_root, &g)? {
            if !seen.insert(dir.clone()) {
                continue;
            }
            let Some(manifest) = read_json(&dir.join("package.json")) else {
                continue;
            };
            let Some(name) = manifest["name"].as_str() else {
                continue;
            };
            out.push(WorkspacePackage {
                name: name.to_string(),
                root: dir,
                private: manifest["private"].as_bool() == Some(true),
                custom_elements: manifest["customElements"].as_str().map(String::from),
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// npm name → daemon pack-id. `@ship/charts` → `charts`; `widgets` → `widgets`.
pub fn pack_id_for_name(npm_name: &str) -> String {
    if let Some(rest) = npm_name.strip_prefix('@') {
        if let Some(slash) = rest.find('/') {
            return rest[slash + 1..].to_string();
        }
    }
    npm_name.to_string()
}

/// Recursively walk `dir`, collecting `(tag, abs_path)` pairs where the file
/// stem equals its parent directory name — i.e. `<X>/<X>.sui`. This is the
/// source-mode component convention; views (`page.sui`, stem ≠ parent) are
/// excluded by the guard. `dist`, `node_modules`, `.git`, and `target` are
/// skipped for speed.
fn scan_sui_components(dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    const SKIP: [&str; 5] = ["node_modules", "dist", "target", ".git", ".deck"];
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if SKIP.contains(&name.as_str()) {
            continue;
        }
        let p = e.path();
        if p.is_dir() {
            scan_sui_components(&p, out);
        } else if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            if p.extension().and_then(|x| x.to_str()) != Some("sui") {
                continue;
            }
            // Guard: stem must equal the parent directory name.
            let parent_name = p
                .parent()
                .and_then(|par| par.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if stem == parent_name {
                out.push((stem.to_string(), p));
            }
        }
    }
}

/// Return all `(tag, abs_path)` component pairs discovered by source scan of
/// `pkg_root/src/`. Only `<X>/<X>.sui` files (stem == parent dir) are
/// returned; views and other files are excluded by the guard.
pub fn source_tags_for_package(pkg_root: &Path) -> Vec<(String, PathBuf)> {
    let src = pkg_root.join("src");
    if !src.is_dir() {
        return vec![];
    }
    let mut out = Vec::new();
    scan_sui_components(&src, &mut out);
    out
}
