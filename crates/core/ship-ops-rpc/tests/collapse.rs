//! Contract test for E4/E6 — the agent surface collapse.
//!
//! Asserts the ONE canonical `AgentsService` (plural, agents-arc E6) is the sole
//! agent RPC surface: it carries every folded RPC (profile reads/writes + the
//! Sessions read), and the folded services (`SessionsService`, `ChatService`) no
//! longer exist in the descriptor set. Chat was gutted entirely in E6. Reads the
//! embedded `FILE_DESCRIPTOR_SET` (proto SSOT).

use std::collections::BTreeSet;

use ship_ops_rpc::policy::read_policies;

/// Every (method) on a given proto service, from the descriptor set.
fn methods_of(service: &str) -> BTreeSet<String> {
    read_policies()
        .into_iter()
        .filter(|p| p.service == service)
        .map(|p| p.method)
        .collect()
}

/// Every service name present in the descriptor set.
fn services() -> BTreeSet<String> {
    read_policies().into_iter().map(|p| p.service).collect()
}

#[test]
fn folded_services_are_gone() {
    let svcs = services();
    for dead in ["AgentService", "SessionsService", "ChatService"] {
        assert!(
            !svcs.contains(dead),
            "{dead} must be collapsed into AgentsService and removed from the proto",
        );
    }
    assert!(
        svcs.contains("AgentsService"),
        "the ONE AgentsService must survive"
    );
}

#[test]
fn agent_service_exposes_every_folded_rpc() {
    let m = methods_of("AgentsService");

    // ── Control (the pre-collapse AgentService surface) ──
    for rpc in ["Dispatch", "Stop", "Steer", "Grant"] {
        assert!(
            m.contains(rpc),
            "AgentsService must keep control rpc `{rpc}`"
        );
    }

    // ── Profile reads ──
    // `Harnesses` is NOT here: it moved to ship.code.v1.CodeService with
    // the 2026-07-10 mount split (coding capability surface).
    for rpc in [
        "List",
        "Summaries",
        "Detail",
        "Catalogs",
        "Get",
        "Wire",
        "Settings",
        "Templates",
        "PermissionPresets",
        "CreateCtx",
    ] {
        assert!(
            m.contains(rpc),
            "AgentsService must expose profile read rpc `{rpc}`"
        );
    }

    // ── Profile writes ──
    for rpc in [
        "Create",
        "Put",
        "Patch",
        "Delete",
        "Rename",
        "Duplicate",
        "MoveScope",
    ] {
        assert!(
            m.contains(rpc),
            "AgentsService must expose profile write rpc `{rpc}`"
        );
    }

    // ── E5: the 6 def-scoped *Live twins are GONE, replaced by ONE Watch ──
    assert!(
        m.contains("Watch"),
        "AgentsService must expose the unified reactive Watch (replaces the *Live twins)",
    );
    for dead in [
        "SummariesLive",
        "CatalogsLive",
        "DetailLive",
        "WireLive",
        "SettingsLive",
        "CreateCtxLive",
    ] {
        assert!(
            !m.contains(dead),
            "E5: def-scoped *Live twin `{dead}` must be deleted (Watch is its successor)",
        );
    }

    // ── Sessions read (folded from SessionsService) ──
    assert!(
        m.contains("Sessions"),
        "AgentsService must expose the Sessions read"
    );
    assert!(
        m.contains("SessionsLive"),
        "AgentsService must keep the relocated SessionsLive twin"
    );

    // ── E6: chat was gutted entirely (no Chat/ChatLive RPC) ──
    for dead in ["Chat", "ChatLive"] {
        assert!(!m.contains(dead), "E6: chat rpc `{dead}` must be gutted");
    }
}

#[test]
fn folded_rpcs_preserve_their_capability_and_blast_radius() {
    use ship_ops_rpc::BlastRadius;
    let by_method = |method: &str| {
        read_policies()
            .into_iter()
            .find(|p| p.service == "AgentsService" && p.method == method)
            .unwrap_or_else(|| panic!("AgentsService/{method} missing from descriptor"))
    };

    // Sessions read keeps `ship.sessions.read` / C0.
    let sessions = by_method("Sessions");
    assert_eq!(sessions.capability, "ship.sessions.read");
    assert_eq!(sessions.blast, BlastRadius::C0Read);

    // A folded profile write keeps `ship.agents.write` / C2.
    let create = by_method("Create");
    assert_eq!(create.capability, "ship.agents.write");
    assert_eq!(create.blast, BlastRadius::C2Elevated);

    // Grant (operator approval) stays C3.
    let grant = by_method("Grant");
    assert_eq!(grant.capability, "ship.agents.grant");
    assert_eq!(grant.blast, BlastRadius::C3Critical);

    // E5 Watch — the unified reactive read carries the same policy the dropped
    // *Live twins did: `ship.agents.read` / C0.
    let watch = by_method("Watch");
    assert_eq!(watch.capability, "ship.agents.read");
    assert_eq!(watch.blast, BlastRadius::C0Read);
}
