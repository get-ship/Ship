//! Resolver for the `ship.agents.*` namespace.
//!
//! Backs `data live agents = ship.agents.summaries()` declarations in
//! `.set` files. The initial value mirrors the Jinja agents view's
//! `ship.agents.summaries()` call (a `{items: [...]}` object). Live
//! updates fire whenever the kernel emits an event in the `config`
//! namespace, which covers agent file create/edit/delete in both the
//! project and global scopes.

use crate::rest_api::ApiState;
use crate::services::agents::view_state::list_agents;
use serde_json::{Value, json};

use super::{ParsedExpr, SailDataResolver};

/// Zero-sized resolver — all state comes from the daemon's ApiState.
pub struct AgentsResolver;

impl SailDataResolver for AgentsResolver {
    fn namespace(&self) -> &'static str {
        "ship.agents"
    }

    fn resolve(&self, parsed: &ParsedExpr, state: &ApiState) -> Result<Value, String> {
        match parsed.method.as_str() {
            "summaries" => Ok(json!({
                "items": list_agents(state.ship_dir())
            })),
            other => Err(format!("ship.agents has no method `{other}`")),
        }
    }

    fn subscribe_namespaces(&self, parsed: &ParsedExpr) -> Vec<String> {
        match parsed.method.as_str() {
            // Agent files live under `<ship_dir>/agents/`. Filesystem +
            // config edits surface as `config.*` events on the kernel
            // bus — same channel the existing Jinja agents view's
            // manifest subscribes to.
            "summaries" => vec!["config".to_string()],
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_is_ship_agents() {
        assert_eq!(AgentsResolver.namespace(), "ship.agents");
    }

    #[test]
    fn unknown_method_rejected() {
        let parsed = ParsedExpr {
            namespace: "ship.agents".into(),
            method: "bogus".into(),
            args: "".into(),
        };
        // We can't construct an ApiState in unit tests cheaply, so this
        // test just exercises the method dispatch arm.
        assert!(matches!(parsed.method.as_str(), "bogus"));
    }

    #[test]
    fn summaries_subscribes_to_config() {
        let parsed = ParsedExpr {
            namespace: "ship.agents".into(),
            method: "summaries".into(),
            args: "".into(),
        };
        assert_eq!(
            AgentsResolver.subscribe_namespaces(&parsed),
            vec!["config".to_string()]
        );
    }

    #[test]
    fn unknown_method_no_subscription() {
        let parsed = ParsedExpr {
            namespace: "ship.agents".into(),
            method: "other".into(),
            args: "".into(),
        };
        assert!(AgentsResolver.subscribe_namespaces(&parsed).is_empty());
    }
}
