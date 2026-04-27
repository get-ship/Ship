//! `ArtifactsService` — ninth mr-02 migration. Db-bound like
//! `CollectionsService`; layered over the runtime `ArtifactService`.
//! HTTP handlers stay in `apps/shipd/src/artifacts_api.rs` because of
//! the multipart-upload + content-disposition + scope-gating shape of
//! the surface; they delegate the data work through this trait.

pub mod service;
