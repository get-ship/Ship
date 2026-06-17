//! `sources/` tree emission — the on-disk source surface for the studio viewer.
//!
//! `ship.ui.Packs` and `ship.ui.Read` (Connect, L1 ship-views) serve a
//! `ReadReq { path }` by reading from `<ship_dir>/.deck/sources/<path>`. The
//! deck-bundler owns emitting that tree as a build artifact: every `.sui`
//! file in `packages/components/{sui,ship}/*/src/**` lands at the matching
//! relative path under `_deck/sources/`.
//!
//! Path is the identifier — there is no per-component manifest, no version
//! tag, no metadata. The `Read` resolver chain is bundle → CDN → npm →
//! ship-store, but the studio viewer's fast path is always the local bundle.
//!
//! Why emit at all (instead of reading from the source tree directly at
//! runtime)? The runtime doesn't know the repo root, the source layout, or
//! the ship_dir / source_dir distinction. The bundler does. The bundle
//! emission keeps the runtime decoupled from the workspace surface.

use std::path::{Path, PathBuf};

use crate::targets::BundleLayout;

/// Walk the component source roots and copy every `.sui` file into
/// `<out>/_deck/sources/<rel_path>`. Returns the number of files written.
///
/// `out` is the layout's `deck_out()` (typically `<ship_dir>/.deck`); the
/// resulting `sources/` tree lands at `<out>/sources/`.
pub fn build_sources(layout: &BundleLayout) -> Result<usize, SourcesError> {
    let deck_out = layout.deck_out();
    let sources_root = deck_out.join("sources");
    std::fs::create_dir_all(&sources_root).map_err(|e| SourcesError::CreateDir {
        path: sources_root.clone(),
        source: e,
    })?;

    let mut count = 0usize;
    for source_root in &layout.source_roots {
        count += copy_sui_files(source_root, &sources_root)?;
    }
    Ok(count)
}

fn copy_sui_files(source_root: &Path, dest_root: &Path) -> Result<usize, SourcesError> {
    let components = source_root.join("packages").join("components");
    if !components.is_dir() {
        return Ok(0);
    }
    let mut count = 0usize;
    for category in ["sui", "ship"] {
        let category_dir = components.join(category);
        if !category_dir.is_dir() {
            continue;
        }
        // category/<pack>/src/**/<name>.sui
        for pack in std::fs::read_dir(&category_dir).map_err(|e| SourcesError::ReadDir {
            path: category_dir.clone(),
            source: e,
        })? {
            let pack = pack.map_err(|e| SourcesError::DirEntry { source: e })?;
            let pack_path = pack.path();
            if !pack_path.is_dir() {
                continue;
            }
            let src_dir = pack_path.join("src");
            if !src_dir.is_dir() {
                continue;
            }
            count += walk_dir(&src_dir, &src_dir, dest_root, category, pack_path.file_name())?;
        }
    }
    Ok(count)
}

fn walk_dir(
    src_root: &Path,
    dir: &Path,
    dest_root: &Path,
    category: &str,
    pack_name: Option<&std::ffi::OsStr>,
) -> Result<usize, SourcesError> {
    let mut count = 0usize;
    for entry in std::fs::read_dir(dir).map_err(|e| SourcesError::ReadDir {
        path: dir.to_path_buf(),
        source: e,
    })? {
        let entry = entry.map_err(|e| SourcesError::DirEntry { source: e })?;
        let path = entry.path();
        let ty = entry.file_type().map_err(|e| SourcesError::FileType { source: e })?;
        if ty.is_dir() {
            count += walk_dir(src_root, &path, dest_root, category, pack_name)?;
        } else if ty.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sui") {
            let rel = path
                .strip_prefix(src_root)
                .map_err(|_| SourcesError::StripPrefix { path: path.clone() })?
                .to_string_lossy()
                .replace('\\', "/");
            // Emit path: <dest>/<category>/<pack>/<rel>. The category
            // is the top-level dir of the file's first path segment
            // (matches the bundle's view catalog convention: @sui/<pack>,
            // @ship/<pack>).
            let pack_str = pack_name.and_then(|s| s.to_str()).unwrap_or("unknown");
            let dest = dest_root.join(category).join(pack_str).join(&rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| SourcesError::CreateDir {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }
            std::fs::copy(&path, &dest).map_err(|e| SourcesError::Copy {
                src: path.clone(),
                dst: dest.clone(),
                source: e,
            })?;
            count += 1;
        }
    }
    Ok(count)
}

#[derive(Debug, thiserror::Error)]
pub enum SourcesError {
    #[error("create sources dir {path}: {source}")]
    CreateDir { path: PathBuf, source: std::io::Error },
    #[error("read dir {path}: {source}")]
    ReadDir { path: PathBuf, source: std::io::Error },
    #[error("dir entry: {source}")]
    DirEntry { source: std::io::Error },
    #[error("file type: {source}")]
    FileType { source: std::io::Error },
    #[error("strip_prefix {path}")]
    StripPrefix { path: PathBuf },
    #[error("copy {src} -> {dst}: {source}")]
    Copy {
        src: PathBuf,
        dst: PathBuf,
        source: std::io::Error,
    },
}
