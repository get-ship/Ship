//! Composition materializer (#122 — slice 1).
//!
//! Resolves the local `.ship/` base plus every locked dep's exports into ONE
//! composed `.ship` view under `~/.ship/cache/composed/<input-hash>/`. Loaders
//! read the composed root, not raw `.ship/` — but slice 1 only BUILDS the
//! overlay; it does not repoint any loader (that is slice 2). The materializer
//! is additive and unwired this slice.
//!
//! Layout produced:
//! ```text
//! ~/.ship/cache/composed/<input-hash>/
//!   views/ pipelines/ triggers/ collections/
//! ```
//! - **Layer 0** = the space's own `.ship/` traveling dirs (copied first).
//! - **Layers 1..N** = each locked dep's exported artifacts, copied from
//!   `~/.ship/cache/objects/<hex>/<export path>` under a **package namespace**
//!   (`<type>/<pkg.path>/<basename>` — mirrors `resolve_dep_agents`'s
//!   `format!("{pkg.path}/{id}")` scheme).
//!
//! Scope (slice 1): only the four non-library types
//! (views / pipelines / triggers / collections). Skills and agents are already
//! composed via the compile path and are intentionally left untouched here to
//! avoid double-composition at runtime.
//!
//! Determinism: the `<input-hash>` is a hash of the local traveling-dir content
//! hashes plus the resolved dep hashes from `ship.lock`. Same inputs → same
//! composed dir → cacheable + reusable. If `composed/<hash>/` exists, it is
//! returned as-is (no rebuild).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use compiler::lockfile::ShipLock;
use compiler::manifest::ShipManifest;

use super::hash::compute_tree_hash;

/// The four non-library traveling types composed by slice 1.
///
/// `(manifest export selector, on-disk dir name, copy granularity)`. Skills and
/// agents are deliberately excluded — see the module docs.
const COMPOSED_TYPES: &[ComposedType] = &[
    ComposedType {
        dir: "views",
        granularity: Granularity::Dir,
    },
    ComposedType {
        dir: "pipelines",
        granularity: Granularity::File,
    },
    ComposedType {
        dir: "triggers",
        granularity: Granularity::File,
    },
    ComposedType {
        dir: "collections",
        granularity: Granularity::File,
    },
];

#[derive(Clone, Copy)]
enum Granularity {
    /// Export path points at a directory (e.g. a view's `.sui` + controller).
    Dir,
    /// Export path points at a single file (e.g. a pipeline `.jsonc`).
    File,
}

#[derive(Clone, Copy)]
struct ComposedType {
    /// Composed-root subdirectory name (also the export taxonomy key).
    dir: &'static str,
    granularity: Granularity,
}

/// Select a manifest export list for a composed type.
fn exports_for<'a>(
    manifest: &'a ShipManifest,
    dir: &str,
) -> &'a [String] {
    match dir {
        "views" => &manifest.exports.views,
        "pipelines" => &manifest.exports.pipelines,
        "triggers" => &manifest.exports.triggers,
        "collections" => &manifest.exports.collections,
        _ => &[],
    }
}

/// Resolve the default cache root (`~/.ship/cache/`) unless overridden.
fn resolve_cache_root(cache_root: Option<&Path>) -> Result<PathBuf> {
    match cache_root {
        Some(p) => Ok(p.to_path_buf()),
        None => {
            let home = home::home_dir().context("cannot determine home directory")?;
            Ok(ship_substrate::ShipHome::project_ship_dir(home).join("cache"))
        }
    }
}

/// The traveling-artifact root a loader should read for `ship_dir`.
///
/// - **Depless space** (no `ship.lock` or no packages): the raw `.ship/` itself
///   — composition is a no-op, so we skip materialization entirely. This is the
///   common case; it copies nothing and behaves exactly as before.
/// - **Has deps**: the materialized composed overlay (local base + dep exports).
///
/// Falls back to the raw `.ship/` on any materialization error — composition
/// must never break a load that worked before deps were introduced. Loaders for
/// the four non-library traveling types (views/pipelines/triggers/collections)
/// resolve their root through this instead of using `ship_dir` directly.
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
///
/// `ship_dir`   — the space's `.ship/` directory.
/// `cache_root` — `~/.ship/cache/` (pass `None` for the default).
pub fn materialize_space(ship_dir: &Path, cache_root: Option<&Path>) -> Result<PathBuf> {
    let cache_root = resolve_cache_root(cache_root)?;

    // Load the lockfile if present; a depless space composes from layer 0 only.
    let lock_path = ship_dir.join("ship.lock");
    let lock = if lock_path.exists() {
        ShipLock::from_file(&lock_path).with_context(|| format!("reading {}", lock_path.display()))?
    } else {
        ShipLock::default()
    };

    let input_hash = compute_input_hash(ship_dir, &lock)?;
    // The hash is `sha256:<hex>`; use the bare hex as the dir name (no colon).
    let hash_dir = input_hash.strip_prefix("sha256:").unwrap_or(&input_hash);
    let composed_root = cache_root.join("composed").join(hash_dir);

    // Short-circuit: identical inputs → existing dir, no rebuild.
    if composed_root.is_dir() {
        return Ok(composed_root);
    }

    // Build into a temp dir alongside composed/, then atomically rename.
    let composed_parent = cache_root.join("composed");
    std::fs::create_dir_all(&composed_parent).context("creating composed cache dir")?;

    let tmp_dir = composed_parent.join(format!(".tmp-{hash_dir}"));
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir).context("removing stale tmp composed dir")?;
    }
    std::fs::create_dir_all(&tmp_dir).context("creating tmp composed dir")?;

    build_overlay(ship_dir, &lock, &cache_root, &tmp_dir)?;

    // Rename into place. If a concurrent build won the race, drop our tmp.
    if composed_root.exists() {
        std::fs::remove_dir_all(&tmp_dir).ok();
    } else {
        std::fs::rename(&tmp_dir, &composed_root)
            .context("renaming tmp composed dir into place")?;
    }

    Ok(composed_root)
}

/// Compute the deterministic input-hash: local traveling-dir content hashes
/// concatenated with the resolved dep hashes from the lockfile.
fn compute_input_hash(ship_dir: &Path, lock: &ShipLock) -> Result<String> {
    let mut acc = String::new();

    // Layer 0: hash each local composed-type dir that exists (stable order).
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

    // Layers 1..N: the resolved dep hashes, sorted by package path for stability.
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

/// Build the composed overlay into `dest` (a fresh temp dir).
fn build_overlay(
    ship_dir: &Path,
    lock: &ShipLock,
    cache_root: &Path,
    dest: &Path,
) -> Result<()> {
    // Layer 0: copy local traveling dirs verbatim (no namespace prefix).
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

    // Track collection-schema names per composed-type dir to detect same-name
    // collisions across packages (hard error for collections per the spec).
    let mut seen_collections: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Layers 1..N: each locked dep's exports, namespaced by package path.
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
                        pkg.path,
                        ct.dir,
                        export_rel,
                        src.display()
                    );
                }

                let basename = Path::new(export_rel)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| export_rel.clone());

                // Collection-schema same-name collision across packages = error.
                if ct.dir == "collections" && !seen_collections.insert(basename.clone()) {
                    anyhow::bail!(
                        "collection-schema collision: two packages export a \
                         same-name collection schema '{}'",
                        basename
                    );
                }

                // Namespace under the full package path:
                //   <type>/<pkg.path>/<basename>
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

/// Recursively copy `src` into `dst` (dst must exist).
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

/// SHA-256 of `data`, returned as `"sha256:<lowercase-hex>"` for consistency
/// with the registry hashing helpers.
fn hex_sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let hex: String = result.iter().map(|b| format!("{b:02x}")).collect();
    format!("sha256:{hex}")
}

#[cfg(test)]
#[path = "compose_tests.rs"]
mod tests;
