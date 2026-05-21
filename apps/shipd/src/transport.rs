//! Transport layer — axum / network plumbing for the daemon.
//!
//! Modular-runtime-04 PR 2: collects cors, pty, server, connections under
//! one umbrella. Routes consume these but do not reach into network
//! internals; transport stays independent of services / routes.

pub mod connections;
pub mod cors;
pub mod pty;
