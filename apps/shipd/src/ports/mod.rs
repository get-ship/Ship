//! Port implementations — shipd-side adapters for L1 crate ports.
//!
//! Each module implements a port trait owned by a lower-layer crate
//! (deck's `DataPort`, ship-ui' `ViewToolPort`, ship-pipeline-runtime's
//! dispatch ports, ship-device's session ports, ship-agents' event
//! adapters), wiring it to `ShipRuntime` state.

pub mod agent_events;
pub mod data;
pub mod device;
// local_paths: ungated path helpers (worktrees_dir without ship-code dep).
// Used by agent_events + pipeline so they compile with flag-off.
pub mod local_paths;
pub mod pipeline;
pub mod view;
