//! Artifacts — three-scope blob store with collection-backed metadata.
//!
//! Scopes:
//!   run     — ephemeral, TTL-based, per pipeline run
//!   project — persistent, project-level
//!   user    — persistent, user-personal
//!
//! Blobs are addressed by ULID and stored at scope-specific paths under
//! `~/.ship/artifacts/`. Metadata is tracked in the built-in `artifacts`
//! collection in `data.db` and queryable through the standard collections API.

pub mod backend;
pub mod gc;
mod schema;
mod service;

pub use backend::{ArtifactBackend, ArtifactScope, LocalFsBackend, sha256_hex};
pub use schema::{ARTIFACTS_COLLECTION_ID, artifacts_schema, ensure_artifacts_collection};
pub use service::{ArtifactRecord, ArtifactService};

#[cfg(test)]
mod tests;
