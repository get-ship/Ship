//! Artifacts — project-scoped blob storage with collection-backed metadata.
//!
//! Blobs live at `~/.ship/artifacts/{project-slug}/blobs/{id[0:2]}/{id}`.
//! Metadata is tracked in the built-in `artifacts` collection in `data.db`
//! so every artifact is queryable through the standard collections API.
//! The per-worktree `.artifacts/` directory is an ephemeral scratch space
//! and is NOT related to this store.

mod schema;
mod service;

pub use schema::{ARTIFACTS_COLLECTION_ID, artifacts_schema, ensure_artifacts_collection};
pub use service::{ArtifactRecord, ArtifactService};

#[cfg(test)]
mod tests;
