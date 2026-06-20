//! Role→scope / role→grant tables. Pure data, no I/O. The
//! projection-aware `resolve_grants` lives in `ship_platform::auth::scopes`
//! (L1) and falls back to these role tables.

use super::types::Scope;

/// Artifact scopes granted to agents running inside a pipeline (worker/lead).
/// `pub(super)` so `space_window_scopes` (in `super::session`) can compose
/// the same set the worker/lead agents carry.
pub(super) fn pipeline_agent_artifact_scopes() -> Vec<Scope> {
    vec![
        Scope::ReadArtifactsRun,
        Scope::WriteArtifactsRun,
        Scope::DeleteArtifactsRun,
        Scope::ReadArtifactsProject,
    ]
}

/// Map an agent role to its granted scopes.
pub fn scopes_for_role(role: Option<&str>) -> Vec<Scope> {
    match role {
        Some("commander") => {
            let mut s = vec![
                Scope::ReadWorkspace,
                Scope::WriteWorkspace,
                Scope::ReadRuns,
                Scope::CreateRuns,
                Scope::ManageRuns,
                Scope::EmitEvents,
                Scope::MeshSend,
                Scope::MeshBroadcast,
                Scope::LogProgress,
            ];
            s.extend(pipeline_agent_artifact_scopes());
            s
        }
        Some("lead") => {
            let mut s = vec![
                Scope::ReadWorkspace,
                Scope::WriteWorkspace,
                Scope::ReadRuns,
                Scope::CreateRuns,
                Scope::LogProgress,
                Scope::StepDone,
                Scope::MeshSend,
                Scope::EmitEvents,
            ];
            s.extend(pipeline_agent_artifact_scopes());
            s
        }
        Some("designer") => {
            // Design agents (shipwright, ship-designer) author Ship's own
            // views. They get the workspace read/write surface plus the
            // gated SuiAuthoring tool-group. Not orchestrators — no
            // CreateRuns/ManageRuns.
            let mut s = vec![
                Scope::ReadWorkspace,
                Scope::WriteWorkspace,
                Scope::ReadRuns,
                Scope::EmitEvents,
                Scope::LogProgress,
                Scope::SuiAuthoring,
            ];
            s.extend(pipeline_agent_artifact_scopes());
            s
        }
        Some("contributor") => vec![Scope::Full],
        Some("worker") | None => {
            let mut s = vec![
                Scope::ReadWorkspace,
                Scope::ReadRuns,
                Scope::LogProgress,
                Scope::StepDone,
                Scope::MeshSend,
                Scope::EmitEvents,
            ];
            s.extend(pipeline_agent_artifact_scopes());
            s
        }
        _ => vec![Scope::ReadWorkspace, Scope::ReadRuns, Scope::LogProgress],
    }
}

/// The **elevated services** (the C2 set) a role confers — the gate's scope axis
/// (distinct from `scopes`, which drives REST route-auth + artifact perms until
/// those die with the REST surface). The gate allows a C2 op only when its
/// service is here; the **C1 baseline is implicit** (every agent reads +
/// workspace-writes in every service); **C3 is never role-conferred**.
///
/// This is the role-derived provisioning source today; the install/approval
/// flow (`.artifacts/capability-provisioning-and-approval.md`) refines it into
/// per-agent operator-approved sets. Authoring services are listed ahead of
/// their C2 re-tiering — harmless while those ops are still C1.
pub fn grants_for_role(role: Option<&str>) -> Vec<String> {
    let g: &[&str] = match role {
        // Commander orchestrates: dispatch/stop/steer live in the `agents` service
        // (the AgentsService proto is plural — `op_name("AgentsService", m)`
        // derives `agents.dispatch` etc.). Keeping the domain singular "agent"
        // would fail to match the wire's op-name and the gate would deny every
        // commander-orchestrated op as ungated (the agents-control 500s root cause).
        Some("commander") | Some("lead") => &["agents"],
        // Designer authors: the C2 authoring services (agent profiles, skills,
        // mcp, surface creation, view edit). Note `view`/`create` are singular —
        // the actual op-domains.
        Some("designer") => &["agents", "skills", "mcp", "create", "view"],
        // contributor = the operator's own profile: every elevated service.
        Some("contributor") => &["agents", "skills", "mcp", "create", "view"],
        // worker / unknown: no elevated services (the C1 baseline only).
        _ => &[],
    };
    g.iter().map(|s| s.to_string()).collect()
}
