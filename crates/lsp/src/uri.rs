//! `lsp_types::Url` <-> `PathBuf` conversion + authoring-stack detection.
//!
//! Kept in its own module because path/URI round-tripping is the easiest
//! place to break Windows clients. Every helper here has a unit test in
//! [`crate::tests`].

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use lsp_types::Url;

/// Which authoring stack a file belongs to. Drives the LSP's lint
/// dispatch — different stacks have different lint engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stack {
    /// SUI authoring: `*.sui` templates and `*.controller.ts` paired
    /// controllers. Linted by `sui-lint`.
    Sui,
    /// j2 authoring: `*.html.j2` under `.ship/views/` and `*.ship`
    /// SFCs. Linted by `lint`.
    J2,
    /// Not a Ship authoring source — LSP clears any stale diagnostics
    /// and otherwise ignores the document.
    None,
}

/// Convert an absolute filesystem path to a `file://` URL.
///
/// Errors if the path is not absolute or contains characters that
/// `Url::from_file_path` rejects.
pub fn path_to_url(path: &Path) -> Result<Url> {
    Url::from_file_path(path)
        .map_err(|_| anyhow!("path is not absolute or not representable as file://: {}", path.display()))
}

/// Convert a `file://` URL back into a `PathBuf`.
pub fn url_to_path(url: &Url) -> Result<PathBuf> {
    url.to_file_path()
        .map_err(|_| anyhow!("URL is not a file:// path: {}", url))
        .context("URL must be file:// scheme with an absolute path")
}

/// Classify `path` by authoring stack. Drives lint dispatch in the
/// server's `publish_for` path.
///
/// Recognised:
/// - SUI: `**/*.sui` anywhere; `**/*.controller.ts` anywhere.
/// - J2:  `**/*.html.j2` under any `.ship/views/` directory; `**/*.ship`.
/// - None: anything else.
pub fn detect_stack(path: &Path) -> Stack {
    let Some(s) = path.to_str() else { return Stack::None };
    if s.ends_with(".sui") || s.ends_with(".controller.ts") {
        return Stack::Sui;
    }
    if s.ends_with(".ship") {
        return Stack::J2;
    }
    if !s.ends_with(".html.j2") {
        return Stack::None;
    }
    // Walk components looking for the `.ship/views/` segment pair.
    let comps: Vec<_> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    if comps.windows(2).any(|w| w[0] == ".ship" && w[1] == "views") {
        Stack::J2
    } else {
        Stack::None
    }
}

/// Convenience: true iff `detect_stack(path) != Stack::None`.
pub fn is_view_source(path: &Path) -> bool {
    !matches!(detect_stack(path), Stack::None)
}
