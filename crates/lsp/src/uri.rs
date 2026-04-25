//! `lsp_types::Url` <-> `PathBuf` conversion.
//!
//! Kept in its own module because path/URI round-tripping is the easiest
//! place to break Windows clients. Every helper here has a unit test in
//! [`crate::tests`].

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use lsp_types::Url;

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

/// True if a path looks like a Ship view source we should lint.
///
/// Today: `**/*.html.j2` under any `.ship/views/` directory.
///
/// TODO(ship-extension): when the `.ship` extension rename ships,
/// also accept `**/*.ship` here. Keep `.html.j2` for back-compat.
pub fn is_view_source(path: &Path) -> bool {
    let s = match path.to_str() {
        Some(s) => s,
        None => return false,
    };
    if !s.ends_with(".html.j2") {
        return false;
    }
    // Walk components looking for the `.ship/views/` segment pair.
    let comps: Vec<_> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    comps
        .windows(2)
        .any(|w| w[0] == ".ship" && w[1] == "views")
}
