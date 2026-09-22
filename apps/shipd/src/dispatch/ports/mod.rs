//! L1 port implementations for bare agent-session dispatch.
//!
//! Formerly in `ship-pipeline-runtime` (renamed `pipeline-defs` for definition
//! CRUD only). Only shipd dispatch consumes these, so they live at the
//! composition root.

pub mod projection_reader;
pub mod spawn_policy;
