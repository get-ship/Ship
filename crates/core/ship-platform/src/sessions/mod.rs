//! Session lifecycle — re-exported from `ship-agents` (C8b fold).
//!
//! The complete session domain (compile bridge + lifecycle + runtime-session
//! types/helpers/hydration/artifacts) now lives in the `ship-agents` crate.
//! This shim keeps the historic `ship_platform::sessions::*` paths resolving
//! for shipd consumers (ship_runtime/{mod,init,accessors,test_ctors}.rs,
//! diag_rpc.rs, services/actor_control.rs) and the xtask TS type-gen.

pub use ship_agents::sessions::{
    get_active_session, helpers, list_sessions, record_session_progress, types_session,
    EndSessionRequest, Session, SessionStatus,
    end_session, start_session,
};
