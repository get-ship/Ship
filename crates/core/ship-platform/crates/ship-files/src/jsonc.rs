//! JSONC comment-stripping helper.
//!
//! Used by the agent config reader. The agent-schema-specific value
//! helpers (`first_letter`, `resolve_array_field`, `extract_str_array`)
//! used to live here but are shaped around the agent config schema, so
//! they moved to `ship_agents::services::agents::agents_jsonc` (D12). This
//! module now exposes only the generic comment-stripper.

/// Strip `// …` / `/* … */` comments and trailing commas from a JSONC
/// input. Thin wrapper over `ship_substrate::fs_util::strip_jsonc_comments`
/// so callers in agents/skills/rules/permissions/mcp continue to import
/// from this module without leaking the substrate path everywhere.
pub fn strip_jsonc(input: &str) -> String {
    ship_substrate::fs_util::strip_jsonc_comments(input)
}
