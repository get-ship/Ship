//! Path-layout helpers for composed traveling-artifact overlays.
//!
//! The four non-library traveling types (views / pipelines / triggers /
//! collections) read from a *composed root* rather than raw `.ship/` when the
//! space has locked deps. This module provides the resolver
//! (`composed_root_for`) so L1 crates can call it without depending on
//! `ship_platform::registry`.
//!
//! The materializer is the same algorithm as `registry/compose.rs` — it lives
//! here so `ship-triggers` (and future sibling crates) do not pull in the
//! platform registry.
// FILE_CAP_OVERRIDE: full materialization logic (~270 lines) moved from
// ship-platform/src/registry/compose.rs; cannot split without duplicating types.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use compiler::lockfile::ShipLock;
use compiler::manifest::ShipManifest;

/// The four non-library traveling types.
const COMPOSED_TYPES: &[ComposedType] = &[
    ComposedType { dir: "views",       granularity: Granularity::Dir  },
    ComposedType { dir: "pipelines",   granularity: Granularity::File },
    ComposedType { dir: "triggers",    granularity: Granularity::File },
    ComposedType { dir: "collections", granularity: Granularity::File },
];

#[derive(Clone, Copy)]
enum Granularity { Dir, File }

#[derive(Clone, Copy)]
struct ComposedType {
    dir: &'static str,
    granularity: Granularity,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// The traveling-artifact root a loader should read for `ship_dir`.
///
/// - **Depless space** (no `ship.lock` or no packages): the raw `.ship/` itself.
/// - **Has deps**: the materialized composed overlay (local base + dep exports).
///
/// Falls back to the raw `.ship/` on any materialization error so composition
/// never breaks a load that worked before deps were introduced.
pub fn composed_root_for(ship_dir: &Path) -> PathBuf {
    let lock_path = ship_dir.join("ship.lock");
    let has_deps = ShipLock::from_file(&lock_path)
        .map(|l| !l.packages.is_empty())
        .unwrap_or(false);
    if !has_deps {
        return ship_dir.to_path_buf();
    }
    match materialize_space(ship_dir, None) {
        Ok(root) => root,
        Err(e) => {
            tracing::warn!(
                "compose: materialize failed for {}, falling back to raw .ship: {e}",
                ship_dir.display()
            );
            ship_dir.to_path_buf()
        }
    }
}

/// Resolve the local `.ship/` base + locked dep exports into a composed root.
///
/// Returns the path to `<cache_root>/composed/<input-hash>/`. The directory is
/// content-hash-cached: identical inputs return the existing dir without a
/// rebuild. Builds happen into a temp dir then atomically rename into place.
pub fn materialize_space(ship_dir: &Path, cache_root: Option<&Path>) -> Result<PathBuf> {
    let cache_root = resolve_cache_root(cache_root)?;

    let lock_path = ship_dir.join("ship.lock");
    let lock = if lock_path.exists() {
        ShipLock::from_file(&lock_path)
            .with_context(|| format!("reading {}", lock_path.display()))?
    } else {
        ShipLock::default()
    };

    let input_hash = compute_input_hash(ship_dir, &lock)?;
    let hash_dir = input_hash.strip_prefix("sha256:").unwrap_or(&input_hash);
    let composed_root = cache_root.join("composed").join(hash_dir);

    if composed_root.is_dir() {
        return Ok(composed_root);
    }

    let composed_parent = cache_root.join("composed");
    std::fs::create_dir_all(&composed_parent).context("creating composed cache dir")?;

    let tmp_dir = composed_parent.join(format!(".tmp-{hash_dir}"));
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir).context("removing stale tmp composed dir")?;
    }
    std::fs::create_dir_all(&tmp_dir).context("creating tmp composed dir")?;

    build_overlay(ship_dir, &lock, &cache_root, &tmp_dir)?;

    if composed_root.exists() {
        std::fs::remove_dir_all(&tmp_dir).ok();
    } else {
        std::fs::rename(&tmp_dir, &composed_root)
            .context("renaming tmp composed dir into place")?;
    }

    Ok(composed_root)
}

// ── Private helpers ──────────────────────────────────────────────────────────

fn resolve_cache_root(cache_root: Option<&Path>) -> Result<PathBuf> {
    match cache_root {
        Some(p) => Ok(p.to_path_buf()),
        None => {
            let home = home::home_dir().context("cannot determine home directory")?;
            Ok(crate::ShipHome::project_ship_dir(home).join("cache"))
        }
    }
}

fn exports_for<'a>(manifest: &'a ShipManifest, dir: &str) -> &'a [String] {
    match dir {
        "views"       => &manifest.exports.views,
        "pipelines"   => &manifest.exports.pipelines,
        "triggers"    => &manifest.exports.triggers,
        "collections" => &manifest.exports.collections,
        _             => &[],
    }
}

fn compute_input_hash(ship_dir: &Path, lock: &ShipLock) -> Result<String> {
    let mut acc = String::new();
    for ct in COMPOSED_TYPES {
        let dir = ship_dir.join(ct.dir);
        if dir.is_dir() {
            let h = compute_tree_hash(&dir)
                .with_context(|| format!("hashing local {} dir", ct.dir))?;
            acc.push_str(ct.dir);
            acc.push('\0');
            acc.push_str(&h);
            acc.push('\n');
        }
    }
    let mut deps: Vec<(&str, &str)> = lock
        .packages
        .iter()
        .map(|p| (p.path.as_str(), p.hash.as_str()))
        .collect();
    deps.sort_by(|a, b| a.0.cmp(b.0));
    for (path, hash) in deps {
        acc.push_str(path);
        acc.push('\0');
        acc.push_str(hash);
        acc.push('\n');
    }
    Ok(hex_sha256(acc.as_bytes()))
}

fn build_overlay(
    ship_dir: &Path,
    lock: &ShipLock,
    cache_root: &Path,
    dest: &Path,
) -> Result<()> {
    for ct in COMPOSED_TYPES {
        let src = ship_dir.join(ct.dir);
        if src.is_dir() {
            let target = dest.join(ct.dir);
            std::fs::create_dir_all(&target)
                .with_context(|| format!("creating composed {} dir", ct.dir))?;
            copy_dir_all(&src, &target)
                .with_context(|| format!("copying local {} into overlay", ct.dir))?;
        }
    }

    let mut seen_collections: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for pkg in &lock.packages {
        let hex = pkg.hash.strip_prefix("sha256:").unwrap_or(&pkg.hash);
        let obj_dir = cache_root.join("objects").join(hex);
        let manifest_path = obj_dir.join("ship.jsonc");
        if !manifest_path.exists() {
            continue;
        }
        let manifest = ShipManifest::from_file(&manifest_path)
            .with_context(|| format!("reading dep manifest {}", manifest_path.display()))?;

        for ct in COMPOSED_TYPES {
            for export_rel in exports_for(&manifest, ct.dir) {
                let src = obj_dir.join(export_rel);
                if !src.exists() {
                    anyhow::bail!(
                        "dep '{}' exports {} '{}' but it is absent from the cache at {}",
                        pkg.path, ct.dir, export_rel, src.display()
                    );
                }
                let basename = Path::new(export_rel)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| export_rel.clone());
                if ct.dir == "collections" && !seen_collections.insert(basename.clone()) {
                    anyhow::bail!(
                        "collection-schema collision: two packages export a \
                         same-name collection schema '{}'",
                        basename
                    );
                }
                let target = dest.join(ct.dir).join(&pkg.path).join(&basename);
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!("creating namespace dir {}", parent.display())
                    })?;
                }
                match ct.granularity {
                    Granularity::Dir => {
                        std::fs::create_dir_all(&target).with_context(|| {
                            format!("creating composed dir {}", target.display())
                        })?;
                        copy_dir_all(&src, &target).with_context(|| {
                            format!("copying dep {} '{}'", ct.dir, export_rel)
                        })?;
                    }
                    Granularity::File => {
                        std::fs::copy(&src, &target).with_context(|| {
                            format!("copying dep {} '{}'", ct.dir, export_rel)
                        })?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry.context("walking source dir")?;
        let rel = entry.path().strip_prefix(src).context("stripping prefix")?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target)
                .with_context(|| format!("copying {}", entry.path().display()))?;
        }
    }
    Ok(())
}

fn compute_tree_hash(root: &Path) -> Result<String> {
    let mut file_entries: Vec<(String, u64, String)> = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| e.file_name() != ".git")
    {
        let entry = entry.context("walking package tree")?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(root).context("stripping root prefix")?;
        let rel_str = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        if should_exclude(&rel_str) {
            continue;
        }
        let content = std::fs::read(entry.path())
            .with_context(|| format!("reading {}", entry.path().display()))?;
        let byte_len = content.len() as u64;
        let file_hash = encode_sha256(&content);
        file_entries.push((rel_str, byte_len, file_hash));
    }
    file_entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut accumulator = String::new();
    for (rel_path, byte_len, file_hash) in &file_entries {
        accumulator.push_str(rel_path);
        accumulator.push('\0');
        accumulator.push_str(&byte_len.to_string());
        accumulator.push('\0');
        accumulator.push_str(file_hash);
    }
    Ok(format!("sha256:{}", encode_sha256(accumulator.as_bytes())))
}

fn should_exclude(rel_path: &str) -> bool {
    let parts: Vec<&str> = rel_path.split('/').collect();
    if parts.first() == Some(&".git") { return true; }
    if rel_path == "ship.lock" { return true; }
    if parts.first() == Some(&"state") { return true; }
    let filename = parts.last().unwrap_or(&"");
    matches!(*filename, ".DS_Store" | "Thumbs.db") || filename.ends_with(".swp")
}

fn hex_sha256(data: &[u8]) -> String {
    format!("sha256:{}", encode_sha256(data))
}

fn encode_sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{b:02x}")).collect()
}
