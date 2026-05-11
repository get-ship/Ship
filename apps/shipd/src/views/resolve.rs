//! Resolver helpers — view dir lookup, manifest parsing, type detection.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use super::{RailMeta, ViewListEntry, ViewManifest};

#[derive(Deserialize)]
pub(crate) struct ManifestFile {
    name: Option<String>,
    #[serde(rename = "type")]
    view_type: Option<String>,
    element: Option<String>,
    claims: Option<Vec<String>>,
    entry: Option<String>,
    pub data: Option<ManifestData>,
    pub events: Option<Vec<String>>,
    /// Optional list of bundle packs this view needs at load time.
    /// Each pack name expands to a fixed set of JS bundles that the daemon
    /// injects as `<script type="module">` tags into the response. Known
    /// packs live in [`super::packs`]. Unknown names are silently skipped.
    #[serde(default)]
    pub packs: Option<Vec<String>>,
    /// Shell display metadata — rail label, icon, pin defaults.
    /// Absent → view has no pinned/launcher presence unless the shell
    /// falls back (e.g. user-authored views default to their id).
    #[serde(default)]
    pub rail: Option<ManifestRail>,
}

/// View identity for the shell rail / launcher. No preference data —
/// pin state, order, default-pinned are user settings (tier 2 storage:
/// daemon-backed `shell.rail.*` keys), not per-view config.
#[derive(Deserialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ManifestRail {
    pub label: Option<String>,
    pub icon: Option<String>,
}

impl ManifestRail {
    pub fn into_meta(self, view_name: &str) -> RailMeta {
        RailMeta {
            label: self.label.unwrap_or_else(|| humanize(view_name)),
            icon: self.icon.unwrap_or_else(|| "app-window".into()),
        }
    }
}

fn humanize(id: &str) -> String {
    id.replace('-', " ")
        .split(' ')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(ch) => ch.to_uppercase().chain(c).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Legacy manifest `data:` declarations, consumed by
/// `views::context::build_generic_context` for views still on the old
/// injection pathway (canvas, setup, overview). File-routed views on
/// the `ship.*` SDK ignore this block. Only `collections` is read
/// today — the other fields were loaders for pre-framework views that
/// have since migrated; kept here so older manifests still parse.
#[derive(Deserialize, Clone)]
pub(crate) struct ManifestData {
    pub collections: Option<ManifestCollections>,
    #[serde(default)]
    pub items: Option<String>,
    // Legacy fields — ignored at runtime. Remove once no in-tree manifest
    // declares them.
    #[serde(default, skip_serializing)]
    pub settings: Option<bool>,
    #[serde(default, skip_serializing)]
    pub agents: Option<bool>,
    #[serde(default, skip_serializing)]
    pub skills: Option<bool>,
    #[serde(default, skip_serializing)]
    pub events_data: Option<bool>,
    #[serde(default, skip_serializing)]
    pub chat: Option<bool>,
    #[serde(default, skip_serializing)]
    pub specs: Option<bool>,
    #[serde(default, skip_serializing)]
    pub pipelines: Option<bool>,
    #[serde(default, skip_serializing)]
    pub git: Option<bool>,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum ManifestCollections {
    All(String),
    List(Vec<String>),
}

pub(crate) fn is_valid_view_name(name: &str) -> bool {
    !name.is_empty() && !name.contains("..") && !name.contains('/') && !name.contains('\\')
}

/// Resolve a view directory by name. Checks project then global dirs.
pub(crate) fn resolve_view_dir(name: &str) -> Option<(PathBuf, &'static str)> {
    if !is_valid_view_name(name) { return None; }
    if let Ok(d) = ship_application::project::get_project_dir(None) {
        let c = d.join("views").join(name);
        if c.is_dir() { return Some((c, "project")); }
    }
    if let Ok(d) = ship_application::project::get_global_dir() {
        let c = d.join("views").join(name);
        if c.is_dir() { return Some((c, "global")); }
    }
    None
}

/// Read the raw manifest file (with data/events fields) for the SSE handler.
///
/// Supports three formats, checked in order: `manifest.yaml`, `manifest.jsonc`
/// (comments stripped), `manifest.json`.
///
/// Parse errors are logged at `warn` level with the file path + error
/// detail, then the handler falls back to an absent manifest — rather
/// than silently hiding author mistakes. Authors see the diagnostic
/// in daemon logs while the view still renders its index.html.j2
/// (sans rail metadata, events, packs).
pub(crate) fn read_raw_manifest(dir: &std::path::Path) -> Option<ManifestFile> {
    fn warn_parse(path: &std::path::Path, err: impl std::fmt::Display) {
        tracing::warn!(
            manifest = %path.display(),
            error = %err,
            "view manifest parse failed — view will render without manifest metadata. \
             Check schema at https://raw.githubusercontent.com/madvib/ship/main/schemas/view-manifest.schema.json"
        );
    }

    if dir.join("manifest.yaml").exists() {
        let path = dir.join("manifest.yaml");
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => { warn_parse(&path, e); return None; }
        };
        match serde_yaml::from_str(&raw) {
            Ok(m) => Some(m),
            Err(e) => { warn_parse(&path, e); None }
        }
    } else if dir.join("manifest.jsonc").exists() {
        let path = dir.join("manifest.jsonc");
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => { warn_parse(&path, e); return None; }
        };
        let json = compiler::jsonc::strip_jsonc_comments(&raw);
        match serde_json::from_str(&json) {
            Ok(m) => Some(m),
            Err(e) => { warn_parse(&path, e); None }
        }
    } else if dir.join("manifest.json").exists() {
        let path = dir.join("manifest.json");
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => { warn_parse(&path, e); return None; }
        };
        match serde_json::from_str(&raw) {
            Ok(m) => Some(m),
            Err(e) => { warn_parse(&path, e); None }
        }
    } else {
        None
    }
}

pub(super) fn read_manifest_file(dir: &PathBuf, name: &str) -> Option<ViewManifest> {
    let raw: ManifestFile = read_raw_manifest(dir)?;

    let vtype = raw.view_type.unwrap_or_else(|| "html".into());
    let mname = raw.name.unwrap_or_else(|| name.into());
    let (el, url) = if vtype == "component" {
        (Some(raw.element.unwrap_or_else(|| format!("ship-{name}-view"))),
         Some(format!("/api/v1/view/{name}/bundle")))
    } else {
        (raw.element, None)
    };
    Some(ViewManifest { name: mname, view_type: vtype, element_name: el, bundle_url: url, claims: raw.claims, entry: raw.entry })
}

fn detect_view_type(dir: &PathBuf) -> Option<String> {
    if dir.join("manifest.yaml").exists()
        || dir.join("manifest.jsonc").exists()
        || dir.join("manifest.json").exists()
    {
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
        let path = entry.path();
        if let Some(vtype) = detect_view_type(&path) {
            // Read rail metadata from the manifest if present. User-authored
            // views without a `rail:` block still land in the launcher (the
            // shell fills in a fallback label/icon client-side).
            let rail = read_raw_manifest(&path)
                .and_then(|m| m.rail)
                .map(|r| r.into_meta(&name));
            out.insert(
                name.clone(),
                ViewListEntry { name, view_type: vtype, source: source.into(), rail },
            );
        }
    }
}
