//! Specs view shaping — pure helpers used by the `/api/v1/specs/*`
//! HTTP routes and the `specs.*` registry ops.
//!
//! `shape` holds the typed `Spec` projection of a `store::Item` plus
//! body-file helpers. `parse` parses markdown bodies into the section
//! map the editor pane consumes. `view_state` builds rail / detail
//! contexts the Studio specs view renders. Storage and CRUD live in
//! `CollectionsService` (specs are a first-party collection).

pub mod parse;
pub mod shape;
pub mod view_state;
