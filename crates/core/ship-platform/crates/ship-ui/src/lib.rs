//! ship-ui — the unified UI authoring service (L1).
//!
//! One crate owns the full authoring surface: component discovery (Packs,
//! Read), SUI compiler analysis (Diagnostics, Hover, Dependencies), and view
//! authoring writes (PatchSession, AppendSession, SignalSession, EditMode,
//! Tool, Scaffold). Replaces the retired `ship-views` + `SuiService` pair.
//!
//! Depends only on substrate ports and the collection/artifact services
//! — no reach back into ship-platform.

pub mod registry;
pub mod themes_contrast;
pub mod view_authoring;
pub mod view_errors;
pub mod view_ports;
pub mod wire;
