//! Path-layout helpers for composed traveling-artifact overlays.
//!
//! The non-library traveling types (pipelines / triggers / collections /
//! components) read from a *composed root* rather than raw `.ship/` when
//! the space has locked deps. This module provides the resolver
//! (`composed_root_for`) so L1 crates can call it without depending on
//! `ship_platform::registry`.
//!
//! `views` is no longer a materializer type — it's composed by the
//! resolver (`stack_model::compose::compose_space`'s namespace-prefix merge
//! on `ExtensionLibrary::views`) and exposed as
//! `stack_model::ComposedLibrary::route_tree`, superseding the file-copy
//! materializer's views handling per `census-views.md` Part 3 / J2.
//!
//! The materializer is the same algorithm as `registry/compose.rs` — it lives
//! here so `ship-triggers` (and future sibling crates) do not pull in the
//! platform registry.
// FILE_CAP_OVERRIDE: full materialization logic (~270 lines) moved from
// ship-platform/src/registry/compose.rs; cannot split without duplicating types.
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use stack_model::lockfile::ShipLock;

/// The four non-library traveling types — `views` was a fifth and is now
/// composed by the resolver (`stack_model::compose::compose_space`'s
/// namespace-prefix merge on `ExtensionLibrary::views`) and exposed as
/// `stack_model::ComposedLibrary::route_tree`, superseding the file-copy
/// materializer's views handling per `census-views.md` Part 3 / the
/// J2 acceptance. Deck (the consumer) reads the composed tree, not a
/// materialized on-disk `views/` dir.
const COMPOSED_TYPES: &[ComposedType] = &[
    ComposedType { dir: "pipelines",   granularity: Granularity::File },
    ComposedType { dir: "triggers",    granularity: Granularity::File },
    ComposedType { dir: "collections", granularity: Granularity::File },
    ComposedType { dir: "components",  granularity: Granularity::Dir  },
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
/// ship.lock lives at the STACK ROOT beside stack.config.ts (the
/// lockfile-beside-manifest convention; ruled 2026-07-12). ONE location,
/// no legacy fallback. Callers hand us the `.ship/` dir; the lock is at
/// its parent.
fn lock_path_for(ship_dir: &Path) -> PathBuf {
    ship_dir
        .parent()
        .expect("ship_dir must have a parent (the stack root)")
        .join("ship.lock")
}

pub fn composed_root_for(ship_dir: &Path) -> PathBuf {
    let lock_path = lock_path_for(ship_dir);
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

    let lock_path = lock_path_for(ship_dir);
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
            // Route through ShipHome::from_env so SHIP_HOME is honored and
            // the test-context isolation gate kicks in (datasafety-shiphome).
            // A raw `home::home_dir()` bypass here would let a non-isolated
            // test resolve the operator's real `~/.cache/ship/`.
            let home = ship_paths::ShipHome::from_env();
            Ok(home.cache_dir().to_path_buf())
        }
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
        if !obj_dir.is_dir() {
            // Partial install — the dep's cache object isn't on disk
            // yet. The roll-up continues with the rest of the deps.
            continue;
        }

        for ct in COMPOSED_TYPES {
            let dep_dir = obj_dir.join(ct.dir);
            if !dep_dir.is_dir() {
                // The dep may not export any files of this type.
                // Walk-on: nothing to compose for this dir.
                continue;
            }
            for entry in walkdir::WalkDir::new(&dep_dir)
                .min_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();
                let entry_type = entry.file_type();
                // A dep's content is the truth: copy what the
                // dep shipped. Symlinks and special files are not
                // stack content (the pack allow-list rejects them at
                // publish time; defensive checks here in case a
                // hand-laid cache object is non-conformant).
                if entry_type.is_symlink() {
                    continue;
                }
                if !entry_type.is_dir() && !entry_type.is_file() {
                    continue;
                }
                let basename = entry_path
                    .strip_prefix(&dep_dir)
                    .unwrap_or(entry_path)
                    .components()
                    .next()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .unwrap_or_default();
                if basename.is_empty() {
                    continue;
                }
                if ct.dir == "collections"
                    && !entry_type.is_dir()
                    && !seen_collections.insert(basename.clone())
                {
                    anyhow::bail!(
                        "collection-schema collision: two packages export \
                         a same-name collection schema '{}'",
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
                        if !target.exists() {
                            std::fs::create_dir_all(&target).with_context(|| {
                                format!("creating composed dir {}", target.display())
                            })?;
                        }
                        if entry_type.is_dir() {
                            // Continue the walk — sub-entries copy
                            // themselves via their own iteration.
                            continue;
                        }
                        std::fs::copy(entry_path, target.join(entry_path.file_name().unwrap()))
                            .with_context(|| {
                                format!("copying dep {} entry '{}'", ct.dir, basename)
                            })?;
                    }
                    Granularity::File => {
                        if !entry_type.is_file() {
                            // Skip dirs and special entries under a
                            // File-granularity composed dir (the
                            // convention is one file per basename).
                            continue;
                        }
                        std::fs::copy(entry_path, &target).with_context(|| {
                            format!("copying dep {} file '{}'", ct.dir, basename)
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
