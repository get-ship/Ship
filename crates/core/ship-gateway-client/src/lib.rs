//! Typed Connect-RPC client for the Ship gateway.
//!
//! Generated from `proto/ship/gateway/v1/*.proto` via `connectrpc-build`
//! in `build.rs`. The gateway (apps/gateway/) is the corresponding TS
//! server, generated from the same protos via buf — single source of
//! truth, two consumers.
//!
//! Convention: callers build a [`Client`] once (it wraps a reqwest pool +
//! a base URL) and reuse it. Each generated service has its own client
//! constructor — e.g. `AuthServiceClient::new(client, config)`.
//!
//! Default gateway base URL is `https://api.getship.dev`. Override for
//! staging / local dev via the `ClientConfig::new()` argument.

pub use connectrpc::client::{ClientConfig, HttpClient};
pub use connectrpc::ConnectError;

// Generated modules — one per proto package. connectrpc-build emits a
// `<package>.mod.rs` aggregator that includes per-file content; we wrap
// it in the matching Rust module tree.
pub mod ship {
    pub mod gateway {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/ship.gateway.v1.mod.rs"));
        }
    }
}

pub use ship::gateway::v1::*;
