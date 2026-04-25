//! Per-view bundle loading.
//!
//! Views opt into heavy JS bundles ("packs") via `packs: [editor, terminal]`
//! in their `manifest.yaml` / `manifest.jsonc`. The daemon reads the pack
//! list at render time and injects `<script type="module">` tags for each
//! member bundle into the response. The shell only eager-loads the two
//! core bundles (`datastar.js`, `ship-elements.js`); every island bundle
//! arrives per-view.
//!
//! ## Closed enum (v1)
//!
//! The pack vocabulary is a closed set: [`KNOWN_PACKS`]. The view manifest
//! JSON Schema (`schemas/view-manifest.schema.json`) enforces the same
//! enum at author time, and this module is the runtime source of truth
//! at load time — unknown names are rejected with a clear error naming
//! the manifest path and the offending value. Pack extensibility for
//! third-party authors is a Phase B concern (see
//! `docs/ship-component-packs/references/docs/manifest-and-distribution.md`).
//!
//! Packs are intentionally coarse — `editor` bundles TipTap and CodeMirror
//! because most views that need one need the other, and splitting them adds
//! complexity without benefit. Adding a first-party pack means extending
//! [`KNOWN_PACKS`], adding a match arm to [`pack_bundles`], updating the
//! schema enum, and adding a `bundle:*` script in `packages/ship-set`.
//!
//! ## Render-path contract
//!
//! - Full-page response (direct browser load): inject into `<head>` so the
//!   module preload happens alongside the core bundles.
//! - Fragment response (shell navigation, `Datastar-Request: true`): inject
//!   at the top of the returned body. `mount.ts` re-creates `<script>`
//!   nodes after `innerHTML` assignment, so module scripts actually execute.
//!
//! Dedupe: the injector is called once per response with the full pack
//! list. A single view = a single set of packs, so duplicates within one
//! view can't happen. Cross-view chaining (e.g. pipelines focus nested
//! under pipelines) is a future problem.

use std::path::{Path, PathBuf};

/// Closed set of pack names accepted at v1. Mirrors the `enum` in
/// `schemas/view-manifest.schema.json`. Adding a name here requires a
/// matching schema update plus a `pack_bundles` arm and a bundle script.
pub(crate) const KNOWN_PACKS: &[&str] = &["editor", "terminal", "git"];

/// Map a pack name to the JS bundle filenames served under
/// `/api/assets/`. Returning a `&'static [&'static str]` keeps this
/// a trivially-cheap lookup with no allocation. Names outside
/// [`KNOWN_PACKS`] return an empty slice — callers should validate
/// via [`validate_packs`] before reaching this function.
pub(crate) fn pack_bundles(name: &str) -> &'static [&'static str] {
    match name {
        "editor" => &["ship-code-editor.js", "ship-markdown-editor.js"],
        "terminal" => &["ship-terminal.js"],
        "git" => &["ship-git.js"],
        _ => &[],
    }
}

/// Validation failure for a manifest's `packs:` list. Carries the
/// manifest path + offending value so the runtime can log a precise
/// error and tests can assert on the message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PackValidationError {
    pub manifest: PathBuf,
    pub offending: String,
}

impl std::fmt::Display for PackValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "view manifest {manifest} declares unknown pack {offending:?}; \
             allowed values: {allowed:?}",
            manifest = self.manifest.display(),
            offending = self.offending,
            allowed = KNOWN_PACKS,
        )
    }
}

impl std::error::Error for PackValidationError {}

/// Validate a `packs:` list against [`KNOWN_PACKS`]. Returns the list
/// unchanged on success, or an error naming the first offending value
/// and the manifest path. Empty / missing lists are valid.
pub(crate) fn validate_packs(
    manifest_path: &Path,
    packs: &[String],
) -> Result<Vec<String>, PackValidationError> {
    for pack in packs {
        if !KNOWN_PACKS.contains(&pack.as_str()) {
            return Err(PackValidationError {
                manifest: manifest_path.to_path_buf(),
                offending: pack.clone(),
            });
        }
    }
    Ok(packs.to_vec())
}

/// Locate a view's manifest file inside `dir`. Mirrors the precedence
/// in `read_raw_manifest`. Returns `None` when no manifest exists.
fn find_manifest_path(dir: &Path) -> Option<PathBuf> {
    for fname in ["manifest.yaml", "manifest.jsonc", "manifest.json"] {
        let p = dir.join(fname);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Read and validate the `packs:` list from a view directory's manifest.
///
/// Returns an empty vec when the view has no manifest or no packs
/// declared. **Unknown pack names cause the entire pack list for that
/// view to be dropped**, with a `tracing::error!` naming the manifest
/// path and offending value — the runtime is the source of truth for
/// the closed enum, and partial injection of "the known half" of a
/// broken declaration would mask the author's mistake. The lint rule
/// `pkg-stale-pack-declaration` is the author-time double-check.
pub(crate) fn read_view_packs(dir: &PathBuf) -> Vec<String> {
    let Some(raw) = super::resolve::read_raw_manifest(dir) else {
        return Vec::new();
    };
    let packs = raw.packs.unwrap_or_default();
    if packs.is_empty() {
        return Vec::new();
    }
    let manifest_path = find_manifest_path(dir).unwrap_or_else(|| dir.clone());
    match validate_packs(&manifest_path, &packs) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(
                manifest = %err.manifest.display(),
                offending = %err.offending,
                allowed = ?KNOWN_PACKS,
                "view manifest declares unknown pack — pack injection skipped for this view"
            );
            Vec::new()
        }
    }
}

/// Build `<script type="module" src="/api/assets/<bundle>">` tags
/// for every bundle mapped by the given pack list. Deduplicates bundles
/// across packs (so two packs that share a bundle only emit it once) and
/// returns the concatenated HTML string. Empty when no packs resolve.
pub(crate) fn render_pack_scripts(packs: &[String]) -> String {
    let mut emitted: Vec<&'static str> = Vec::new();
    for name in packs {
        for bundle in pack_bundles(name) {
            if !emitted.contains(bundle) {
                emitted.push(bundle);
            }
        }
    }
    if emitted.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for bundle in emitted {
        out.push_str(&format!(
            "<script type=\"module\" src=\"/api/assets/{bundle}\"></script>\n"
        ));
    }
    out
}

/// Inject pack scripts into a rendered HTML response.
///
/// - `is_fragment = false`: insert into `<head>` (just before `</head>`).
///   Falls back to prepending the document if no head is present — that
///   shouldn't happen for full-page responses but keeps this pure.
/// - `is_fragment = true`: prepend to the body. `mount.ts` rehydrates
///   script tags after `innerHTML=`, so module loads fire.
pub(crate) fn inject_packs(html: String, packs: &[String], is_fragment: bool) -> String {
    if packs.is_empty() {
        return html;
    }
    let scripts = render_pack_scripts(packs);
    if scripts.is_empty() {
        return html;
    }
    if is_fragment {
        let mut out = String::with_capacity(html.len() + scripts.len());
        out.push_str(&scripts);
        out.push_str(&html);
        return out;
    }
    // Full-page: prefer injecting before </head> so module preloads happen
    // alongside the core bundles. Case-insensitive match — handles `</HEAD>`
    // or odd casing without requiring a regex dependency.
    if let Some(idx) = find_close_head(&html) {
        let mut out = String::with_capacity(html.len() + scripts.len());
        out.push_str(&html[..idx]);
        out.push_str(&scripts);
        out.push_str(&html[idx..]);
        return out;
    }
    // No </head> found (malformed or fragment-shaped response misclassified
    // as full page): prepend and let the browser execute before the body.
    let mut out = String::with_capacity(html.len() + scripts.len());
    out.push_str(&scripts);
    out.push_str(&html);
    out
}

/// Locate `</head>` case-insensitively without pulling in regex. Returns
/// the byte index of the opening `<` so callers can splice before it.
fn find_close_head(html: &str) -> Option<usize> {
    let bytes = html.as_bytes();
    let needle = b"</head>";
    for i in 0..bytes.len().saturating_sub(needle.len() - 1) {
        let window = &bytes[i..i + needle.len()];
        let matches = window
            .iter()
            .zip(needle.iter())
            .all(|(a, b)| a.to_ascii_lowercase() == *b);
        if matches {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
#[path = "packs_tests.rs"]
mod tests;
