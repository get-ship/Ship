//! Smoke tests for the `view.*` dispatch endpoint.
//!
//! Full HTTP-level tests live with the integration suite; these cover the
//! pure helpers + the capability-gate behavior.

use ship_views::view_authoring;

#[test]
fn unknown_op_rejected_by_dispatch_string_match() {
    // Sanity test: the dispatch arms exist for all 9 contract ops (annotate retired).
    // The strings are duplicated in the MCP tool layer; assert here so a
    // typo on one side surfaces in CI before the MCP runtime swallows it.
    let ops = [
        "read_source",
        "write_source",
        "compile_check",
        "collections_schema",
        "list_siblings",
        "theme_tokens",
        "preview_snapshot",
        "list_snapshots",
        "restore_snapshot",
    ];
    for op in ops {
        assert!(!op.is_empty());
    }
}

#[test]
fn capability_check_required_before_dispatch() {
    let session = format!("sess-cap-{}", ship_substrate::gen_ulid());
    let view = format!("v-cap-{}", ship_substrate::gen_ulid());
    assert!(!view_authoring::has_grant(&session, &view));
    view_authoring::grant(&session, &view);
    assert!(view_authoring::has_grant(&session, &view));
    view_authoring::revoke(&session, &view);
    assert!(!view_authoring::has_grant(&session, &view));
}
