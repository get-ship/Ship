//! Ship registry v0.1 — content-addressed package cache and resolver.
//!
//! Public API surface for callers:
//!
//! - [`cache::PackageCache`] — content-addressed cache (`~/.ship/cache/`).
//! - [`constraint::parse_constraint`] / [`constraint::VersionConstraint`] — constraint parsing.
//! - [`resolver::resolve_version`] — git tag → exact commit resolution.
//! - [`hash::compute_tree_hash`] / [`hash::compute_file_hash`] — deterministic SHA-256 hashing.

pub mod bundle_sig;
pub mod cache;
pub mod collection_schemas;
pub mod compose;
pub mod constraint;
pub mod dep_skills;
pub mod hash;
pub mod install_bundle;
pub mod resolver;
pub mod url_encode;
pub mod verify;

pub use cache::{CachedPackage, PackageCache};
pub use compose::materialize_space;
pub use constraint::{VersionConstraint, parse_constraint};
pub use hash::{ExportHashes, compute_export_hashes, compute_file_hash, compute_tree_hash};
pub use resolver::{ResolvedVersion, resolve_alias, resolve_version};
// Canonical lockfile types live in `compiler::lockfile`; re-export the names
// callers expect at this crate path.
pub use compiler::lockfile::{LockPackage, ShipLock};

#[cfg(test)]
mod tests;
