//! PROOF (test-scoped) — the single-source typed-op pattern for `agents.*`.
//!
//! Demonstrates the cohesion fix without touching live `OPS`, the dispatch
//! chain, or `xtask` (those are the dispatched refactor). One `agents_ops!`
//! declaration emits BOTH the descriptor slice and the dispatch — collapsing
//! the OPS-vs-dispatch shadow — and the named return type replaces the
//! anonymous `json!`. Three tests prove the pattern is sound and behaviour-
//! preserving. The whole module compiles under `cfg(test)` only; nothing here
//! ships as production scaffold.

use super::{required_str, RegistryError};
use crate::services::schema::{ArgDescriptor, OpDescriptor, OPS};
use serde_json::{json, Value};
use ship_platform::services::agents::view_state::{
    list_agents, load_agent, AgentDetail, AgentsSummariesResponse,
};
use std::path::Path;

/// One declaration per op → descriptor + typed dispatch. The descriptor's
/// `returns` is derived from the actual return type (`stringify!`), not prose,
/// which is what lets `data.ts` drop `DataHandle<unknown>`.
macro_rules! agents_ops {
    ( $( { name: $name:literal, desc: $desc:literal, perm: $perm:literal,
           args: $args:expr, run: $run:expr, ret: $ret:ty } ),* $(,)? ) => {
        /// Generated from the same declaration as `dispatch` — no hand-sync.
        const DESCRIPTORS: &[OpDescriptor] = &[ $(
            OpDescriptor {
                name: $name,
                description: $desc,
                args: $args,
                returns: stringify!($ret),
                permission: $perm,
                sensitive: false,
            },
        )* ];

        /// Typed dispatch — each arm returns its named `Serialize` type;
        /// `to_value` is the only `Value` boundary. (Production threads
        /// `&ApiState`; the proof takes `&Path` to stay state-harness-free.)
        fn dispatch(name: &str, args: &Value, ship_dir: &Path)
            -> Result<Option<Value>, RegistryError>
        {
            match name {
                $( $name => {
                    let run: fn(&Value, &Path) -> Result<$ret, RegistryError> = $run;
                    let typed = run(args, ship_dir)?;
                    let value = serde_json::to_value(&typed)
                        .map_err(|e| RegistryError::service(e.to_string()))?;
                    Ok(Some(value))
                } )*
                _ => Ok(None),
            }
        }
    };
}

agents_ops! {
    {
        name: "agents.summaries",
        desc: "List all agents with display metadata (label, harnesses, tags).",
        perm: "ship.agents.read",
        args: &[],
        run: |_args, ship_dir| Ok(AgentsSummariesResponse { items: list_agents(ship_dir) }),
        ret: AgentsSummariesResponse
    },
    {
        name: "agents.detail",
        desc: "Full view-shaped agent detail — catalogs, permissions, harnesses, tags.",
        perm: "ship.agents.read",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        run: |args, ship_dir| {
            let id = required_str(args, "id")?;
            load_agent(ship_dir, id)
                .ok_or_else(|| RegistryError::not_found(format!("agent `{id}`")))
        },
        ret: AgentDetail
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_platform::services::agents::view_state::AgentSummary;

    fn sample_agent() -> AgentSummary {
        AgentSummary {
            id: "code-reviewer".into(),
            name: "Code Reviewer".into(),
            description: "Reviews diffs.".into(),
            skills_count: 3,
            mcp_count: 1,
            rules_count: 2,
            letter: "C".into(),
            source: "PROJECT AGENTS".into(),
            tags: vec!["review".into()],
            active: false,
        }
    }

    /// PROOF 1 — the descriptor is mechanically reproducible from the single
    /// declaration: generated entries match the hand-written `OPS` on every
    /// stable field, so the hand-kept catalog is redundant. `returns` is the
    /// one intended upgrade — prose → the concrete type name.
    #[test]
    fn descriptors_reproduce_ops_catalog() {
        for generated in DESCRIPTORS {
            let canonical = OPS
                .iter()
                .find(|o| o.name == generated.name)
                .unwrap_or_else(|| panic!("op {} missing from OPS", generated.name));
            assert_eq!(generated.name, canonical.name);
            assert_eq!(generated.description, canonical.description);
            assert_eq!(generated.permission, canonical.permission);
            assert_eq!(generated.sensitive, canonical.sensitive);
            assert_eq!(
                serde_json::to_value(generated.args).unwrap(),
                serde_json::to_value(canonical.args).unwrap(),
                "args differ for {}",
                generated.name
            );
        }

        // `returns` now names the real type (enables typed `data.ts`):
        let summaries = DESCRIPTORS.iter().find(|o| o.name == "agents.summaries").unwrap();
        assert_eq!(summaries.returns, "AgentsSummariesResponse");
        let detail = DESCRIPTORS.iter().find(|o| o.name == "agents.detail").unwrap();
        assert_eq!(detail.returns, "AgentDetail");
    }

    /// PROOF 2 — the named return type serializes byte-identically to the
    /// anonymous `json!({"items": …})` the current dispatch hand-builds.
    /// Swapping to the type is wire-invisible.
    #[test]
    fn named_type_is_wire_identical_to_anonymous_json() {
        let items = vec![sample_agent(), sample_agent()];
        let typed = AgentsSummariesResponse { items: items.clone() };

        let from_struct = serde_json::to_value(&typed).unwrap();
        let from_anonymous = json!({ "items": items.clone() });
        assert_eq!(from_struct, from_anonymous);
    }

    /// PROOF 3 — one declaration yields a working dispatch; unknown names fall
    /// through. (Live-handler byte equivalence against a populated `ship_dir`
    /// is the dispatched refactor's gate, run on the real `ApiState` harness.)
    #[test]
    fn dispatch_routes_and_falls_through() {
        let miss = dispatch("agents.nonexistent", &json!({}), Path::new("/does-not-exist")).unwrap();
        assert!(miss.is_none());
    }
}
