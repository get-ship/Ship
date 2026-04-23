//! Per-view bundle loading.
//!
//! Views opt into heavy JS bundles ("packs") via `packs: [editor, terminal]`
//! in their `manifest.yaml` / `manifest.jsonc`. The daemon reads the pack
//! list at render time and injects `<script type="module">` tags for each
//! member bundle into the response. The shell only eager-loads the two
//! core bundles (`datastar.js`, `ship-elements.js`); every island bundle
//! arrives per-view.
//!
//! Packs are intentionally coarse — `editor` bundles TipTap and CodeMirror
//! because most views that need one need the other, and splitting them adds
//! complexity without benefit. New packs (e.g. `viz` for charts) get added
//! here as a single source of truth.
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

use std::path::PathBuf;

/// Map a pack name to the JS bundle filenames served under
/// `/api/assets/`. Returning a `&'static [&'static str]` keeps this
/// a trivially-cheap lookup with no allocation.
pub(crate) fn pack_bundles(name: &str) -> &'static [&'static str] {
    match name {
        "editor" => &["ship-code-editor.js", "ship-markdown-editor.js"],
        "terminal" => &["ship-terminal.js"],
        "git" => &["ship-git.js"],
        _ => &[],
    }
}

/// Read the `packs:` list from a view directory's manifest. Returns an
/// empty vec when the view has no manifest or no packs declared. Silently
/// drops unknown pack names so a typo doesn't break the view — the lint
/// rule `stale-pack-declaration` surfaces those at author time.
pub(crate) fn read_view_packs(dir: &PathBuf) -> Vec<String> {
    let Some(raw) = super::resolve::read_raw_manifest(dir) else {
        return Vec::new();
    };
    raw.packs.unwrap_or_default()
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
