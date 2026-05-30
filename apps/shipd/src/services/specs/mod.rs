//! Specs read shaping — `shape::Spec` projects a `store::Item` for the
//! `specs.list` / `specs.get` registry ops. Storage and CRUD live in
//! `CollectionsService` (specs are a first-party collection).

pub mod shape;
