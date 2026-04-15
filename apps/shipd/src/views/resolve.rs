//! Resolver helpers — view dir lookup, manifest parsing, type detection.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use super::{ViewListEntry, ViewManifest};

#[derive(Deserialize)]
pub(super) struct ManifestFile {
    name: Option<String>,
    #[serde(rename = "type")]
    view_type: Option<String>,
    element: Option<String>,
    claims: Option<Vec<String>>,
    entry: Option<String>,
}

pub(crate) fn is_valid_view_name(name: &str) -> bool {
    !name.is_empty() && !name.contains("..") && !name.contains('/') && !name.contains('\\')
}

/// Resolve a view directory by name. Checks project then global dirs.
pub(crate) fn resolve_view_dir(name: &str) -> Option<(PathBuf, &'static str)> {
    if !is_valid_view_name(name) { return None; }
    if let Ok(d) = runtime::project::get_project_dir(None) {
        let c = d.join("views").join(name);
        if c.is_dir() { return Some((c, "project")); }
    }
    if let Ok(d) = runtime::project::get_global_dir() {
        let c = d.join("views").join(name);
        if c.is_dir() { return Some((c, "global")); }
    }
    None
}

pub(super) fn read_manifest_file(dir: &PathBuf, name: &str) -> Option<ViewManifest> {
    let raw: ManifestFile = if dir.join("manifest.yaml").exists() {
        serde_yaml::from_str(&std::fs::read_to_string(dir.join("manifest.yaml")).ok()?).ok()?
    } else if dir.join("manifest.json").exists() {
        serde_json::from_str(&std::fs::read_to_string(dir.join("manifest.json")).ok()?).ok()?
    } else {
        return None;
    };

    let vtype = raw.view_type.unwrap_or_else(|| "html".into());
    let mname = raw.name.unwrap_or_else(|| name.into());
    let (el, url) = if vtype == "component" {
        (Some(raw.element.unwrap_or_else(|| format!("ship-{name}-view"))),
         Some(format!("/api/runtime/view/{name}/bundle")))
    } else {
        (raw.element, None)
    };
    Some(ViewManifest { name: mname, view_type: vtype, element_name: el, bundle_url: url, claims: raw.claims, entry: raw.entry })
}

fn detect_view_type(dir: &PathBuf) -> Option<String> {
    if dir.join("manifest.yaml").exists() || dir.join("manifest.json").exists() {
        if let Some(m) = read_manifest_file(dir, "unknown") { return Some(m.view_type); }
    }
    if dir.join("index.html").exists() { return Some("html".into()); }
    if dir.join("index.js").exists() { return Some("component".into()); }
    None
}

pub(super) fn scan_views_dir(dir: &PathBuf, source: &'static str, out: &mut HashMap<String, ViewListEntry>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
        let name = entry.file_name().to_string_lossy().to_string();
        if out.contains_key(&name) { continue; }
        if let Some(vtype) = detect_view_type(&entry.path()) {
            out.insert(name.clone(), ViewListEntry { name, view_type: vtype, source: source.into() });
        }
    }
}
