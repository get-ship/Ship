//! `compiler` — agent-config compiler (L0 pure logic, no ship runtime deps).
//!
//! Resolves a project's agent library (`resolve`), compiles it into per-harness
//! output files (`compile`), and decompiles existing harness configs back to
//! Ship's format (`decompile`). Has no dependency on `ship-substrate` or any
//! runtime — safe to use in CLI tools and the daemon alike.
//!
//! Key modules: `types` (all config types), `compile` (per-harness codegen),
//! `decompile` (harness→ship reverse), `resolve` (library resolution),
//! `schemas` (harness managed-key sets + schema URLs).

pub mod compile;
pub mod decompile;
pub mod events;
pub mod jsonc;
pub mod lockfile;
pub mod manifest;
pub mod output;
pub mod resolve;
pub mod schemas;
pub mod types;
pub mod vars;

// ─── Top-level re-exports ─────────────────────────────────────────────────────

pub use output::{ensure_parent, merge_json, merge_json_file, write_output};

pub use compile::{
    AgentsDir, CURSOR_PERMISSIVE_ALLOW, CompileOutput, ContextFile, McpKey, HarnessDescriptor,
    ProviderFeatureFlags, SkillsDir, agents::compile_agent_profiles, build_claude_settings_patch,
    compile, get_harness, list_harnesses, translate_to_cursor_permission,
};
pub use decompile::{
    DetectedProviders, decompile_all, decompile_claude, decompile_codex, decompile_cursor,
    decompile_opencode, detect_harnesses,
};
pub use resolve::{ProjectLibrary, ResolvedConfig, WorkspaceOverrides, resolve, resolve_library};
pub use schemas::{HARNESS_MANAGED_KEYS, HARNESS_SCHEMAS, managed_keys, schema_url};
pub use types::{
    AgentBundle, AgentLayerConfig, AgentLimits, AgentProfile, ArtifactType, BlastRadius, Capability,
    CatalogCategory, CatalogEntry, CatalogKind, CommandPermissions, FsPermissions,
    HookConfig, HookTrigger, ListAgentsResponse, McpRefs, McpServerConfig, McpServerType,
    AgentConfig, NamespaceConfig, NetworkPermissions, NetworkPolicy, PermissionConfig, Permissions,
    PluginEntry, PluginRefs, PluginsManifest, ProfileMeta, ProfilePermissions, ProfileRules,
    ProjectConfig, PullAgent, PullMcpServer, PullProfile, PullResponse, PullRule, PullSkill, Rule,
    Skill, SkillBundle, SkillRefs, SkillSource, ToolPermissions, TransferBundle,
    list_catalog, list_catalog_by_kind, search_catalog,
};

/// Generate a nanoid using Ship's 56-character alphabet (no ambiguous chars).
pub fn gen_nanoid() -> String {
    let alphabet: [char; 56] = [
        '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K',
        'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd',
        'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
        'y', 'z',
    ];
    nanoid::format(nanoid::rngs::default, &alphabet, 8)
}

#[cfg(test)]
#[path = "schema_drift_tests.rs"]
mod schema_drift_tests;
