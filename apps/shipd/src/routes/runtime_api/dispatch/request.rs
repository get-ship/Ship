//! Re-exports from ship-agents — DispatchRequest type moved to the domain crate.

pub use ship_agents::services::agents::dispatch_request::{
    DispatchRequest,
    billing_warning_for_axes,
    default_invocation,
    default_keepalive,
    default_open_tab,
    validate_axes,
};
