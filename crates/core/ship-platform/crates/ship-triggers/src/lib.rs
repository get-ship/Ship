//! Reactive trigger engine — descriptor-sourced triggers that match Ship
//! events and dispatch actions.
//!
//! Triggers arrive via the composed `ExtensionLibrary::triggers`
//! (`Vec<stack_model::TriggerDef>`), sourced from `defineTrigger` declare-forms
//! in the space's `ship.config.ts` (format-migration Wave 4 phase A —
//! the prior `.ship/triggers/*.jsonc` file format is retired, no dual
//! path; see `census-format-migration.md` §4 row 3). Each trigger has a
//! `trigger` (internal event type OR an external source) and a sequence
//! of `actions` rendered with minijinja against an `{event, workspace,
//! actor, results}` context.
//!
//! This module owns the descriptor→`HookDef` conversion (`def.rs`), the
//! executor (event → action dispatch), and the three external source
//! adapters (webhook, file, cron). There is no file-based hot-reload —
//! `HookEngine::reload` takes the current descriptor's trigger set
//! directly; the caller re-runs the roll-up and calls `reload` again
//! when the source changes.

pub mod actions;
pub mod def;
pub mod engine;
pub mod filter;
pub mod render;
pub mod sources;

pub use def::{HookAction, HookActionType, HookDef, HookTrigger};
pub use engine::{HookEngine, HookExecutionCtx};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
#[cfg(test)]
mod schema_tests;
