//! Integration tests for `/api/v1/views/{name}/edit-mode/{enter|exit}` and
//! the capability-gated tool dispatch.

use ship_platform::view_authoring::{self, ViewEditCapability};

fn unique_id(prefix: &str) -> String {
    format!("{prefix}-{}", ship_substrate::gen_ulid())
}

#[test]
fn enter_exit_flip_capability_state() {
    let session = unique_id("sess-flip");
    let view = unique_id("flip-view");
    assert!(!view_authoring::has_grant(&session, &view));

    view_authoring::grant(&session, &view);
    assert!(view_authoring::has_grant(&session, &view));
    assert!(view_authoring::grants_snapshot().contains(&ViewEditCapability {
        session_id: session.clone(),
        view_name: view.clone(),
    }));

    let removed = view_authoring::revoke(&session, &view);
    assert!(removed, "first revoke removes the grant");
    let removed_again = view_authoring::revoke(&session, &view);
    assert!(!removed_again, "second revoke is a no-op");
    assert!(!view_authoring::has_grant(&session, &view));
}

#[test]
fn revoke_session_end_drops_all_views_for_session() {
    let session = unique_id("sess-rs");
    let other = unique_id("sess-rs-other");
    view_authoring::grant(&session, "view-a");
    view_authoring::grant(&session, "view-b");
    view_authoring::grant(&other, "view-c");
    view_authoring::revoke_session(&session);
    assert!(!view_authoring::has_grant(&session, "view-a"));
    assert!(!view_authoring::has_grant(&session, "view-b"));
    assert!(view_authoring::has_grant(&other, "view-c"));
    view_authoring::revoke(&other, "view-c");
}
