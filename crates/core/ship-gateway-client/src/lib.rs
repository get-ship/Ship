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
// it in the matching Rust module tree. The generated code trips a few
// stylistic clippy lints (match_single_binding from default impls,
// large stack arrays from the file descriptor blobs); allow on the
// module so we don't have to monkey-patch the generator output.
#[allow(
    clippy::match_single_binding,
    clippy::large_stack_arrays,
    clippy::needless_borrow,
    clippy::useless_conversion,
    clippy::redundant_clone,
)]
pub mod ship {
    pub mod gateway {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/ship.gateway.v1.mod.rs"));
        }
    }
}

pub use ship::gateway::v1::*;
