//! Tool stability + naming registry.
//!
//! All MCP tools are always registered (no compile-time gating of tool surface).
//! Stability is a runtime property advertised via the tool's description prefix
//! and exposed in the catalog.
//!
//! Naming convention is **verb-first** (`verb_noun` / `verb_resource`).
//! Tools renamed to fit the convention keep one-release aliases that route to
//! the new name with a deprecation warning surfaced in the description.
//!
//! Maintainers: when renaming a tool, add an entry to `RENAMES` and prefer
//! editing the canonical name in the `#[tool(name = "…")]` attribute. Aliases
//! are removed in the release after they ship.

use std::borrow::Cow;

use rmcp::handler::server::router::tool::{ToolRoute, ToolRouter};

/// One-release alias map: deprecated_name → canonical_name.
///
/// Deprecation policy: aliases survive exactly one release after rename, then
/// drop. Audit before each release: anything still listed here that's older
/// than one cycle must be removed.
pub const RENAMES: &[(&str, &str)] = &[
    // Mesh: noun_verb → verb_noun
    ("mesh_send", "send_mesh_message"),
    ("mesh_broadcast", "broadcast_mesh_message"),
    ("mesh_discover", "discover_mesh_agents"),
    ("mesh_status", "update_mesh_status"),
    // View errors / lint / sdk / scaffold: drop ship_ prefix and reorder
    ("ship_view_errors", "get_view_errors"),
    ("ship_view_lint", "lint_view"),
    ("ship_sdk_query", "query_sdk"),
    ("ship_view_scaffold", "scaffold_view"),
    // Artifacts: noun_verb → verb_noun
    ("artifact_upload", "upload_artifact"),
    ("artifact_get", "get_artifact"),
    ("artifact_list", "list_artifacts"),
    ("artifact_delete", "delete_artifact"),
    ("artifact_promote", "promote_artifact"),
    // Views (canvas): noun_verb → verb_noun
    ("view_open", "open_view"),
    ("view_patch", "patch_view"),
    ("view_append", "append_view"),
    ("view_signal", "signal_view"),
    ("view_close", "close_view"),
    // Single-word event → emit_event
    ("event", "emit_event"),
];

/// Tools whose interface may still change. Description and catalog flag these
/// as experimental. They are always registered (no compile-time gating).
pub const EXPERIMENTAL_TOOLS: &[&str] = &[
    "dispatch_agent",
    "list_agents",
    "stop_agent",
    "steer_agent",
    "emit_event",
    "open_view",
    "patch_view",
    "append_view",
    "signal_view",
    "close_view",
];

/// Description prefix for experimental tools.
pub const EXPERIMENTAL_PREFIX: &str =
    "**EXPERIMENTAL** — interface may change without notice.\n\n";

/// Description prefix for deprecated aliases (the `{}` is replaced with the
/// canonical name).
pub const DEPRECATED_PREFIX_FMT: &str =
    "**DEPRECATED** — use `{canonical}` instead. This alias is kept for one release.\n\n";

pub fn canonical_name(name: &str) -> &str {
    RENAMES
        .iter()
        .find(|(alias, _)| *alias == name)
        .map(|(_, canonical)| *canonical)
        .unwrap_or(name)
}

/// Post-process a [`ToolRouter`] to:
///   1. Prefix experimental tool descriptions.
///   2. Register aliases for renamed tools, each with a DEPRECATED prefix.
///
/// Returns the (mutated) router.
pub fn finalize_router<S>(mut router: ToolRouter<S>) -> ToolRouter<S>
where
    S: Send + Sync + 'static,
{
    // 1. Mark experimental tools in their descriptions.
    for tool_name in EXPERIMENTAL_TOOLS {
        if let Some(route) = router.map.get_mut(*tool_name) {
            let prev = route
                .attr
                .description
                .clone()
                .unwrap_or_else(|| Cow::Borrowed(""));
            if !prev.starts_with("**EXPERIMENTAL**") {
                route.attr.description = Some(Cow::Owned(format!("{EXPERIMENTAL_PREFIX}{prev}")));
            }
        }
    }

    // 2. Register aliases pointing at the canonical handler.
    let mut alias_routes: Vec<ToolRoute<S>> = Vec::new();
    for (alias, canonical) in RENAMES {
        let Some(canonical_route) = router.map.get(*canonical) else {
            continue;
        };
        let mut clone = canonical_route.clone();
        let canonical_desc = canonical_route
            .attr
            .description
            .clone()
            .unwrap_or_else(|| Cow::Borrowed(""));
        let prefix = DEPRECATED_PREFIX_FMT.replace("{canonical}", canonical);
        clone.attr.name = Cow::Borrowed(*alias);
        clone.attr.description = Some(Cow::Owned(format!("{prefix}{canonical_desc}")));
        alias_routes.push(clone);
    }
    for route in alias_routes {
        router.add_route(route);
    }

    router
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renames_have_no_collisions() {
        // No canonical name appears as both an alias and another canonical.
        for (alias, canonical) in RENAMES {
            assert_ne!(alias, canonical);
            for (a2, c2) in RENAMES {
                if alias == a2 {
                    continue;
                }
                assert_ne!(alias, c2, "alias {alias} collides with canonical {c2}");
                assert_ne!(canonical, a2, "canonical {canonical} collides with alias {a2}");
            }
        }
    }

    #[test]
    fn experimental_tools_use_canonical_names() {
        for tool in EXPERIMENTAL_TOOLS {
            assert_eq!(
                canonical_name(tool),
                *tool,
                "EXPERIMENTAL_TOOLS must list canonical names, not aliases"
            );
        }
    }

    #[test]
    fn canonical_name_resolves_aliases() {
        assert_eq!(canonical_name("mesh_send"), "send_mesh_message");
        assert_eq!(canonical_name("send_mesh_message"), "send_mesh_message");
        assert_eq!(canonical_name("unknown_tool"), "unknown_tool");
    }
}
