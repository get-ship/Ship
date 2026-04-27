//! `CollectionsService` — eighth mr-02 migration. The first db-bound
//! service: the impl owns an `Arc<Mutex<Option<DataService>>>` cloned
//! from `ShipRuntime` rather than a `FilesService`. HTTP handlers stay
//! in `apps/shipd/src/collection_api.rs` for now (multipart / scope
//! gating / utoipa annotations are transport-shaped); they delegate
//! the actual work through this trait.

pub mod service;
