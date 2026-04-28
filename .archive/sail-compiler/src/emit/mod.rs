//! Phases 7-10 — emission. Lowered IR → JS module + SSR template +
//! TypeScript declaration + JSON schema.
//!
//! Each module here owns one of the four outputs:
//! - `ssr.rs` — server-renderable HTML/template (phase 7)
//! - `runtime.rs` — ES module with custom-element class (phase 8)
//! - `types.rs` — TypeScript .d.ts (phase 9)
//! - `schema.rs` — JSON schema (phase 10)

pub mod ssr;
pub mod runtime;
pub mod types;
pub mod schema;
