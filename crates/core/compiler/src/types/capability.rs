//! Capability — the agent's authority grant for the ship-op control plane.
//!
//! A capability is a `{ domain, blast }` pair: the agent may reach ops in
//! `domain` up to the `blast` ceiling. This replaces the dying `role` /
//! `tools_preset` derivation — the gate enforces capability, not the role.
//!
//! `BlastRadius` here MIRRORS the canonical `ship_ops_rpc::policy::BlastRadius`
//! (variants `C0Read | C1Space | C2Elevated | C3Critical`). We keep a local
//! copy rather than depend on `ship-ops-rpc` because that crate carries a
//! protoc `build.rs` + buffa/connectrpc — far too heavy for the WASM-safe,
//! pure-transform `compiler` crate (and its `BlastRadius` is `Serialize`-only,
//! so it can't back a (de)serializable file-format field anyway). The two
//! enums MUST stay in sync; the descriptor-string mapping is documented on
//! each variant.

use serde::{Deserialize, Serialize};

/// Blast-radius ceiling a capability grants, ordered `C0 < C1 < C2 < C3`.
///
/// - **C0** read — anyone (implicit baseline).
/// - **C1** workspace-write — any agent (implicit baseline).
/// - **C2** elevated — granted agents only (dispatch / settings / authoring).
/// - **C3** critical — operator / 2FA only (account / permissions / creds).
///   **Never agent-settable** — the schema rejects it and [`Capability::validate`]
///   fails closed on it. It exists only so the type can round-trip the canonical
///   enum and the gate can compare against the operator ceiling.
///
/// Serde uses the short tier code (`"C0".."C3"`) — ergonomic in `.ship/agents/*`
/// files. The canonical proto descriptor names (`C0_SAFE`, `C1_WORKSPACE`,
/// `C2_ELEVATED`, `C3_CRITICAL`) are accepted as aliases and produced by
/// [`BlastRadius::as_descriptor`].
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash,
)]
pub enum BlastRadius {
    #[default]
    #[serde(rename = "C0", alias = "C0_SAFE", alias = "C0_READ", alias = "C0Read")]
    C0Read,
    #[serde(rename = "C1", alias = "C1_WORKSPACE", alias = "C1_SPACE", alias = "C1Space")]
    C1Space,
    #[serde(rename = "C2", alias = "C2_ELEVATED", alias = "C2Elevated")]
    C2Elevated,
    #[serde(rename = "C3", alias = "C3_CRITICAL", alias = "C3Critical")]
    C3Critical,
}

impl BlastRadius {
    /// Parse from a canonical proto descriptor string (or the short tier code).
    /// Mirrors `ship_ops_rpc::policy::BlastRadius::from_descriptor` — unknown
    /// fails closed to `C3Critical` (the most restrictive tier).
    pub fn from_descriptor(s: &str) -> Self {
        match s {
            "C0" | "C0_SAFE" | "C0_READ" => Self::C0Read,
            "C1" | "C1_WORKSPACE" | "C1_SPACE" => Self::C1Space,
            "C2" | "C2_ELEVATED" => Self::C2Elevated,
            "C3" | "C3_CRITICAL" => Self::C3Critical,
            _ => Self::C3Critical,
        }
    }

    /// The canonical proto descriptor string (matches `ship_ops_rpc`).
    pub fn as_descriptor(self) -> &'static str {
        match self {
            Self::C0Read => "C0_SAFE",
            Self::C1Space => "C1_WORKSPACE",
            Self::C2Elevated => "C2_ELEVATED",
            Self::C3Critical => "C3_CRITICAL",
        }
    }

    /// C0/C1 are an implicit baseline every agent already has; only C2 needs an
    /// explicit grant. C3 is operator-only and never agent-settable.
    pub fn is_agent_settable(self) -> bool {
        !matches!(self, Self::C3Critical)
    }
}

/// One authority grant: the agent may reach ops in `domain` up to `blast`.
///
/// `blast` defaults to `C0Read` (the read baseline). Granting a whole domain at
/// `C2Elevated` is the elevated-services grant the gate enforces. `domain` is a
/// ship-op service domain, e.g. `"collections"`, `"runtime"`, `"mcp"`.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Capability {
    /// Ship-op service domain this grant covers (e.g. `"collections"`).
    pub domain: String,
    /// Blast-radius ceiling for ops in `domain`. Defaults to `C0Read`.
    #[serde(default)]
    pub blast: BlastRadius,
}

impl Capability {
    pub fn new(domain: impl Into<String>, blast: BlastRadius) -> Self {
        Self {
            domain: domain.into(),
            blast,
        }
    }

    /// Reject grants an agent author may not declare. C3 (`C3Critical`) is
    /// operator-only; everything else (C0/C1/C2) is author-settable.
    pub fn validate(&self) -> Result<(), String> {
        if !self.blast.is_agent_settable() {
            return Err(format!(
                "capability '{}' declares blast {} — C3 (critical) is operator-only and never agent-settable",
                self.domain,
                self.blast.as_descriptor()
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blast_serde_uses_short_codes() {
        assert_eq!(serde_json::to_string(&BlastRadius::C0Read).unwrap(), "\"C0\"");
        assert_eq!(
            serde_json::to_string(&BlastRadius::C2Elevated).unwrap(),
            "\"C2\""
        );
    }

    #[test]
    fn blast_accepts_descriptor_aliases() {
        let c2: BlastRadius = serde_json::from_str("\"C2_ELEVATED\"").unwrap();
        assert_eq!(c2, BlastRadius::C2Elevated);
        let c1: BlastRadius = serde_json::from_str("\"C1\"").unwrap();
        assert_eq!(c1, BlastRadius::C1Space);
    }

    #[test]
    fn blast_is_ordered() {
        assert!(BlastRadius::C0Read < BlastRadius::C1Space);
        assert!(BlastRadius::C2Elevated < BlastRadius::C3Critical);
    }

    #[test]
    fn capability_defaults_blast_to_c0() {
        let cap: Capability = serde_json::from_str(r#"{ "domain": "collections" }"#).unwrap();
        assert_eq!(cap.blast, BlastRadius::C0Read);
    }

    #[test]
    fn capability_validate_rejects_c3() {
        let cap = Capability::new("account", BlastRadius::C3Critical);
        assert!(cap.validate().is_err());
        let ok = Capability::new("collections", BlastRadius::C2Elevated);
        assert!(ok.validate().is_ok());
    }
}
