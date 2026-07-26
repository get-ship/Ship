//! Stream-synth observation helpers — formerly emitted through the global
//! `KernelRouter`. That was the layering smell: harness stream parsing is
//! not a host event bus.
//!
//! Call sites now go through [`crate::lifecycle::LifecycleSink`] directly
//! (`op_tool_use_started` / `turn_ended`). This module is kept as a thin
//! re-export/docs anchor so older references can find the new path.

#![allow(dead_code)]

// Intentionally empty of emit logic. Host implements LifecycleSink.
