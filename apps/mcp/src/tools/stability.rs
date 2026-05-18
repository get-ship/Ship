#![allow(dead_code)]
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

use rmcp::handler::server::router::tool::ToolRouter;

/// Stability marker shown in the tool description and catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stability {
    Stable,
    Experimental,
}

impl Stability {
    pub fn as_str(self) -> &'static str {
        match self {
            Stability::Stable => "stable",
            Stability::Experimental => "experimental",
        }
    }
}

/// Catalog entry for a single tool.
#[derive(Clone, Debug)]
pub struct CatalogEntry {
    pub name: &'static str,
    pub stability: Stability,
    /// `Some(canonical)` if this entry is a deprecated alias.
    pub alias_of: Option<&'static str>,
}

/// Tool rename map. Empty pre-v1 — there is no installed-client surface
/// to protect, so renames take effect immediately with no alias kept.
/// Add aliases here only after a real public release, and audit them out
/// each release cycle.
pub const RENAMES: &[(&str, &str)] = &[];

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

pub fn is_experimental(name: &str) -> bool {
    EXPERIMENTAL_TOOLS.contains(&name)
}

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
                .unwrap_or(Cow::Borrowed(""));
            if !prev.starts_with("**EXPERIMENTAL**") {
                route.attr.description = Some(Cow::Owned(format!("{EXPERIMENTAL_PREFIX}{prev}")));
            }
        }
    }

    router
}

/// Walk a router and return a catalog of all tools (canonical + aliases).
pub fn catalog<S>(router: &ToolRouter<S>) -> Vec<CatalogEntry>
where
    S: Send + Sync + 'static,
{
    let mut out: Vec<CatalogEntry> = Vec::new();
    for tool in router.list_all() {
        let name = static_str(&tool.name);
        let alias_of = RENAMES
            .iter()
            .find(|(a, _)| *a == name)
            .map(|(_, c)| *c);
        let stability = if alias_of
            .map(is_experimental)
            .unwrap_or_else(|| is_experimental(name))
        {
            Stability::Experimental
        } else {
            Stability::Stable
        };
        out.push(CatalogEntry {
            name,
            stability,
            alias_of,
        });
    }
    out.sort_by(|a, b| a.name.cmp(b.name));
    out
}

/// Best-effort static-str lookup. Tool names from the macro are always
/// `&'static str`, so this should always succeed; the fallback is a safety net.
#[allow(
    clippy::ptr_arg,
    reason = "the Cow variant is the discriminator — clippy's &str suggestion erases it"
)]
fn static_str(s: &Cow<'static, str>) -> &'static str {
    match s {
        Cow::Borrowed(b) => b,
        // Aliases use Cow::Borrowed too; an Owned name only appears for tools
        // built dynamically. Leak as a last resort to keep the API zero-cost.
        Cow::Owned(o) => Box::leak(o.clone().into_boxed_str()),
    }
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
    fn canonical_name_is_identity_when_no_aliases_registered() {
        assert_eq!(canonical_name("send_mesh_message"), "send_mesh_message");
        assert_eq!(canonical_name("unknown_tool"), "unknown_tool");
    }
}
