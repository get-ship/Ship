//! Ship registry v0.1 — dependency resolver and content-addressed package cache.
//!
//! Public API surface for callers:
//!
//! - [`install::resolve_and_fetch`] — top-level orchestration.
//! - [`cache::PackageCache`] — content-addressed cache (`~/.ship/cache/`).
//! - [`constraint::parse_constraint`] / [`constraint::VersionConstraint`] — constraint parsing.
//! - [`resolver::resolve_version`] — git tag → exact commit resolution.
//! - [`hash::compute_tree_hash`] / [`hash::compute_file_hash`] — deterministic SHA-256 hashing.
//! - [`types`] — minimal `ShipManifest`/`Dependency` view for the install
//!   pipeline; the full lockfile types live in `stack_model::lockfile`.

pub mod bundle_sig;
pub mod cache;
pub mod collection_schemas;
pub mod compose;
pub mod constraint;
pub mod dep_skills;
pub mod fetch;
pub mod hash;
pub mod init_deps;
pub mod install;
pub mod install_bundle;
mod install_helpers;
pub mod resolver;
pub mod tracking;
pub mod url_encode;
pub mod types;
pub mod verify;

pub use cache::{CachedPackage, PackageCache};
pub use compose::materialize_space;
pub use constraint::{VersionConstraint, parse_constraint};
pub use hash::{ExportHashes, compute_export_hashes, compute_file_hash, compute_tree_hash};
pub use init_deps::{
    DEFAULT_INIT_DEPS, SeedResult, seed_default_dependencies, try_install_init_deps,
};
pub use install::{InstallOptions, InstallResult, resolve_and_fetch};
pub use resolver::{ResolvedVersion, resolve_alias, resolve_version};
pub use types::{Dependency, ShipManifest, SyncStatus, diff_manifest_against_lock};
// Canonical lockfile types live in `stack_model::lockfile`; re-export the names
// callers expect at this crate path.
pub use stack_model::lockfile::{LockPackage, ShipLock};

#[cfg(test)]
mod tests;
