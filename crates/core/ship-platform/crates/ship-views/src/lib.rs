//! ship-views — view authoring service (L1).
//!
//! Carved out of `ship-platform` (C5). Owns the agent-facing `.sui` view edit
//! surface: source read/write, snapshot + restore, annotate, and the in-memory
//! edit-mode capability registry. Depends only on substrate ports and the
//! collection/artifact services — no reach back into ship-platform.

pub mod sdk_manifest;
pub mod themes_contrast;
pub mod view_authoring;
pub mod view_errors;
pub mod view_ports;
pub mod wire;
