//! Schema source resolvers for the `data live` SSE endpoint.
//!
//! A resolver owns a namespace prefix (e.g. `ship.agents`) and translates
//! parsed expressions into:
//!   1. An initial JSON value for the signal at hydrate time
//!   2. A list of kernel namespaces whose events should trigger a refetch
//!
//! `serve_data` parses the incoming `?expr=...`, dispatches to the matching
//! resolver, sends the initial patch, and (if the resolver subscribes)
//! spawns a kernel actor that re-resolves and re-emits on each event.

pub mod agents;

use crate::rest_api::ApiState;
use serde_json::Value;

/// A parsed `ship.<ns>.<method>(<args>)` expression. Owns its strings so
/// it can travel through `'static` SSE streams.
#[derive(Debug, Clone)]
pub struct ParsedExpr {
    /// Full namespace including the resolver prefix, e.g. `ship.agents`.
    pub namespace: String,
    /// Method name, e.g. `summaries`.
    pub method: String,
    /// Verbatim argument string between the parens, possibly empty.
    pub args: String,
}

/// Parse `ship.foo.bar(args)` into its three parts. Permissive — does not
/// validate the arg syntax; the resolver decides what to do with it.
pub fn parse_expr(expr: &str) -> Option<ParsedExpr> {
    let (head, args) = expr.split_once('(')?;
    let args = args.strip_suffix(')')?;
    let parts: Vec<&str> = head.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    let method = parts.last().unwrap();
    let namespace = parts[..parts.len() - 1].join(".");
    Some(ParsedExpr {
        namespace,
        method: (*method).to_string(),
        args: args.to_string(),
    })
}

/// Schema source for one namespace prefix.
pub trait SailDataResolver: Send + Sync {
    /// Namespace prefix this resolver owns, e.g. `"ship.agents"`.
    fn namespace(&self) -> &'static str;

    /// Compute the current value for a parsed expression. Called once at
    /// hydrate time and again on every subscribed event.
    fn resolve(&self, parsed: &ParsedExpr, state: &ApiState) -> Result<Value, String>;

    /// Kernel namespaces this resolver subscribes to. An empty list means
    /// the data is static — initial value only, no live updates.
    fn subscribe_namespaces(&self, _parsed: &ParsedExpr) -> Vec<String> {
        Vec::new()
    }
}

/// Look up a resolver by namespace. Static registry for now — extending
/// to more namespaces means adding a match arm + a sibling module.
pub fn find(namespace: &str) -> Option<&'static dyn SailDataResolver> {
    match namespace {
        "ship.agents" => Some(&agents::AgentsResolver),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_zero_arg_call() {
        let p = parse_expr("ship.agents.summaries()").unwrap();
        assert_eq!(p.namespace, "ship.agents");
        assert_eq!(p.method, "summaries");
        assert_eq!(p.args, "");
    }

    #[test]
    fn parses_call_with_args() {
        let p = parse_expr("ship.jobs.detail(id=props.id)").unwrap();
        assert_eq!(p.namespace, "ship.jobs");
        assert_eq!(p.method, "detail");
        assert_eq!(p.args, "id=props.id");
    }

    #[test]
    fn parses_nested_namespace() {
        let p = parse_expr("ship.actors.mesh.snapshot()").unwrap();
        assert_eq!(p.namespace, "ship.actors.mesh");
        assert_eq!(p.method, "snapshot");
    }

    #[test]
    fn rejects_missing_parens() {
        assert!(parse_expr("ship.agents.summaries").is_none());
    }

    #[test]
    fn rejects_missing_namespace() {
        assert!(parse_expr("foo()").is_none());
        assert!(parse_expr("foo.bar()").is_none());
    }

    #[test]
    fn finds_known_namespace() {
        assert!(find("ship.agents").is_some());
        assert!(find("ship.unknown").is_none());
    }
}
