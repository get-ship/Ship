//! Expression parsing and resolution helpers for `<data live />` sources.
//!
//! Every `ship.<domain>.<op>(...)` expression used in a `.sui` template's
//! live-data section is parsed here. The `resolve` function is intentionally
//! a stub that always returns an error — registry string-dispatch was removed
//! as part of the one-RPC refactor. Signal-backed sources (`git.overview`,
//! etc.) bypass this path entirely via `run_signal_data_source` in `sui_sse`.
//! Non-converted sources will surface a `{"kind":"loading"}` envelope until
//! they are ported to typed Connect RPCs.

use crate::ship_runtime::ApiState;
use serde_json::Value;

/// A parsed `ship.<domain>.<op>(...)` expression. Owns its strings so it
/// can travel through `'static` SSE streams.
#[derive(Debug, Clone)]
pub struct ParsedExpr {
    /// Registry name without the `ship.` prefix — e.g. `agents.summaries`.
    pub registry_name: String,
    /// Verbatim argument string between the parens, possibly empty.
    pub args: String,
}

/// Parse `ship.foo.bar(args)`. Permissive on the arg payload — does not
/// validate the JS expression; the dispatcher decides what to do with it.
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

/// Resolve a `ship.*` expression. Registry string-dispatch was removed (one
/// RPC story); this always returns an error. SSE surfaces it as
/// `{"kind":"loading"}`. Clients should migrate to typed Connect RPCs.
pub fn resolve(parsed: &ParsedExpr, _state: &ApiState) -> Result<Value, String> {
    let _ = parse_args(&parsed.args)?; // still validate args syntax
    Err(format!(
        "registry dispatch removed: use typed Connect RPC for '{}'",
        parsed.registry_name
    ))
}

/// Kernel namespaces to subscribe to for live updates. Every resolved
/// expression listens on `config` — filesystem changes under `.ship/` plus
/// agent / pipeline / collection edits all fire `config.*` events.
pub fn subscribe_namespaces(_parsed: &ParsedExpr) -> Vec<String> {
    vec!["config".to_string()]
}

/// Parse the argument string of a `ship.<x>(...)` expression into a JSON
/// object. Accepted forms: empty/whitespace → `{}`; bare object literal.
/// Anything else is a 400-shaped error.
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
        assert!(parse_args("[1,2,3]").is_err());
    }

    #[test]
    fn parse_args_rejects_partial_json() {
        assert!(parse_args("{commit:abc}").is_err());
    }
}
