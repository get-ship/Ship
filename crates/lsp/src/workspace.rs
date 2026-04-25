//! Workspace context resolution for the LSP.
//!
//! The lint engine has two entry points:
//! - `lint_view_source(name, content)` — pure in-memory; skips pack
//!   rules because they need a manifest on disk.
//! - `lint_view_dir(view_dir)` — walks the directory, applies every
//!   rule including pack-declaration checks.
//!
//! On `didOpen` / `didSave` we have a real file path on disk and run
//! the directory variant. On `didChange` we only have the buffer text,
//! so we fall back to the in-memory variant — the user gets pack-rule
//! feedback again as soon as they save.
//!
//! Manifest lookup walks up from the open file until it finds a
//! `manifest.{yaml,jsonc,json}` sibling. This mirrors the resolution
//! logic in `apps/shipd/src/views/resolve.rs::read_raw_manifest`
//! without re-implementing it — we only care about the *directory*
//! that contains the manifest, and pack lint already opens the
//! manifest itself via the same `read_raw_manifest` helper inside
//! `lint::rules::packs`.

use std::path::{Path, PathBuf};

use lint::report::Finding;

/// Find the nearest ancestor directory of `file` that contains a
/// recognised manifest file. Returns `None` if the file is not part
/// of a Ship view (no manifest above it before hitting the root).
pub fn find_view_dir(file: &Path) -> Option<PathBuf> {
    let mut cursor = file.parent()?;
    loop {
        if has_manifest(cursor) {
            return Some(cursor.to_path_buf());
        }
        // Stop walking once we leave the `.ship/views/<name>/` envelope:
        // if the *parent* is a `views` directory whose parent is `.ship`,
        // then `cursor` is the view root even if it has no manifest.
        if let (Some(parent), Some(name)) = (cursor.parent(), cursor.file_name()) {
            if name != "views" {
                if let (Some(grand), Some(pname)) = (parent.parent(), parent.file_name()) {
                    if pname == "views" && grand.file_name().and_then(|n| n.to_str()) == Some(".ship") {
                        return Some(cursor.to_path_buf());
                    }
                }
            }
        }
        cursor = cursor.parent()?;
    }
}

fn has_manifest(dir: &Path) -> bool {
    ["manifest.yaml", "manifest.jsonc", "manifest.json"]
        .iter()
        .any(|n| dir.join(n).exists())
}

/// Lint a view, preferring the on-disk directory walk so pack rules
/// surface. Falls back to the in-memory variant when the file is not
/// part of a recognised view layout.
pub fn lint_for_path(path: &Path, fallback_source: &str) -> Vec<Finding> {
    if let Some(dir) = find_view_dir(path) {
        return lint::lint_view_dir(&dir);
    }
    let label = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("view");
    lint::lint_view_source(label, fallback_source)
}

/// In-memory lint for `didChange` — pack rules are skipped (they need
/// a manifest sibling on disk). Returns findings tagged with a label
/// derived from the file path so the LSP can re-attach the URI.
pub fn lint_in_memory(path: &Path, source: &str) -> Vec<Finding> {
    let label = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("view");
    lint::lint_view_source(label, source)
}
