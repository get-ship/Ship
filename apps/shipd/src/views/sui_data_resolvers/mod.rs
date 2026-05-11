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

use crate::routes::rest_api::ApiState;
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
/// registry.
///
/// Args grammar: a single JSON object literal between the parens, or
/// nothing. Empty args become an empty JSON object. Anything else
/// (positional args, identifier-only args, JS expressions) is rejected
/// — clients are expected to substitute identifiers and serialize a
/// JSON literal before opening the SSE channel.
pub fn resolve(parsed: &ParsedExpr, state: &ApiState) -> Result<Value, String> {
    let args = parse_args(&parsed.args)?;
    registry::call(&parsed.registry_name, &args, state)
        .map_err(|e| format!("{e}"))
}

/// Parse the argument string of a `ship.<x>(...)` expression into a
/// JSON object the registry can consume. Three accepted forms:
///
///   - Empty / whitespace → `{}`
///   - Bare object literal — `{ "id": "abc-1" }`
///   - Quoted/encoded object literal — same shape after URL decoding
///
/// Anything else surfaces as a 400-shaped error string. Keeping the
/// grammar this narrow is deliberate: the client side controls
/// serialization and never needs anything richer; loosening this
/// silently invites injection-shaped behavior into a server-evaluated
/// expression.
fn parse_args(raw: &str) -> Result<Value, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Value::Object(Default::default()));
    }
    let parsed: Value = serde_json::from_str(trimmed).map_err(|e| {
        format!(
            "data live expression args must be a JSON object literal (got `{trimmed}`): {e}"
        )
    })?;
    if !parsed.is_object() {
        return Err(format!(
            "data live expression args must be a JSON OBJECT literal (got `{trimmed}`)"
        ));
    }
    Ok(parsed)
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
    fn parses_args_string() {
        let p = parse_expr("ship.git.diff({\"commit\":\"abc123\"})").unwrap();
        assert_eq!(p.registry_name, "git.diff");
        assert_eq!(p.args, "{\"commit\":\"abc123\"}");
    }

    #[test]
    fn parse_args_empty_returns_empty_object() {
        let v = parse_args("").unwrap();
        assert!(v.is_object());
        assert_eq!(v.as_object().unwrap().len(), 0);
    }

    #[test]
    fn parse_args_whitespace_returns_empty_object() {
        let v = parse_args("   ").unwrap();
        assert!(v.is_object());
    }

    #[test]
    fn parse_args_object_literal() {
        let v = parse_args(r#"{"id": "abc-1"}"#).unwrap();
        assert_eq!(v["id"], "abc-1");
    }

    #[test]
    fn parse_args_rejects_bare_identifier() {
        assert!(parse_args("foo").is_err());
    }

    #[test]
    fn parse_args_rejects_array_literal() {
        // Registries take object args, not arrays.
        assert!(parse_args("[1,2,3]").is_err());
    }

    #[test]
    fn parse_args_rejects_partial_json() {
        assert!(parse_args("{commit:abc}").is_err()); // unquoted keys
    }
}
