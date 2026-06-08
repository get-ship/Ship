//! Git — an L0 library wrapping `git2`: repository status, diffs, reads, writes.
//! Consumed in-process by pipelines, agents, and the CLI. Not an RPC service.

pub mod diff;
pub mod read;
pub mod state;
pub mod write;

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
