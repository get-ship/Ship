//! Phase 2 — resolve `data` declarations against schema sources.
//!
//! Walks each `data x = expr` declaration, calls `SchemaSource::resolve_data`
//! on the configured sources (first match wins), records the typed result.
//! Errors are reported as diagnostics.
//!
//! See `.artifacts/specs/sail/05-server-data-bridge.md`.
