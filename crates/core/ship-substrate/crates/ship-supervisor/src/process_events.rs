//! Process lifecycle event emitters — re-exports from the L0 runner.
//!
//! The canonical emit helpers live in `ship_runner::emit` (the L0 runner
//! mechanism). This module re-exports for convenience in shipd code that needs
//! to emit process events (e.g. liveness sweep, shutdown).

#[allow(unused_imports)]
pub use ship_runner::emit::{
    emit_process_exited, emit_process_signal_sent, emit_process_spawned,
};
