//! State-machine tests for the SandboxRegistry. Behavioral coverage:
//!
//! - Registration enforces unique (project, name).
//! - Pending → Running attaches the endpoint.
//! - Adding an actor moves Idle → Running and clears the idle clock.
//! - Removing the last actor moves Running → Idle and arms the clock.
//! - is_ready_to_reap fires only when idle, no actors, and timeout
//!   has elapsed.
//! - Reaping then Stopped removes name + actor indexes.
//! - mark_stopped is final; reaping a running sandbox is rejected
//!   from inappropriate states.

#![allow(clippy::field_reassign_with_default)]

use super::registry::SandboxRegistry;
use super::types::{
    RuntimeEndpoint, SandboxError, SandboxLifetime, SandboxName,
    SandboxState,
};
use chrono::{Duration as ChronoDuration, Utc};
use ship_isolation::tier::{IsolationProfile, IsolationStrategy, IsolationTier};
#[allow(unused_imports)]
use std::time::Duration;

fn profile() -> IsolationProfile {
    IsolationProfile {
        tier: Some(IsolationTier::Container),
        strategy: Some(IsolationStrategy::DockerEngine),
        policy: None,
    }
}

#[test]
fn register_assigns_id_in_pending_state() {
    let mut reg = SandboxRegistry::new();
    let sandbox = reg
        .register("my-repo", SandboxName::default_name(), profile())
        .expect("register");
    assert_eq!(sandbox.project, "my-repo");
    assert_eq!(sandbox.name, SandboxName::default_name());
    assert_eq!(sandbox.state, SandboxState::Pending);
    assert!(sandbox.runtime_endpoint.is_none());
    assert!(sandbox.actors.is_empty());
}

#[test]
fn register_rejects_duplicate_name_in_same_project() {
    let mut reg = SandboxRegistry::new();
    reg.register("my-repo", SandboxName::default_name(), profile())
        .unwrap();
    let err = reg
        .register("my-repo", SandboxName::default_name(), profile())
        .unwrap_err();
    assert!(matches!(err, SandboxError::DuplicateName { .. }));
}

#[test]
fn register_allows_same_name_across_projects() {
    let mut reg = SandboxRegistry::new();
    reg.register("repo-a", SandboxName::default_name(), profile())
        .unwrap();
    reg.register("repo-b", SandboxName::default_name(), profile())
        .expect("different project, same name should succeed");
    assert_eq!(reg.len(), 2);
}

#[test]
fn pending_to_running_attaches_endpoint() {
    let mut reg = SandboxRegistry::new();
    let id = reg
        .register("my-repo", SandboxName::default_name(), profile())
        .unwrap()
        .id
        .clone();
    reg.mark_running(&id, RuntimeEndpoint::InProcess).unwrap();
    let sandbox = reg.get(&id).unwrap();
    assert_eq!(sandbox.state, SandboxState::Running);
    assert!(matches!(sandbox.runtime_endpoint, Some(RuntimeEndpoint::InProcess)));
}

#[test]
fn add_actor_rejected_on_pending_state() {
    let mut reg = SandboxRegistry::new();
    let id = reg
        .register("my-repo", SandboxName::default_name(), profile())
        .unwrap()
        .id
        .clone();
    let err = reg.add_actor(&id, "actor-1").unwrap_err();
    assert!(matches!(err, SandboxError::InvalidStateForActor { .. }));
}

#[test]
fn add_actor_on_idle_transitions_to_running_and_clears_idle_clock() {
    let mut reg = SandboxRegistry::new();
    let id = reg
        .register("my-repo", SandboxName::default_name(), profile())
        .unwrap()
        .id
        .clone();
    reg.mark_running(&id, RuntimeEndpoint::InProcess).unwrap();
    reg.add_actor(&id, "actor-1").unwrap();
    reg.remove_actor("actor-1", Utc::now()).unwrap();
    assert_eq!(reg.get(&id).unwrap().state, SandboxState::Idle);
    assert!(reg.get(&id).unwrap().idle_since.is_some());

    reg.add_actor(&id, "actor-2").unwrap();
    assert_eq!(reg.get(&id).unwrap().state, SandboxState::Running);
    assert!(reg.get(&id).unwrap().idle_since.is_none());
}

#[test]
fn removing_last_actor_arms_idle_clock() {
    let mut reg = SandboxRegistry::new();
    let id = reg
        .register("my-repo", SandboxName::default_name(), profile())
        .unwrap()
        .id
        .clone();
    reg.mark_running(&id, RuntimeEndpoint::InProcess).unwrap();
    reg.add_actor(&id, "actor-1").unwrap();
    reg.add_actor(&id, "actor-2").unwrap();
    let now = Utc::now();

    reg.remove_actor("actor-1", now).unwrap();
    // One actor still attached — sandbox stays Running, no idle clock.
    assert_eq!(reg.get(&id).unwrap().state, SandboxState::Running);
    assert!(reg.get(&id).unwrap().idle_since.is_none());

    reg.remove_actor("actor-2", now).unwrap();
    // Last actor gone — transition to Idle, arm clock.
    assert_eq!(reg.get(&id).unwrap().state, SandboxState::Idle);
    assert_eq!(reg.get(&id).unwrap().idle_since, Some(now));
}

#[test]
fn reap_candidates_picks_only_idle_past_timeout() {
    let mut reg = SandboxRegistry::new();
    let timeout = Duration::from_secs(60);

    let id_a = reg
        .register("repo-a", SandboxName::default_name(), profile())
        .unwrap()
        .id
        .clone();
    let id_b = reg
        .register("repo-b", SandboxName::default_name(), profile())
        .unwrap()
        .id
        .clone();
    reg.get_mut(&id_a).unwrap().lifetime = SandboxLifetime::Idle { timeout_secs: timeout.as_secs() };
    reg.get_mut(&id_b).unwrap().lifetime = SandboxLifetime::Idle { timeout_secs: timeout.as_secs() };
    reg.mark_running(&id_a, RuntimeEndpoint::InProcess).unwrap();
    reg.mark_running(&id_b, RuntimeEndpoint::InProcess).unwrap();
    reg.add_actor(&id_a, "actor-a").unwrap();
    reg.add_actor(&id_b, "actor-b").unwrap();

    let now = Utc::now();
    reg.remove_actor("actor-a", now - ChronoDuration::seconds(120))
        .unwrap();
    reg.remove_actor("actor-b", now - ChronoDuration::seconds(30))
        .unwrap();

    let candidates = reg.reap_candidates(now);
    assert!(candidates.contains(&id_a), "A idle past 60s should reap");
    assert!(!candidates.contains(&id_b), "B idle only 30s should NOT reap");
}

#[test]
fn reap_candidates_skips_running_and_per_run() {
    let mut reg = SandboxRegistry::new();
    let id_running = reg
        .register("repo-a", SandboxName::default_name(), profile())
        .unwrap()
        .id
        .clone();
    let id_per_run = reg
        .register("repo-b", SandboxName::new("ephemeral"), profile())
        .unwrap()
        .id
        .clone();
    reg.get_mut(&id_per_run).unwrap().lifetime = SandboxLifetime::PerRun;
    reg.mark_running(&id_running, RuntimeEndpoint::InProcess).unwrap();
    reg.mark_running(&id_per_run, RuntimeEndpoint::InProcess).unwrap();
    reg.add_actor(&id_running, "actor-1").unwrap();

    // PerRun sandbox is Running with no actors; it should NOT be in
    // reap_candidates (per-run is reaped explicitly by the run's
    // lifecycle owner, not by the idle reaper).
    let now = Utc::now();
    let candidates = reg.reap_candidates(now + ChronoDuration::seconds(3600));
    assert!(!candidates.contains(&id_running), "Running sandbox should not be reaped");
    assert!(!candidates.contains(&id_per_run), "PerRun sandbox should not be reaped by idle timer");
}

#[test]
fn mark_stopped_drops_name_and_actor_indexes() {
    let mut reg = SandboxRegistry::new();
    let id = reg
        .register("my-repo", SandboxName::default_name(), profile())
        .unwrap()
        .id
        .clone();
    reg.mark_running(&id, RuntimeEndpoint::InProcess).unwrap();
    reg.add_actor(&id, "actor-1").unwrap();
    reg.mark_reaping(&id).unwrap();
    reg.mark_stopped(&id).unwrap();

    // Indexes cleared; sandbox lookup by name fails.
    assert!(reg
        .by_project_name("my-repo", &SandboxName::default_name())
        .is_none());
    assert!(reg.sandbox_for_actor("actor-1").is_none());

    // The sandbox struct itself is still in the registry (state =
    // Stopped) until forget() drops it.
    let sandbox = reg.get(&id).unwrap();
    assert_eq!(sandbox.state, SandboxState::Stopped);
    assert!(sandbox.runtime_endpoint.is_none());
    assert!(sandbox.actors.is_empty());

    reg.forget(&id);
    assert!(reg.get(&id).is_none());
}

#[test]
fn mark_reaping_rejects_pending() {
    let mut reg = SandboxRegistry::new();
    let id = reg
        .register("my-repo", SandboxName::default_name(), profile())
        .unwrap()
        .id
        .clone();
    let err = reg.mark_reaping(&id).unwrap_err();
    assert!(matches!(err, SandboxError::InvalidStateTransition { .. }));
}

#[test]
fn actor_index_supports_lookup_back_to_sandbox() {
    let mut reg = SandboxRegistry::new();
    let id = reg
        .register("my-repo", SandboxName::new("untrusted"), profile())
        .unwrap()
        .id
        .clone();
    reg.mark_running(&id, RuntimeEndpoint::InProcess).unwrap();
    reg.add_actor(&id, "actor-x").unwrap();
    let s = reg.sandbox_for_actor("actor-x").unwrap();
    assert_eq!(s.id, id);
    assert_eq!(s.name, SandboxName::new("untrusted"));
}

#[test]
fn multiple_sandboxes_in_same_project_iterate() {
    let mut reg = SandboxRegistry::new();
    reg.register("my-repo", SandboxName::default_name(), profile())
        .unwrap();
    reg.register("my-repo", SandboxName::new("untrusted"), profile())
        .unwrap();
    reg.register("my-repo", SandboxName::new("gpu"), profile())
        .unwrap();
    reg.register("other-repo", SandboxName::default_name(), profile())
        .unwrap();

    let mine: Vec<_> = reg.sandboxes_in_project("my-repo").collect();
    assert_eq!(mine.len(), 3);
    let other: Vec<_> = reg.sandboxes_in_project("other-repo").collect();
    assert_eq!(other.len(), 1);
}

#[test]
fn sandbox_lifetime_default_is_idle_15min() {
    let lt = SandboxLifetime::default();
    let SandboxLifetime::Idle { timeout_secs } = lt else {
        panic!("default must be Idle");
    };
    assert_eq!(timeout_secs, 15 * 60);
}
