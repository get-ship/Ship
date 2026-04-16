//! Process lifecycle event emitters — re-exports from the runtime.
//!
//! The canonical emit helpers live in `runtime::services::orchestrator::emit`
//! because the dispatch path (where spawned/exited events originate) is in
//! the runtime crate. This module re-exports for convenience in shipd code
//! that needs to emit process events (e.g. liveness sweep, shutdown).

#[allow(unused_imports)]
pub use runtime::services::orchestrator::emit::{
    emit_process_exited, emit_process_signal_sent, emit_process_spawned,
};
