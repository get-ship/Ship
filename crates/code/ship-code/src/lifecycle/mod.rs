//! Lifecycle mechanism — the harness-driver-specific orchestration helpers.
//!
//! The L0 entry gate (`dedup`, `router`, `DISPATCH_GATE`, `terminal_guard`)
//! lives in [`ship_substrate::dispatch_gate`]
//! (`crates/core/ship-substrate/src/dispatch_gate/`) so both
//! `ship-agents`'s actor and `ship-pipeline-runtime`'s orchestrator
//! can depend on it. The carrot — `observe` (git evidence + agent-log
//! tail + ANSI strip) — moved to `ship_pipeline_runtime::loader::git_helpers`
//! and `ship_substrate::text` during the seam eviction; only the harness
//! concerns remain:
//!
//! - `exit_handler` — translates harness `ExitSignal`s (claude/codex/etc.)
//! - `teardown` — kill-by-slug/process-handle bookkeeping
//! - `spawn_env` — wraps `SpawnPolicy` with the harness-only `Authority::Agent`
//! - `invocation` — step-mode resolution (headless/server/keepalive matrix)
//!
//! ALL of these depend on `ship-code` types (port traits, harness specifics),
//! so they stay L1 here.

pub mod exit_handler;
pub mod invocation;
pub mod spawn_env;
pub mod teardown;

pub use exit_handler::ExitVocabulary;
pub use invocation::resolve_step_invocation;
pub use teardown::{stop_by_slug, stop_handle};
pub use spawn_env::build_spawn_env;
