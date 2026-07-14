//! Library install pipeline — registry-resolved bundles → unpacked stack roots.
//!
//! The shape (per the install-lifecycle job spec):
//!
//! 1. [`crate::registry::Registry::resolve_version`] — constraint → concrete
//!    `version` + the registry-stated content hash.
//! 2. [`crate::registry::Registry::fetch_bundle`] — `path` + `version` → the
//!    tarball bytes + publisher attestation.
//! 3. [`crate::install_bundle::extract_and_verify_bundle`] — extract the
//!    gzip-tarball into an isolated temp dir, hash the extracted tree, verify
//!    the attestation binds that hash. The verify gate; the temp dir is wiped
//!    on any failure.
//! 4. Move the verified tree into the managed library dir under
//!    `<ship_home>/library/<url-encoded-path>/<version>/`. Re-running with the
//!    same `(path, version)` is idempotent — a `library/<encoded-path>/<version>/`
//!    that already exists at the same hash short-circuits.
//!
//! The function returns [`InstalledStack`] — the on-disk location the
//! unpacked stack landed at, the resolved version, and the canonical content
//! hash. The caller (the `SpaceService.InstallStack` RPC) emits the
//! `stack.installed` event with these fields and the runtime folds it into
//! the `library` projection; the `space_stacks` projection needs a
//! follow-up `stack.registered` to add the unpacked root to membership.
//!
//! Library layout: `<ship_home>/library/<url-encoded-path>/<version>/`.
//! The path segment is percent-encoded so `/` etc. are filesystem-safe —
//! mirrors the cache's `<root>/objects/<sha>/` and `<root>/index/<path>@<ver>`
//! convention.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use crate::url_encode::percent_encode;

use super::bundle_sig::TrustedKey;
use super::install_bundle::{extract_and_verify_bundle, InstallBundleError};
use super::registry::Registry;

/// A successfully-installed stack: where it landed, the resolved version, and
/// the canonical content hash (matches the registry attestation AND the cache
/// key — the SINGLE hash that flows registry ↔ attestation ↔ cache ↔ disk).
#[derive(Debug, Clone)]
pub struct InstalledStack {
    /// Absolute path to the unpacked stack root on this device.
    pub root_path: PathBuf,
    /// Resolved version (the registry-stated exact version).
    pub version: String,
    /// Canonical tree hash, e.g. `"sha256:<hex>"`.
    pub content_hash: String,
    /// Registry-issued package path (the identity the install was keyed on).
    pub path: String,
}

/// Errors the install pipeline can return. Surfaced verbatim to the wire
/// layer for `BadArgs` (empty path / version), `NotFound` (registry 404),
/// and `Internal` (extraction / verify / unpack failures).
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("bad args: {0}")]
    BadArgs(String),
    #[error("registry unavailable: {0}")]
    Registry(String),
    #[error("bundle verification failed: {0}")]
    Verify(String),
    #[error("internal: {0}")]
    Internal(String),
}

/// Resolve the managed library dir for `path` + `version` under the supplied
/// ship home's data dir. The path segment is percent-encoded so a path with
/// slashes (the registry-issued `github.com/owner/repo` shape) stays a single
/// dir component.
pub fn library_root(ship_home_data: &Path, path: &str, version: &str) -> PathBuf {
    ship_home_data
        .join("library")
        .join(percent_encode(path))
        .join(percent_encode(version))
}

/// Install a stack from the registry into the managed library dir.
///
/// `library_root_parent` is the directory under which
/// `<library_root_parent>/library/<path>/<version>/` is materialised —
/// typically `<ship_home>/data`. `registry` does the resolve+fetch;
/// `trusted_keys` are passed to the attestation verifier.
///
/// Idempotent: if the library row already exists at the SAME canonical hash,
/// returns it without re-downloading or re-extracting.
pub async fn install_stack<R: Registry + ?Sized>(
    registry: &R,
    library_root_parent: &Path,
    path: &str,
    version_constraint: &str,
    trusted_keys: &[TrustedKey],
) -> Result<InstalledStack, InstallError> {
    if path.is_empty() {
        return Err(InstallError::BadArgs("path is required".into()));
    }
    let constraint = if version_constraint.is_empty() {
        "*"
    } else {
        version_constraint
    };

    // 1) Resolve constraint → version + content hash.
    let resolved = registry
        .resolve_version(path, constraint)
        .await
        .map_err(|e| {
            InstallError::Registry(format!("resolve_version({path}, {constraint}): {e}"))
        })?;

    // 2) Compute the destination.
    let dest = library_root(library_root_parent, path, &resolved.version);

    // 3) Idempotency: existing library row at the same hash → return as-is.
    if dest.exists() {
        if let Ok(existing_hash) = crate::hash::compute_tree_hash(&dest) {
            if existing_hash == resolved.sha256 {
                return Ok(InstalledStack {
                    root_path: dest,
                    version: resolved.version,
                    content_hash: resolved.sha256,
                    path: path.to_string(),
                });
            }
        }
    }

    // 4) Fetch the bundle bytes + attestation.
    let fetched = registry.fetch_bundle(path, &resolved.version).await.map_err(|e| {
        InstallError::Registry(format!(
            "fetch_bundle({path}, {}): {e}",
            resolved.version
        ))
    })?;

    // 5) Extract + verify (extract → compute_tree_hash → verify attestation).
    //    The verify step requires the registry-stated content hash to equal
    //    the extracted tree hash; we fail closed on any mismatch.
    let tmp = tempfile::tempdir()
        .map_err(|e| InstallError::Internal(format!("tempdir: {e}")))?;
    let claim = match extract_and_verify_bundle(
        &fetched.bytes,
        &fetched.attestation_jwt,
        trusted_keys,
        tmp.path(),
    ) {
        Ok(claim) => claim,
        Err(InstallBundleError::Verify(e)) => return Err(InstallError::Verify(e.to_string())),
        Err(e) => return Err(InstallError::Internal(e.to_string())),
    };

    // Belt-and-braces: the extracted tree hash MUST equal the registry-stated
    // content hash (the attestation binds the publisher's claim, but the
    // registry's `ResolveVersion.sha256` is the gate the install pipeline
    // trusts). If they ever diverge we treat it as a registry inconsistency
    // — surface to the operator, refuse to install.
    let extracted_hash = crate::hash::compute_tree_hash(tmp.path())
        .map_err(|e| InstallError::Internal(format!("hashing extracted bundle: {e}")))?;
    if extracted_hash != fetched.sha256 {
        return Err(InstallError::Internal(format!(
            "registry/content mismatch: registry says {fetched_sha}, extracted tree is {extracted_hash} (publisher claim: {claim_sha})",
            fetched_sha = fetched.sha256,
            claim_sha = claim.sha256,
        )));
    }

    // 6) Move the verified tree into place under the managed library dir.
    if dest.exists() {
        // Stale row at a different hash — wipe and replace. The projection
        // events have already been folded for the prior hash, so a stale
        // `library` row pointing here would also need dropping; we leave
        // that to the caller (the `space_stacks` projection is keyed by the
        // root path, not the hash).
        std::fs::remove_dir_all(&dest)
            .map_err(|e| InstallError::Internal(format!("clearing stale library row: {e}")))?;
    }
    let parent = dest
        .parent()
        .ok_or_else(|| InstallError::Internal("library root has no parent".into()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| InstallError::Internal(format!("creating library parent: {e}")))?;
    // Atomic-ish move: rename works across the same filesystem; the temp
    // dir's lifetime guarantees we hold the only reference.
    std::fs::rename(tmp.path(), &dest)
        .with_context(|| format!("moving {} → {}", tmp.path().display(), dest.display()))
        .map_err(|e| InstallError::Internal(e.to_string()))?;

    Ok(InstalledStack {
        root_path: dest,
        version: resolved.version,
        content_hash: fetched.sha256,
        path: path.to_string(),
    })
}

/// Remove an installed stack from the managed library dir. Best-effort —
/// missing directories are a no-op (idempotent uninstall). The caller is
/// expected to have already emitted `stack.unregistered` (which the
/// `library` projection folds to remove the row) and unregistered the
/// stack from the space's membership.
pub fn remove_installed(
    library_root_parent: &Path,
    path: &str,
    version: &str,
) -> Result<()> {
    let dest = library_root(library_root_parent, path, version);
    if !dest.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(&dest)
        .with_context(|| format!("removing {}", dest.display()))?;
    // Best-effort cleanup of empty parents: `<library>/<encoded-path>/`.
    if let Some(parent) = dest.parent() {
        let lib_dir = library_root_parent.join("library");
        if parent != lib_dir
            && parent
                .read_dir()
                .map(|mut i| i.next().is_none())
                .unwrap_or(false)
        {
            let _ = std::fs::remove_dir(parent);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_root_url_encodes_path_segment() {
        let dir = library_root(Path::new("/data"), "github.com/owner/repo", "1.2.3");
        assert_eq!(
            dir,
            PathBuf::from("/data/library/github.com%2Fowner%2Frepo/1.2.3")
        );
    }

    #[test]
    fn library_root_safe_for_plain_scoped_path() {
        let dir = library_root(Path::new("/data"), "@scope/name", "0.0.1");
        assert_eq!(dir, PathBuf::from("/data/library/%40scope%2Fname/0.0.1"));
    }

    #[test]
    fn remove_installed_is_idempotent_for_missing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        // Never installed — must be a no-op, not an error.
        remove_installed(tmp.path(), "github.com/x/y", "1.0.0").unwrap();
    }
}
