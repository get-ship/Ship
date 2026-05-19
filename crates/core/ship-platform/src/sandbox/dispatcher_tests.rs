//! Tests for `SandboxDispatcher` — the surface the orchestrator uses
//! to route actor spawns through the registry + the right
//! ActorRuntime. Written first; impl follows.

use super::dispatcher::SandboxDispatcher;
use super::types::{RuntimeEndpoint, SandboxLifetime, SandboxName, SandboxState};
use chrono::Utc;
use ship_isolation::tier::{IsolationProfile, IsolationStrategy, IsolationTier};

fn container_profile() -> IsolationProfile {
    IsolationProfile {
        tier: Some(IsolationTier::Container),
        strategy: Some(IsolationStrategy::DockerEngine),
        policy: None,
    }
}

fn none_profile() -> IsolationProfile {
    IsolationProfile {
        tier: Some(IsolationTier::None),
        strategy: Some(IsolationStrategy::Noop),
        policy: None,
    }
}

#[tokio::test]
async fn ensure_default_sandbox_creates_then_returns_existing() {
    let dispatcher = SandboxDispatcher::new();
    let id1 = dispatcher
        .ensure_default_sandbox("my-project", none_profile())
        .await
        .expect("first call creates");
    let id2 = dispatcher
        .ensure_default_sandbox("my-project", none_profile())
        .await
        .expect("second call returns existing");
    assert_eq!(id1, id2, "must return the same sandbox id for same project");

    // Different project gets a different sandbox.
    let id3 = dispatcher
        .ensure_default_sandbox("other-project", none_profile())
        .await
        .unwrap();
    assert_ne!(id1, id3, "different project must get a different sandbox");
}

#[tokio::test]
async fn ensure_default_sandbox_marks_tier_zero_in_process_immediately() {
    let dispatcher = SandboxDispatcher::new();
    let id = dispatcher
        .ensure_default_sandbox("my-project", none_profile())
        .await
        .unwrap();
    let registry = dispatcher.registry();
    let reg = registry.read().await;
    let sandbox = reg.get(&id).expect("registered");
    assert_eq!(sandbox.state, SandboxState::Running);
    assert!(
        matches!(sandbox.runtime_endpoint, Some(RuntimeEndpoint::InProcess)),
        "Tier 0/1 sandboxes resolve to InProcess endpoint immediately"
    );
}

#[tokio::test]
async fn register_actor_attaches_to_sandbox_and_idle_clock_clears() {
    let dispatcher = SandboxDispatcher::new();
    let sandbox_id = dispatcher
        .ensure_default_sandbox("my-project", none_profile())
        .await
        .unwrap();

    dispatcher
        .register_actor(&sandbox_id, "actor-1")
        .await
        .expect("register actor");
    let registry = dispatcher.registry();
    let reg = registry.read().await;
    let sandbox = reg.get(&sandbox_id).unwrap();
    assert_eq!(sandbox.actors, vec!["actor-1"]);
    assert!(sandbox.idle_since.is_none(), "idle clock cleared when actor attaches");
}

#[tokio::test]
async fn notify_actor_exit_arms_idle_clock_when_last_actor_leaves() {
    let dispatcher = SandboxDispatcher::new();
    let sandbox_id = dispatcher
        .ensure_default_sandbox("my-project", none_profile())
        .await
        .unwrap();
    dispatcher.register_actor(&sandbox_id, "actor-1").await.unwrap();
    dispatcher.register_actor(&sandbox_id, "actor-2").await.unwrap();

    dispatcher.notify_actor_exit("actor-1").await.unwrap();
    // One actor still attached — no idle clock.
    let registry = dispatcher.registry();
    let reg = registry.read().await;
    assert_eq!(reg.get(&sandbox_id).unwrap().state, SandboxState::Running);
    drop(reg);

    dispatcher.notify_actor_exit("actor-2").await.unwrap();
    let registry = dispatcher.registry();
    let reg = registry.read().await;
    let sandbox = reg.get(&sandbox_id).unwrap();
    assert_eq!(sandbox.state, SandboxState::Idle);
    assert!(sandbox.idle_since.is_some());
}

#[tokio::test]
async fn project_with_explicit_pod_keeps_default_separate() {
    let dispatcher = SandboxDispatcher::new();

    let default_id = dispatcher
        .ensure_default_sandbox("my-project", container_profile())
        .await
        .unwrap();
    let strict_id = dispatcher
        .ensure_sandbox(
            "my-project",
            SandboxName::new("untrusted"),
            container_profile(),
            SandboxLifetime::PerRun,
        )
        .await
        .unwrap();
    assert_ne!(default_id, strict_id, "different pods within project are independent");

    let registry = dispatcher.registry();
    let reg = registry.read().await;
    let strict = reg.get(&strict_id).unwrap();
    assert_eq!(strict.name, SandboxName::new("untrusted"));
    assert!(matches!(strict.lifetime, SandboxLifetime::PerRun));
}

#[tokio::test]
async fn reap_eligible_returns_idle_sandboxes_past_timeout() {
    let dispatcher = SandboxDispatcher::new();
    let id = dispatcher
        .ensure_sandbox(
            "my-project",
            SandboxName::default_name(),
            none_profile(),
            SandboxLifetime::Idle { timeout_secs: 1 },
        )
        .await
        .unwrap();
    dispatcher.register_actor(&id, "actor-1").await.unwrap();
    dispatcher.notify_actor_exit("actor-1").await.unwrap();

    // Idle clock is set to "now"; advance test clock by sleeping >1s.
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    let candidates = dispatcher.reap_candidates(Utc::now()).await;
    assert!(
        candidates.contains(&id),
        "sandbox idle past timeout must be reapable"
    );
}

#[tokio::test]
async fn runtime_for_returns_in_process_for_tier_zero() {
    let dispatcher = SandboxDispatcher::new();
    let id = dispatcher
        .ensure_default_sandbox("my-project", none_profile())
        .await
        .unwrap();
    let runtime = dispatcher
        .runtime_for(&id)
        .await
        .expect("InProcess endpoint must resolve to a runtime");
    assert_eq!(runtime.name(), "in-process");
}

#[tokio::test]
async fn runtime_for_returns_none_for_remote_endpoint_caller_resolves_sidecar() {
    let dispatcher = SandboxDispatcher::new();
    // Tier 2 profile keeps sandbox in Pending state initially.
    let id = dispatcher
        .ensure_default_sandbox("my-project", container_profile())
        .await
        .unwrap();
    let reg = dispatcher.registry();
    let r = reg.read().await;
    assert_eq!(r.get(&id).unwrap().state, SandboxState::Pending);
    drop(r);

    // Orchestrator does the prep (mocked here by just marking remote
    // running with an addr) and the dispatcher exposes the endpoint
    // — but returns None from runtime_for so the orchestrator picks
    // the SidecarActorRuntime impl itself (lives in ship-isolation).
    dispatcher
        .mark_remote_running(&id, "http://127.0.0.1:2468", "http")
        .await
        .expect("mark running");
    let runtime = dispatcher.runtime_for(&id).await;
    assert!(
        runtime.is_none(),
        "Remote endpoint is opaque to dispatcher; caller resolves the sidecar runtime"
    );
    let endpoint = dispatcher.endpoint_for(&id).await.unwrap();
    match endpoint {
        RuntimeEndpoint::Remote { addr, scheme } => {
            assert_eq!(addr, "http://127.0.0.1:2468");
            assert_eq!(scheme, "http");
        }
        other => panic!("expected Remote endpoint, got {other:?}"),
    }
}

#[tokio::test]
async fn reap_marks_stopped_and_clears_endpoint() {
    let dispatcher = SandboxDispatcher::new();
    let id = dispatcher
        .ensure_default_sandbox("my-project", none_profile())
        .await
        .unwrap();
    dispatcher.reap(&id).await.expect("reap idle sandbox");

    let registry = dispatcher.registry();
    let reg = registry.read().await;
    let sandbox = reg.get(&id).unwrap();
    assert_eq!(sandbox.state, SandboxState::Stopped);
    assert!(sandbox.runtime_endpoint.is_none());
}
