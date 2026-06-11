//! Resolver helpers — view dir lookup, manifest parsing, type detection.


use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct ManifestFile {
    #[allow(dead_code)]
    pub events: Option<Vec<String>>,
    /// Optional list of bundle packs this view needs at load time.
    /// Each pack name expands to a fixed set of JS bundles that the daemon
    /// injects as `<script type="module">` tags into the response. Known
    /// packs live in [`super::packs`]. Unknown names are silently skipped.
    #[serde(default)]
    #[allow(dead_code)]
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
