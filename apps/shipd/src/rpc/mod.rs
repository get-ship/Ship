//! Connect RPC surface — per-domain wire impls and the ops router.
//!
//! `ops` is the composition root: it builds the `connectrpc` router, mounts
//! every service wire, and owns the per-op policy table (`policy_for`).
//! The sibling modules each carry one domain's wire impl (or, for `agent`,
//! the policy/gate conformance tests where the wire lives in `ship-agents`).

pub mod account;
// agent: policy + gate conformance tests for AgentsService.
// The impl AgentsService for OpsImpl is deleted — AgentsWire (ship-agents) is
// now the sole handler, mounted via AgentsServiceExt::register in `ops`.
pub mod agent;
pub mod events;
pub mod ops;
pub mod pipelines;
pub mod registry;
pub mod runtime;
pub mod search;
// search_context: shipd-side SearchContext impl — sole consumer is `search`.
pub mod search_context;
pub mod settings;
pub mod space;
