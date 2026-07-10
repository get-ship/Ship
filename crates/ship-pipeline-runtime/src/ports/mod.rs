//! L1 port implementations for the pipeline runtime.
//!
//! Lifted out of the (now-deleted) `orchestrator/` module so the bare
//! agent-session wiring in `apps/shipd::dispatch::run_dispatch` and
//! `apps/helmd::dispatch` can keep its shared `SpawnPolicy` +
//! `ProjectionReader` instances without depending on the orchestrator
//! runtime. The runner mechanism is gone; these ports are the seam the
//! bare-agent dispatch path still threads through.

pub mod projection_reader;
pub mod spawn_policy;
