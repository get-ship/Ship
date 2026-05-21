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

#[cfg(not(target_arch = "wasm32"))]
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
    AgentBundle, AgentLayerConfig, AgentLimits, AgentProfile, ArtifactType,
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

// ─── WASM bindings ────────────────────────────────────────────────────────────

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
mod wasm {
    use wasm_bindgen::prelude::*;

    use crate::{ProjectLibrary, compile, resolve_library};

    /// The JSON-serialisable result returned to JS callers.
    #[derive(serde::Serialize)]
    struct CompileResult {
        /// Harness id that was compiled (e.g. "claude", "gemini", "codex", "cursor").
        harness: String,
        /// Context file content (CLAUDE.md / GEMINI.md / AGENTS.md), if any.
        context_content: Option<String>,
        /// MCP servers object — ready to merge into the harness's config file.
        mcp_servers: serde_json::Value,
        /// Project-relative path where the MCP config file should be written.
        /// e.g. `".mcp.json"` (Claude) or `".cursor/mcp.json"` (Cursor).
        mcp_config_path: Option<String>,
        /// Skill files: relative path → file content.
        skill_files: std::collections::HashMap<String, String>,
        /// Per-file rule output for harnesses that use individual rule files.
        /// Populated for Cursor (`.cursor/rules/<name>.mdc`). Empty for other harnesses.
        rule_files: std::collections::HashMap<String, String>,
        /// Claude-only: `permissions` + `hooks` patch for `.claude/settings.json`.
        claude_settings_patch: Option<serde_json::Value>,
        /// Codex-only: `[mcp_servers.*]` TOML tables for `.codex/config.toml`.
        codex_config_patch: Option<String>,
        /// Cursor-only: full `.cursor/hooks.json` content.
        cursor_hooks_patch: Option<serde_json::Value>,
        /// Cursor-only: `.cursor/cli.json` permissions (CLI-only, not IDE).
        cursor_cli_permissions: Option<serde_json::Value>,
        /// Cursor-only: `.cursor/environment.json` content.
        cursor_environment_json: Option<serde_json::Value>,
        /// Claude-only: `.claude/lsp.json` content.
        claude_lsp_config: Option<serde_json::Value>,
        /// OpenCode-only: full `opencode.json` content (model + MCP + extras).
        opencode_config_patch: Option<serde_json::Value>,
        /// Harness-native agent files: path → content.
        /// e.g. `.claude/agents/reviewer.md`, `.gemini/agents/reviewer.md`.
        agent_files: std::collections::HashMap<String, String>,
        /// Plugin install intent declared by the active preset.
        /// The CLI/runtime reads this to execute plugin installs — never the compiler.
        plugins_manifest: crate::PluginsManifest,
    }

    /// Compile a [`ProjectLibrary`] for a single harness.
    ///
    /// # Arguments
    /// - `library_json`  — JSON-serialised [`ProjectLibrary`].
    /// - `harness`      — Target harness id: `"claude"`, `"gemini"`, or `"codex"`.
    /// - `active_mode`   — Optional workspace mode override (e.g. `"planning"`).
    ///
    /// # Returns
    /// JSON string of [`CompileResult`], or a JS error string on failure.
    #[wasm_bindgen(js_name = compileLibrary)]
    pub fn compile_library(
        library_json: &str,
        harness: &str,
        active_agent: Option<String>,
    ) -> Result<String, JsValue> {
        let library: ProjectLibrary = serde_json::from_str(library_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid library JSON: {e}")))?;

        let resolved = resolve_library(&library, None, active_agent.as_deref());

        let output = compile(&resolved, harness)
            .ok_or_else(|| JsValue::from_str(&format!("Unknown harness: {harness}")))?;

        let result = CompileResult {
            harness: harness.to_string(),
            context_content: output.context_content,
            mcp_servers: output.mcp_servers,
            mcp_config_path: output.mcp_config_path,
            skill_files: output.skill_files,
            rule_files: output.rule_files,
            claude_settings_patch: output.claude_settings_patch,
            codex_config_patch: output.codex_config_patch,
            cursor_hooks_patch: output.cursor_hooks_patch,
            cursor_cli_permissions: output.cursor_cli_permissions,
            cursor_environment_json: output.cursor_environment_json,
            claude_lsp_config: output.claude_lsp_config,
            opencode_config_patch: output.opencode_config_patch,
            agent_files: output.agent_files,
            plugins_manifest: output.plugins_manifest,
        };

        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialisation error: {e}")))
    }

    /// Compile a [`ProjectLibrary`] for all harnesses in the resolved config.
    ///
    /// Returns a JSON object keyed by harness id → [`CompileResult`].
    #[wasm_bindgen(js_name = compileLibraryAll)]
    pub fn compile_library_all(
        library_json: &str,
        active_agent: Option<String>,
    ) -> Result<String, JsValue> {
        let library: ProjectLibrary = serde_json::from_str(library_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid library JSON: {e}")))?;

        let resolved = resolve_library(&library, None, active_agent.as_deref());

        let mut results = serde_json::Map::new();
        for harness_id in &resolved.harnesses {
            if let Some(output) = compile(&resolved, harness_id) {
                let result = CompileResult {
                    harness: harness_id.clone(),
                    context_content: output.context_content,
                    mcp_servers: output.mcp_servers,
                    mcp_config_path: output.mcp_config_path,
                    skill_files: output.skill_files,
                    rule_files: output.rule_files,
                    claude_settings_patch: output.claude_settings_patch,
                    codex_config_patch: output.codex_config_patch,
                    cursor_hooks_patch: output.cursor_hooks_patch,
                    cursor_cli_permissions: output.cursor_cli_permissions,
                    cursor_environment_json: output.cursor_environment_json,
                    claude_lsp_config: output.claude_lsp_config,
                    opencode_config_patch: output.opencode_config_patch,
                    agent_files: output.agent_files,
                    plugins_manifest: output.plugins_manifest,
                };
                if let Ok(v) = serde_json::to_value(&result) {
                    results.insert(harness_id.clone(), v);
                }
            }
        }

        serde_json::to_string(&results)
            .map_err(|e| JsValue::from_str(&format!("Serialisation error: {e}")))
    }

    /// List the supported harness ids.
    #[wasm_bindgen(js_name = listProviders)]
    pub fn list_harnesses_js() -> JsValue {
        let ids: Vec<&str> = crate::list_harnesses().iter().map(|p| p.id).collect();
        serde_wasm_bindgen::to_value(&ids).unwrap_or(JsValue::NULL)
    }
}
