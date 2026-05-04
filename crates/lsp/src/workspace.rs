//! Workspace context resolution + lint dispatch for the LSP.
//!
//! Dispatch by stack (j2 vs SUI), then by event (`didChange` vs
//! `didSave`). For j2, the lint engine has two entry points:
//!
//! - `lint_view_source(name, content)` — pure in-memory; skips pack
//!   rules because they need a manifest on disk.
//! - `lint_view_dir(view_dir)` — walks the directory, applies every
//!   rule including pack-declaration checks.
//!
//! For SUI, `sui_lint::lint_path` reads from disk; controllers are
//! self-contained, so there's no in-memory vs. on-disk split.
//!
//! On `didOpen` / `didSave` we have a real file path on disk and run
//! the directory variant (j2) or the path variant (SUI). On
//! `didChange` we only have the buffer text, so for j2 we fall back to
//! the in-memory variant; for SUI we re-read from disk (controller
//! sources are typically small and the user just saved).
//!
//! Manifest lookup walks up from the open file until it finds a
//! `manifest.{yaml,jsonc,json}` sibling.

use std::path::{Path, PathBuf};

use lint::report::Finding;

use crate::uri::{detect_stack, Stack};

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

/// Lint a path, dispatching by authoring stack.
///
/// SUI sources (`*.sui`, `*.controller.ts`) go through `sui_lint`.
/// j2 sources prefer the on-disk directory walk so pack rules
/// surface. Anything else returns no findings.
pub fn lint_for_path(path: &Path, fallback_source: &str) -> Vec<Finding> {
    match detect_stack(path) {
        Stack::Sui => sui_lint::lint_path(path),
        Stack::J2 => {
            if let Some(dir) = find_view_dir(path) {
                return lint::lint_view_dir(&dir);
            }
            let label = path.file_stem().and_then(|s| s.to_str()).unwrap_or("view");
            lint::lint_view_source(label, fallback_source)
        }
        Stack::None => Vec::new(),
    }
}

/// In-memory lint for `didChange`. For SUI controllers we lint the
/// supplied buffer directly (no manifest dependency); for j2 we fall
/// back to the in-memory engine which skips pack rules. Returns
/// findings tagged with a label derived from the file path so the LSP
/// can re-attach the URI.
pub fn lint_in_memory(path: &Path, source: &str) -> Vec<Finding> {
    match detect_stack(path) {
        Stack::Sui => sui_lint::lint_controller_source(path, source),
        Stack::J2 => {
            let label = path.file_stem().and_then(|s| s.to_str()).unwrap_or("view");
            lint::lint_view_source(label, source)
        }
        Stack::None => Vec::new(),
    }
}
