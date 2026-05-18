//! Scope policy unit tests.
//!
//! Per-route enforcement is now declarative (`@auth ...` on each `Route`
//! variant) and is exhaustively covered by
//! `apps/shipd/src/routes/scope_enforcement_tests.rs` — that suite walks
//! `Route::all()` and proves the router rejects insufficient scopes for
//! every variant.
//!
//! What stays here: the small set of tests that exercise scope policy in
//! the abstract (role → scopes mapping, `Scope::Full` semantics, auth
//! context identity), independent of any specific HTTP route.

use crate::auth::Scope;
use crate::test_harness::TestDaemon;

#[tokio::test(flavor = "multi_thread")]
async fn daemon_token_has_full_scope() {
    let auth = TestDaemon::cli_auth();
    assert!(auth.has_scope(Scope::Full));
    assert!(auth.has_scope(Scope::CreateJobs));
    assert!(auth.has_scope(Scope::AdminDaemon));
    assert!(auth.has_scope(Scope::MeshBroadcast));
}

#[tokio::test(flavor = "multi_thread")]
async fn contributor_has_full_scope() {
    let scopes = crate::auth::scopes_for_role(Some("contributor"));
    assert!(Scope::Full.is_granted_by(&scopes));
    assert!(Scope::CreateJobs.is_granted_by(&scopes));
    assert!(Scope::AdminDaemon.is_granted_by(&scopes));
}

#[tokio::test(flavor = "multi_thread")]
async fn unknown_role_gets_minimal_scopes() {
    let scopes = crate::auth::scopes_for_role(Some("unknown-role"));
    assert!(Scope::ReadWorkspace.is_granted_by(&scopes));
    assert!(Scope::ReadJobs.is_granted_by(&scopes));
    assert!(Scope::LogProgress.is_granted_by(&scopes));
    assert!(!Scope::CreateJobs.is_granted_by(&scopes));
    assert!(!Scope::MeshSend.is_granted_by(&scopes));
    assert!(!Scope::WriteWorkspace.is_granted_by(&scopes));
}
