//! SUI SSE patch encoders — re-exported from `deck_core::live_wire`.
//!
//! The encoder bodies live in `deck_core::live_wire` (the single source of
//! truth). This module re-exports them so existing `super::sui_patches::*`
//! import paths in this crate continue to resolve without change.
//!
//! Ship-specific glue (e.g. type aliases, helper wrappers) belongs here if
//! it ever grows; today there is none.

pub use deck_core::live_wire::{
    PatchElements, PatchElementsMode,
    format_keepalive, format_patch_elements, format_patch_signals,
};
