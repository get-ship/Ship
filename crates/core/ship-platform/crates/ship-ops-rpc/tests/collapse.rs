//! Contract test for E4 — the agent surface collapse.
//!
//! Asserts the ONE canonical `AgentService` is the sole agent RPC surface: it
//! carries every folded RPC (the AgentsService profile reads/writes, the
//! Sessions read, the Chat read), and the three folded services
//! (`AgentsService`, `SessionsService`, `ChatService`) no longer exist in the
//! descriptor set. Reads the embedded `FILE_DESCRIPTOR_SET` (proto SSOT), so it
//! fails RED until the protos are merged + regenerated.

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
    for dead in ["AgentsService", "SessionsService", "ChatService"] {
        assert!(
            !svcs.contains(dead),
            "{dead} must be collapsed into AgentService and removed from the proto",
        );
    }
    assert!(svcs.contains("AgentService"), "the ONE AgentService must survive");
}

#[test]
fn agent_service_exposes_every_folded_rpc() {
    let m = methods_of("AgentService");

    // ── Control (the pre-collapse AgentService surface) ──
    for rpc in ["Dispatch", "Stop", "Steer", "Grant"] {
        assert!(m.contains(rpc), "AgentService must keep control rpc `{rpc}`");
    }

    // ── Profile reads (folded from AgentsService) ──
    for rpc in [
        "List", "Summaries", "Detail", "Catalogs", "Get", "Wire", "Settings",
        "Templates", "PermissionPresets", "Harnesses", "CreateCtx",
    ] {
        assert!(m.contains(rpc), "AgentService must expose profile read rpc `{rpc}`");
    }

    // ── Profile writes (folded from AgentsService) ──
    for rpc in [
        "Create", "Put", "Patch", "Delete", "Rename", "Duplicate", "MoveScope",
    ] {
        assert!(m.contains(rpc), "AgentService must expose profile write rpc `{rpc}`");
    }

    // ── The 6 *Live twins relocated (E5 deletes them when Watch lands) ──
    for rpc in [
        "SummariesLive", "CatalogsLive", "DetailLive", "WireLive", "SettingsLive",
        "CreateCtxLive",
    ] {
        assert!(m.contains(rpc), "AgentService must keep relocated *Live twin `{rpc}`");
    }

    // ── Sessions read (folded from SessionsService) ──
    assert!(m.contains("Sessions"), "AgentService must expose the Sessions read");
    assert!(m.contains("SessionsLive"), "AgentService must keep the relocated SessionsLive twin");

    // ── Chat read (folded from ChatService) ──
    assert!(m.contains("Chat"), "AgentService must expose the Chat read");
    assert!(m.contains("ChatLive"), "AgentService must keep the relocated ChatLive twin");
}

#[test]
fn folded_rpcs_preserve_their_capability_and_blast_radius() {
    use ship_ops_rpc::BlastRadius;
    let by_method = |method: &str| {
        read_policies()
            .into_iter()
            .find(|p| p.service == "AgentService" && p.method == method)
            .unwrap_or_else(|| panic!("AgentService/{method} missing from descriptor"))
    };

    // Sessions read keeps `ship.sessions.read` / C0.
    let sessions = by_method("Sessions");
    assert_eq!(sessions.capability, "ship.sessions.read");
    assert_eq!(sessions.blast, BlastRadius::C0Read);

    // Chat read keeps `ship.chat.read` / C0.
    let chat = by_method("Chat");
    assert_eq!(chat.capability, "ship.chat.read");
    assert_eq!(chat.blast, BlastRadius::C0Read);

    // A folded profile write keeps `ship.agents.write` / C2.
    let create = by_method("Create");
    assert_eq!(create.capability, "ship.agents.write");
    assert_eq!(create.blast, BlastRadius::C2Elevated);

    // Grant (operator approval) stays C3.
    let grant = by_method("Grant");
    assert_eq!(grant.capability, "ship.agents.grant");
    assert_eq!(grant.blast, BlastRadius::C3Critical);
}
