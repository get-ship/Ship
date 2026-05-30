//! Canonical upstream harness schema URLs.
//!
//! Used by:
//! - Ship JSON schemas — `$ref` in harness_settings / harness_defaults
//! - Web app — CodeMirror autocomplete for harness_settings
//! - CI — drift detection (schema_drift_tests.rs)
//!
//! When a harness publishes a new schema URL, update this file.

/// Claude Code settings JSON Schema (SchemaStore).
pub const CLAUDE_SCHEMA_URL: &str = "https://www.schemastore.org/claude-code-settings.json";

/// OpenAI Codex config JSON Schema (GitHub).
pub const CODEX_SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/config.schema.json";

/// OpenCode config JSON Schema (official site).
pub const OPENCODE_SCHEMA_URL: &str = "https://opencode.ai/config.json";

/// Cursor has no published JSON Schema. Config is file-based:
/// `.cursor/rules/*.mdc`, `.cursor/mcp.json`, `.cursor/cli.json`, `.cursor/hooks.json`.
pub const CURSOR_SCHEMA_URL: Option<&str> = None;

/// All harness schema URLs, keyed by harness ID.
/// Returns `(harness_id, Option<schema_url>)`.
pub const HARNESS_SCHEMAS: &[(&str, Option<&str>)] = &[
    ("claude", Some(CLAUDE_SCHEMA_URL)),
    ("codex", Some(CODEX_SCHEMA_URL)),
    ("cursor", None),
    ("opencode", Some(OPENCODE_SCHEMA_URL)),
    // Pi has no settings-file schema (model/tools are CLI/env; config is
    // user-authored TS extensions). Source: pi.dev/docs.
    ("pi", None),
];

/// Look up the upstream schema URL for a harness.
pub fn schema_url(harness_id: &str) -> Option<&'static str> {
    HARNESS_SCHEMAS
        .iter()
        .find(|(id, _)| *id == harness_id)
        .and_then(|(_, url)| *url)
}

/// Ship-managed keys per harness.
///
/// These keys are written by the compiler from Ship's own schema fields
/// (model, permissions, env, agent_limits, hooks, MCP). They must NOT
/// appear in `harness_settings` / `harness_defaults` — the JSON schemas
/// use `allOf` + `false` to reject them.
///
/// Source of truth: each harness's `build_*` function in `compile/`.
pub const HARNESS_MANAGED_KEYS: &[(&str, &[&str])] = &[
    (
        "claude",
        &[
            "permissions",
            "hooks",
            "env",
            "availableModels",
            "maxCostPerSession",
            "maxTurns",
            "autoMemoryEnabled",
        ],
    ),
    ("codex", &["mcp_servers"]),
    ("cursor", &[]),
    ("opencode", &["mcp", "permission"]),
];

/// Look up Ship-managed keys for a harness.
pub fn managed_keys(harness_id: &str) -> &'static [&'static str] {
    HARNESS_MANAGED_KEYS
        .iter()
        .find(|(id, _)| *id == harness_id)
        .map(|(_, keys)| *keys)
        .unwrap_or(&[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_url_returns_known_harnesses() {
        assert!(schema_url("claude").is_some());
        assert!(schema_url("codex").is_some());
        assert!(schema_url("opencode").is_some());
        assert!(schema_url("cursor").is_none());
    }

    #[test]
    fn schema_url_returns_none_for_unknown() {
        assert!(schema_url("unknown-harness").is_none());
    }
}
