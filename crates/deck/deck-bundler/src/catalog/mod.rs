//! Component catalog aggregation.
//!
//! Walk per-component CEMs from published packages (each package's
//! `customElements` manifest) AND from `.ship/sui-out/**/*.custom-elements.json`
//! (non-packaged: shell-local, view-local), merge into the canonical workspace
//! catalog document. Filesystem-only, deterministic (sort by tagName).
//! Pass-through CEM arrays are kept as raw JSON so component metadata isn't
//! lost to a typed schema.

mod workspace;

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Value, json};

pub use workspace::{WorkspacePackage, discover_workspace_packages, pack_id_for_name};
use workspace::read_json;

const SUI_OUT: &str = ".ship/sui-out";
const CEM_SUFFIX: &str = ".custom-elements.json";
const SKIP_DIRS: [&str; 4] = ["node_modules", "dist", "target", ".git"];

/// One catalog entry. Pass-through CEM arrays are raw `Value`.
#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct CatalogEntry {
    pub tagName: String,
    pub description: String,
    pub firstDesc: String,
    pub attributes: Value,
    pub slots: Value,
    pub events: Value,
    pub methods: Value,
    pub cssProperties: Value,
    pub cssParts: Value,
    pub examples: Value,
    pub package: String,
    pub private: bool,
    pub sourcePath: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct CatalogDocument {
    pub schemaVersion: &'static str,
    pub count: usize,
    pub entries: Vec<CatalogEntry>,
}

pub fn stringify_catalog(doc: &CatalogDocument) -> String {
    serde_json::to_string_pretty(doc).unwrap_or_default() + "\n"
}

/// Build the workspace catalog for a repo — every component, sorted by tagName.
///
/// `first_party_app` is the optional embedded host app (e.g. `apps/shell` →
/// `app:shell` from `deck.jsonc`). When `None`, the shell-local CEM projection
/// does not apply — suitable for zero-config hosts.
pub fn build_catalog(
    repo_root: &Path,
    first_party_app: Option<&deck_config::FirstPartyApp>,
) -> Result<CatalogDocument, String> {
    let packages = discover_workspace_packages(repo_root)?;

    let mut all: Vec<CatalogEntry> = Vec::new();
    for pkg in &packages {
        if pkg.custom_elements.is_some() {
            all.extend(entries_from_packaged_manifest(repo_root, pkg));
        }
    }
    all.extend(entries_from_sui_out(repo_root, &packages, first_party_app));

    all.sort_by(|a, b| a.tagName.cmp(&b.tagName));
    // Dedup by tagName, first wins (packaged entries precede sui-out ones).
    let mut seen = std::collections::HashSet::new();
    all.retain(|e| seen.insert(e.tagName.clone()));

    Ok(CatalogDocument {
        schemaVersion: "1",
        count: all.len(),
        entries: all,
    })
}

/// Map every workspace component tag → its source `.sui` path (absolute), for
/// the per-view resolver: the bundler resolves `@sui-tag/<tag>` to this path so
/// the component compiles into the view bundle. Derived from the same CEM walk
/// that drives catalog enumeration — one source of truth. A tag whose source
/// can't be located is simply absent; the resolver then falls back to external.
pub fn tag_source_index(
    repo_root: &Path,
    first_party_app: Option<&deck_config::FirstPartyApp>,
) -> std::collections::HashMap<String, PathBuf> {
    let mut out = std::collections::HashMap::new();
    let Ok(workspace) = build_catalog(repo_root, first_party_app) else {
        return out;
    };
    for e in workspace.entries {
        if !e.sourcePath.ends_with(".sui") {
            continue;
        }
        let abs = repo_root.join(&e.sourcePath);
        if abs.is_file() {
            out.insert(e.tagName, abs);
        }
    }
    out
}

fn arr_or_empty(v: &Value) -> Value {
    if v.is_array() { v.clone() } else { json!([]) }
}

/// Build an entry from a CEM declaration. `None` unless it's a named custom
/// element (`tagName` + `customElement: true`).
fn entry_from_decl(
    decl: &Value,
    package: String,
    private: bool,
    source_path: String,
) -> Option<CatalogEntry> {
    let tag = decl["tagName"].as_str().filter(|s| !s.is_empty())?;
    if decl["customElement"].as_bool() != Some(true) {
        return None;
    }
    let description = decl["description"].as_str().unwrap_or("").trim().to_string();
    let first_desc = description.split('\n').next().unwrap_or("").to_string();
    Some(CatalogEntry {
        tagName: tag.to_string(),
        description,
        firstDesc: first_desc,
        attributes: arr_or_empty(&decl["attributes"]),
        slots: arr_or_empty(&decl["slots"]),
        events: arr_or_empty(&decl["events"]),
        methods: arr_or_empty(&decl["members"]),
        cssProperties: arr_or_empty(&decl["cssProperties"]),
        cssParts: arr_or_empty(&decl["cssParts"]),
        examples: arr_or_empty(&decl["examples"]),
        package,
        private,
        sourcePath: source_path,
    })
}

fn entries_from_packaged_manifest(repo_root: &Path, pkg: &WorkspacePackage) -> Vec<CatalogEntry> {
    let Some(ce) = &pkg.custom_elements else { return vec![] };
    let manifest_abs = pkg.root.join(ce);
    if !manifest_abs.exists() {
        return vec![];
    }
    let Some(cem) = read_json(&manifest_abs) else { return vec![] };
    let Some(modules) = cem["modules"].as_array() else { return vec![] };
    let pkg_rel_root = rel_unix(repo_root, &pkg.root);

    let mut out = Vec::new();
    for m in modules {
        let mod_path = m["path"].as_str().unwrap_or("");
        let stem = mod_path.strip_suffix(".sui").unwrap_or(mod_path);
        let source_path = if !stem.is_empty() {
            format!("{pkg_rel_root}/src/{stem}/{stem}.sui")
        } else {
            format!("{pkg_rel_root}/src/{mod_path}")
        };
        if let Some(decls) = m["declarations"].as_array() {
            for decl in decls {
                if let Some(e) = entry_from_decl(
                    decl,
                    pkg.name.clone(),
                    pkg.private,
                    source_path.clone(),
                ) {
                    out.push(e);
                }
            }
        }
    }
    out
}

fn entries_from_sui_out(
    repo_root: &Path,
    packages: &[WorkspacePackage],
    first_party_app: Option<&deck_config::FirstPartyApp>,
) -> Vec<CatalogEntry> {
    let root = repo_root.join(SUI_OUT);
    if !root.exists() {
        return vec![];
    }
    let mut cem_files = Vec::new();
    walk_cem_files(&root, &mut cem_files);
    cem_files.sort();

    // Owning-package lookup, longest rel path first so nested packages win.
    let mut packaged_roots: Vec<(String, &WorkspacePackage)> =
        packages.iter().map(|p| (rel_unix(repo_root, &p.root), p)).collect();
    packaged_roots.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    // First-party app root prefix for the shell-local CEM projection. Derived
    // from `deck.jsonc firstPartyApp.root`; absent in zero-config hosts so the
    // projection simply doesn't apply (no host-app assumption here).
    let fpa_root_prefix = first_party_app.as_ref().map(|fpa| {
        let r = fpa.root.to_string_lossy().replace('\\', "/");
        if r.ends_with('/') { r } else { format!("{r}/") }
    });
    let fpa_label = first_party_app.as_ref().map(|fpa| fpa.label.as_str());

    let mut out = Vec::new();
    for cem_abs in &cem_files {
        let source_path = source_path_for_cem(repo_root, cem_abs);
        let owning = find_owning_package(&source_path, &packaged_roots);
        // Owning pack with an aggregated manifest → served from there; skip.
        if owning.is_some_and(|(_, p)| p.custom_elements.is_some()) {
            continue;
        }
        let Some(cem) = read_json(cem_abs) else { continue };
        let Some(modules) = cem["modules"].as_array() else { continue };

        // When the source lives under the configured first-party app root, use
        // the configured label (e.g. `app:shell`) — no hardcoded path.
        let is_first_party_local = fpa_root_prefix
            .as_deref()
            .is_some_and(|prefix| source_path.starts_with(prefix));
        let origin = if is_first_party_local {
            fpa_label.unwrap_or("app:unknown").to_string()
        } else {
            owning
                .map(|(_, p)| p.name.clone())
                .unwrap_or_else(|| origin_for_non_packaged_source(&source_path))
        };
        let is_private = owning.map(|(_, p)| p.private).unwrap_or(true);

        for m in modules {
            if let Some(decls) = m["declarations"].as_array() {
                for decl in decls {
                    if let Some(e) = entry_from_decl(
                        decl,
                        origin.clone(),
                        is_private,
                        source_path.clone(),
                    ) {
                        out.push(e);
                    }
                }
            }
        }
    }
    out
}

fn walk_cem_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if SKIP_DIRS.contains(&name.as_str()) {
            continue;
        }
        let p = e.path();
        if p.is_dir() {
            walk_cem_files(&p, out);
        } else if name.ends_with(CEM_SUFFIX) {
            out.push(p);
        }
    }
}

fn find_owning_package<'a>(
    rel: &str,
    packaged_roots: &'a [(String, &'a WorkspacePackage)],
) -> Option<&'a (String, &'a WorkspacePackage)> {
    packaged_roots
        .iter()
        .find(|(r, _)| rel == r || rel.starts_with(&format!("{r}/")))
}

/// Repo-relative source `.sui` path for a per-component CEM under
/// `.ship/sui-out/<mirror>/<stem>.custom-elements.json`.
fn source_path_for_cem(repo_root: &Path, cem_abs: &Path) -> String {
    let repo_rel = rel_unix(repo_root, cem_abs);
    let mirrored = repo_rel
        .strip_prefix(&format!("{SUI_OUT}/"))
        .unwrap_or(&repo_rel);
    let file = mirrored.rsplit('/').next().unwrap_or(mirrored);
    let stem = file.strip_suffix(CEM_SUFFIX).unwrap_or(file);
    let dir = &mirrored[..mirrored.len() - file.len()];
    format!("{dir}{stem}.sui")
}

/// Origin tag for a non-packaged source. `views/<name>/` → `view:<name>`;
/// else `workspace:<first-seg>`. First-party app labelling is done by the
/// caller via the configured `firstPartyApp` — no path literal here.
fn origin_for_non_packaged_source(norm: &str) -> String {
    if let Some(rest) = norm.strip_prefix("views/") {
        if let Some(name) = rest.split('/').next().filter(|s| !s.is_empty()) {
            return format!("view:{name}");
        }
    }
    if let Some(idx) = norm.find("assets/views/") {
        let rest = &norm[idx + "assets/views/".len()..];
        if let Some(name) = rest.split('/').next().filter(|s| !s.is_empty()) {
            return format!("view:{name}");
        }
    }
    let first = norm.split('/').next().unwrap_or("unknown");
    format!("workspace:{first}")
}

fn rel_unix(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write(path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn cem(tag: &str) -> String {
        json!({
            "schemaVersion": "1",
            "modules": [{
                "path": format!("{tag}.sui"),
                "declarations": [{
                    "kind": "class", "name": tag, "tagName": tag,
                    "customElement": true, "description": "demo"
                }]
            }]
        })
        .to_string()
    }

    #[test]
    fn aggregates_packaged_and_sui_out() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), r#"{"workspaces":["packages/*"]}"#);
        // A public pack with a CEM manifest.
        write(
            &root.join("packages/charts/package.json"),
            r#"{"name":"@ship/charts","customElements":"dist/custom-elements.json"}"#,
        );
        write(&root.join("packages/charts/dist/custom-elements.json"), &cem("sui-bar"));
        // A shell-local component in the sui-out mirror → app:shell (via configured firstPartyApp).
        write(
            &root.join(".ship/sui-out/apps/shell/src/sui-x/sui-x.custom-elements.json"),
            &cem("sui-x"),
        );

        // Configured first-party app: apps/shell → app:shell.
        let fpa = deck_config::FirstPartyApp {
            root: std::path::PathBuf::from("apps/shell"),
            label: "app:shell".to_string(),
        };
        let workspace = build_catalog(root, Some(&fpa)).unwrap();

        let ws_tags: Vec<&str> = workspace.entries.iter().map(|e| e.tagName.as_str()).collect();
        assert_eq!(ws_tags, vec!["sui-bar", "sui-x"], "sorted, both present");

        let bar = workspace.entries.iter().find(|e| e.tagName == "sui-bar").unwrap();
        assert_eq!(bar.package, "@ship/charts");
        assert!(!bar.private);

        let x = workspace.entries.iter().find(|e| e.tagName == "sui-x").unwrap();
        assert_eq!(x.package, "app:shell");
        assert!(x.private);
    }

    #[test]
    fn aggregates_sui_out_zero_config() {
        // Without firstPartyApp, apps/shell components fall through to
        // origin_for_non_packaged_source → workspace:apps (not app:shell).
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), r#"{"workspaces":[]}"#);
        write(
            &root.join(".ship/sui-out/apps/shell/src/sui-x/sui-x.custom-elements.json"),
            &cem("sui-x"),
        );
        let workspace = build_catalog(root, None).unwrap();
        let x = workspace.entries.iter().find(|e| e.tagName == "sui-x").unwrap();
        // No firstPartyApp configured — origin falls back to workspace:<first-seg>.
        assert_eq!(x.package, "workspace:apps");
    }

    #[test]
    fn index_maps_tag_to_existing_source_and_omits_missing() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), r#"{"workspaces":["packages/*"]}"#);
        write(
            &root.join("packages/charts/package.json"),
            r#"{"name":"@ship/charts","customElements":"dist/custom-elements.json"}"#,
        );
        write(&root.join("packages/charts/dist/custom-elements.json"), &cem("sui-bar"));
        // The CEM implies source packages/charts/src/sui-bar/sui-bar.sui — write it.
        write(&root.join("packages/charts/src/sui-bar/sui-bar.sui"), "<div>bar</div>\n");

        let idx = tag_source_index(root, None);
        // Invariant: an indexed tag points at an EXISTING `.sui` source.
        let src = idx.get("sui-bar").expect("a tag with a real source is indexed");
        assert!(src.is_file());
        assert_eq!(src.extension().and_then(|e| e.to_str()), Some("sui"));
        assert!(src.ends_with("packages/charts/src/sui-bar/sui-bar.sui"));
        // Invariant: an unknown tag is absent → the resolver falls back to external.
        assert!(!idx.contains_key("sui-nonexistent"));
    }
}
