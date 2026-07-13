//! `harness-config-compiler` — the third-party-harness config compiler.
//!
//! Emits harness-native config trees (`compile`: .claude/, codex,
//! opencode, pi) and reads existing harness configs back into Ship's
//! format (`decompile`). Depends on `stack-model` for every config type
//! and the resolved library shape; the model never knows this crate
//! exists.
//!
//! Boundary: ship-code's world. Floki runs agents natively and never
//! emits — lint_layers R3 holds floki and this crate apart; the standard
//! shipd graph must not carry it (lint-boundary).
pub mod compile;
pub mod decompile;
pub mod output;
pub mod schemas;

pub use compile::{
    AgentsDir, CompileOutput, ContextFile, HarnessDescriptor, McpKey, ProviderFeatureFlags,
    SkillsDir, agents::compile_agent_profiles, build_claude_settings_patch, compile, get_harness,
    list_harnesses,
};
pub use decompile::{
    DetectedProviders, decompile_all, decompile_claude, decompile_codex, decompile_opencode,
    detect_harnesses,
};
pub use output::{ensure_parent, write_output};
pub use schemas::{HARNESS_MANAGED_KEYS, HARNESS_SCHEMAS, managed_keys, schema_url};

#[cfg(test)]
#[path = "schema_drift_tests.rs"]
mod schema_drift_tests;
