//! Schema source for the `data live` SSE endpoint.
//!
//! Every `ship.<domain>.<op>(...)` expression resolves through the
//! existing service registry (`apps/shipd/src/services/registry.rs`).
//! That registry is the single source of truth for read-side domain
//! services — minijinja templates already use it via the `ship.*`
//! globals; this module wires the same calls into Sail's data layer
//! so authors get the same vocabulary in `.set` files.
//!
//! `serve_data` parses the incoming `?expr=...`, calls
//! `services::registry::call` with `<domain>.<op>` and the parsed
//! args, sends the result as the initial patch, then subscribes to a
//! coarse kernel namespace (currently `config`) and re-resolves on
//! each event. Domain-specific subscription scopes can replace the
//! coarse subscription as we narrow the wake-up surface.

use crate::rest_api::ApiState;
use crate::services::registry;
use serde_json::Value;

/// A parsed `ship.<domain>.<op>(<args>)` expression. Owns its strings
/// so it can travel through `'static` SSE streams.
#[derive(Debug, Clone)]
pub struct ParsedExpr {
    /// Registry name (without the `ship.` prefix) — e.g. `agents.summaries`,
    /// `dashboard.state`, `actors.mesh.snapshot`.
    pub registry_name: String,
    /// Verbatim argument string between the parens, possibly empty.
    /// Currently passed as-is — non-empty args are rejected at the
    /// dispatcher level until argument parsing lands.
    pub args: String,
}

/// Parse `ship.foo.bar(args)`. Permissive on the arg payload — does
/// not validate the JS expression; the dispatcher decides what to do
/// with it.
pub fn parse_expr(expr: &str) -> Option<ParsedExpr> {
    let (head, args) = expr.split_once('(')?;
    let args = args.strip_suffix(')')?;
    let registry_name = head.strip_prefix("ship.")?;
    if registry_name.is_empty() || !registry_name.contains('.') {
        return None;
    }
    Some(ParsedExpr {
        registry_name: registry_name.to_string(),
        args: args.to_string(),
    })
}

/// Resolve a `ship.*` expression by delegating to the service
/// registry. Empty args become an empty JSON object.
pub fn resolve(parsed: &ParsedExpr, state: &ApiState) -> Result<Value, String> {
    if !parsed.args.trim().is_empty() {
        return Err(format!(
            "argument-bearing data live expressions not yet supported (got `({})`)",
            parsed.args,
        ));
    }
    registry::call(&parsed.registry_name, &Value::Object(Default::default()), state)
        .map_err(|e| format!("{e}"))
}

/// Kernel namespaces to subscribe to for live updates. v1: every
/// resolved expression listens on `config`. Filesystem changes under
/// `.ship/` plus agent / pipeline / collection edits all fire
/// `config.*` events; the resolver re-fetches and emits a patch.
///
/// Domain-specific scopes (e.g. just `run.*` for pipeline-state
/// queries) can replace this when we want to narrow the wake-up
/// surface — single function to update.
pub fn subscribe_namespaces(_parsed: &ParsedExpr) -> Vec<String> {
    vec!["config".to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_zero_arg_call() {
        let p = parse_expr("ship.agents.summaries()").unwrap();
        assert_eq!(p.registry_name, "agents.summaries");
        assert_eq!(p.args, "");
    }

    #[test]
    fn parses_dashboard_state() {
        let p = parse_expr("ship.dashboard.state()").unwrap();
        assert_eq!(p.registry_name, "dashboard.state");
    }

    #[test]
    fn parses_nested_namespace() {
        let p = parse_expr("ship.actors.mesh.snapshot()").unwrap();
        assert_eq!(p.registry_name, "actors.mesh.snapshot");
    }

    #[test]
    fn rejects_missing_parens() {
        assert!(parse_expr("ship.agents.summaries").is_none());
    }

    #[test]
    fn rejects_missing_ship_prefix() {
        assert!(parse_expr("foo.bar()").is_none());
    }

    #[test]
    fn rejects_top_level_ship() {
        // `ship.summaries()` — registry names are always `<domain>.<op>`.
        assert!(parse_expr("ship.summaries()").is_none());
    }

    #[test]
    fn rejects_args_for_now() {
        // Argument-bearing expressions are deferred until proper arg parsing.
        let p = parse_expr("ship.jobs.detail(id)").unwrap();
        assert_eq!(p.args, "id");
    }
}
