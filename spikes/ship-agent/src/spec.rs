//! AgentSpec — the four-facet native agent definition.
//!
//! Settled in design: the native Ship agent is NOT the harness-compiler profile.
//! It is four facets and nothing else. Output contract and run mode are RUNTIME
//! call parameters, not config. Roles are deleted. Composition is just
//! capabilities. Harness fields (default_mode, harness_settings, plugins,
//! isolation, shell hooks) belong to the SUPERSET the compiler reads — never here.
//!
//!   1. Identity     — who it is, how it travels (registry metadata)
//!   2. Disposition  — how it behaves (rules + model + params)
//!   3. Capability   — what it can do: capability IDs (= service domains granted).
//!                     Blast is intrinsic to each op in the proto; never claimed here.
//!   4. Memory       — what it remembers (memento binding; stubbed for slice 1)

use serde::{Deserialize, Serialize};

/// 1 · Identity — pure distribution metadata. Maps 1:1 to the existing schema's
/// `agent` block; harness-independent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}
pub(crate) fn default_version() -> String {
    "0.1.0".into()
}

/// 2 · Disposition — how the agent behaves. `rules` is the user's word and
/// stays (not "instructions"). `model` is a top-level native field — there is
/// no harness to nest it under in the native runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disposition {
    #[serde(default)]
    pub rules: String,
    pub model: String,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_turns: Option<usize>,
}

/// 3 · Capability — the explicit grant. A flat list of capability IDs. Each ID
/// names a service domain the agent may reach (e.g. "collections", "runtime").
/// This becomes the gate's `grants` / elevated_services — replacing the dying
/// role-derivation. Blast is NEVER written here: it is intrinsic to the op in
/// the proto, looked up by the gateway. The author can grant a domain; they can
/// never claim a blast tier.
///
/// `mcp` is a service domain like any other (granting it is the first
/// axis of the MCP gate). Within that grant, `mcp.allow` is the
/// SECOND deny-by-default axis: a per-server list the spec author uses
/// to pin the agent to exactly the configured servers it may call.
/// The list is enforced on the registry, not the gate.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capability {
    /// Granted service domains (= elevated_services for C2). C0/C1 are baseline.
    #[serde(default)]
    pub grants: Vec<String>,
    /// Skill refs — ecosystem/procedural bundles, trigger-loaded. Not gated here.
    #[serde(default)]
    pub skills: Vec<String>,
    /// MCP per-server allow-list. The agent's `mcp` grant must also
    /// exist for MCP to be reachable. When present, this list pins
    /// the agent to a subset of the configured MCP servers.
    #[serde(default)]
    pub mcp_allow: Vec<String>,
}

/// 4 · Memory — binding to the memento product (Space-scoped `decisions`
/// collection + record/query/supersede ops). Stubbed for slice 1: presence of
/// a binding, not its wiring.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Memory {
    /// Whether this agent has a durable decision-record memory bound.
    #[serde(default)]
    pub enabled: bool,
}

/// The native agent definition. Distributable, gated, compiler-readable as the
/// core subset of the superset profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub identity: Identity,
    pub disposition: Disposition,
    #[serde(default)]
    pub capability: Capability,
    #[serde(default)]
    pub memory: Memory,
}

impl AgentSpec {
    /// Parse a native spec from JSON (the `.ship/agents/<id>` core subset).
    pub fn from_json(s: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    /// Convenience for the demo: a minimal collections-granted agent.
    pub fn demo(id: &str, model: &str, grants: &[&str]) -> Self {
        Self {
            identity: Identity {
                id: id.into(),
                name: id.into(),
                version: default_version(),
                description: String::new(),
                tags: vec![],
            },
            disposition: Disposition {
                rules: String::new(),
                model: model.into(),
                temperature: None,
                max_turns: Some(20),
            },
            capability: Capability {
                grants: grants.iter().map(|s| s.to_string()).collect(),
                skills: vec![],
                mcp_allow: vec![],
            },
            memory: Memory { enabled: false },
        }
    }

    /// Convenience for the S-mcp demo: an agent that has the `mcp`
    /// grant plus an explicit `mcp_allow` list (the second
    /// deny-by-default axis). Returns a spec the main demo uses to
    /// show the three branches: denied-no-grant, denied-no-allow,
    /// allowed.
    pub fn demo_mcp(id: &str, model: &str, grants: &[&str], mcp_allow: &[&str]) -> Self {
        let mut s = Self::demo(id, model, grants);
        s.capability.mcp_allow = mcp_allow.iter().map(|s| s.to_string()).collect();
        s
    }
}
