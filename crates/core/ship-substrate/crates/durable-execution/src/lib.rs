//! `durable-execution` (L0) — the journaling engine for durable program execution.
//!
//! Runs a *program* (a DAG of ops; an agent turn or a pipeline is the intended
//! production shape — live wiring is complete (floki Stage-2 shipped; Stage-3
//! deferrals tracked in durable_loop.rs))
//! and journals every op outcome. Because the journal is the source of truth, a
//! program can **suspend** (a human checkpoint, an approval, an await) and later
//! **resume** by *replaying* the journal: already-completed ops — including model
//! calls — return their logged results instead of re-executing. Replay
//! reconstructs state without re-running effects or re-burning tokens.
//!
//! Three surfaces:
//! - [`engine`] — the journal, [`engine::ExecCtx`] (op / suspend / parallel /
//!   retry / record), [`engine::Program`], [`engine::execute`]/[`engine::resume`],
//!   and [`engine::JournalingDispatch`] (the bridge that makes an externally
//!   driven op stream — e.g. a ship-js script — durable).
//! - [`Dispatch`] — the op-execution seam, **shared with `ship-js`** (the engine
//!   consumes ship-js, so there is ONE contract, not a shadow). The host captures
//!   auth/gate/transport inside its impl, so the engine never names them.
//! - [`projection`] — the model's context as a disposable, re-derivable *view* of
//!   the journal; compaction is non-destructive rendering.

pub mod engine;
pub mod projection;
pub mod store;

/// The op-execution contract, re-exported from `ship-js` — the engine and the JS
/// runtime share ONE seam. The host implements it over its gateway + credential.
pub use ship_js::{Dispatch, DispatchError};
pub use engine::{
    aggregate, execute, resume, AggregateDecision, EngineError, ExecCtx, Journal, JournalEntry,
    JournalingDispatch, OpRecord, Program, RunStatus,
};
pub use projection::{ContextItem, ContextProjector, KeepRecent, Verbatim};
pub use store::{InMemoryJournalStore, JournalStore, JournalStoreError};
