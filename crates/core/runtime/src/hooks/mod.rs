//! Reactive hook engine — JSONC hooks that match Ship events and dispatch actions.
//!
//! Hooks live in `.ship/hooks/*.jsonc`. Each hook has a `trigger` (internal
//! event type + optional dot-path filter OR an external source) and a sequence
//! of `actions` rendered with minijinja against an `{event, workspace, actor,
//! results}` context.
//!
//! This module owns loading (JSONC → `HookDef`), the executor (event → action
//! dispatch), a small hot-reload watcher keyed off the hooks directory, and
//! the three external source adapters (webhook, file, cron).

pub mod actions;
pub mod def;
pub mod engine;
pub mod filter;
pub mod loader;
pub mod render;
pub mod sources;
pub mod watcher;

pub use def::{HookAction, HookActionType, HookDef, HookTrigger};
pub use engine::{HookEngine, HookExecutionCtx};
pub use loader::{LoadError, LoadReport, load_hooks_dir};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod schema_tests;
