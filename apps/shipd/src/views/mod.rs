//! View discovery + manifest metadata.
//!
//! What remains of the daemon's view layer after the UI purge (deck owns all
//! serving): the search-dir walker (`discovery`) and manifest reader
//! (`resolve`) that back the search index (`rpc::search_context`).
//!
//! **Phase-4 J2 (STOP-AND-SURFACE, 2026-07-06):** the dump-bucket
//! `ViewManifest` / `ViewListEntry` / `RailMeta` types are removed — no live
//! consumer outside `apps/shipd/src/views/**` (verified across the repo).
//! The remaining `discovery` + `resolve` modules HAVE a live consumer
//! (`rpc::search_context::views()`), so they're NOT deleted here: the spec
//! explicitly says "STOP and surface (do not redesign — deck owns consumption)".
//! Future work: deck (the views-feature owner per
//! `decision-views-deck-boundary.md`) re-derives a view source; until then
//! the daemon's `views()` returns the empty `view_search_dirs()` walk.
//!
//! Deleted as prior purge residue: the agent-canvas `ViewSessionRegistry`
//! (`session`/`session_types`), the HTML sanitizer it fed (`sanitize`),
//! and the `state` re-export shim — no surface could create or watch a
//! session once `view.open` and the SSE subscribe route were cut.

pub mod discovery;
pub mod resolve;
