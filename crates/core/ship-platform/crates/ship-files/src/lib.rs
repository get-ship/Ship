//! `ship-files` — L1 filesystem helpers carved from ship-platform/services.
//!
//! Provides [`files::FilesService`], JSONC utilities, path-tolerant RFC 6902
//! support, and a bounded [`line_accumulator::LineAccumulator`]. Depends only
//! on `ship-substrate` (L0) and std — no ship-platform dependency.

pub mod files;
pub mod jsonc;
pub mod line_accumulator;
pub mod path_tolerant;
