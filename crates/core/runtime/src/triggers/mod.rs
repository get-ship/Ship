//! Reactive trigger engine — JSONC definitions that match Ship events and dispatch actions.
//!
//! Triggers live in `.ship/triggers/*.jsonc`. Each trigger has a `trigger` clause
//! (event type + optional dot-path filter on payload) and a sequence of `actions`
//! rendered with minijinja against an `{event, workspace, actor, results}` context.
//!
//! Distinct from *provider hooks* — those are lifecycle hooks the chat CLI
//! (claude/codex/gemini) runs on its own events (PreToolUse, Stop, etc.) and
//! live under `.ship/agents/<agent>/hooks.jsonc`. Triggers are runtime-side;
//! provider hooks are chat-CLI-side.
//!
//! This module owns loading (JSONC → `TriggerDef`), the executor (event → action
//! dispatch), and a small hot-reload watcher keyed off the triggers directory.

pub mod actions;
pub mod def;
pub mod engine;
pub mod filter;
pub mod loader;
pub mod render;
pub mod watcher;

pub use def::{ActionKind, TriggerAction, TriggerDef, TriggerMatch};
pub use engine::{TriggerEngine, TriggerExecutionCtx};
pub use loader::{LoadError, LoadReport, load_triggers_dir};

#[cfg(test)]
mod tests;
