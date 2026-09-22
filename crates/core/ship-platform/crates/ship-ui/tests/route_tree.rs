//! Golden tests for the Phase-4 route-tree composition law
//! (`ship_ui::route_tree`).
//!
//! The law under test (census-views.md Part 5 J3): given the space's
//! already namespace-prefix-merged `views: Vec<ViewDescriptor>`, emit ONE
//! Rust-resident nested [`RouteTree`] — each view's `prefix` + `route`
//! split into path segments and nested one [`RouteNode`] per segment, with
//! children sorted for byte-stable output regardless of declaration order.

use ship_ui::{RouteNode, compose_route_tree};
use ship_ui::ViewDescriptor;

fn view(prefix: &str, route: &str, component_ref: &str) -> ViewDescriptor {
    ViewDescriptor {
        prefix: prefix.to_string(),
        route: route.to_string(),
        component_ref: component_ref.to_string(),
    }
}

/// Linear-scan child lookup by segment — small trees, tests read better
/// than a map.
fn child<'a>(node: &'a RouteNode, segment: &str) -> Option<&'a RouteNode> {
    node.children.iter().find(|c| c.segment == segment)
}

fn component_ref(node: &RouteNode) -> Option<&str> {
    node.view.as_ref().map(|v| v.component_ref.as_str())
}

// ── Prefix-scoped subtrees ───────────────────────────────────────────────────

#[test]
fn two_extensions_views_compose_into_one_tree_with_prefix_scoped_subtrees() {
    let views = vec![
        view("ext-alpha", "/dashboard", "alpha.dashboard"),
        view("ext-beta", "/dashboard", "beta.dashboard"),
    ];
    let tree = compose_route_tree(&views);

    assert_eq!(tree.root.children.len(), 2);
    let alpha = child(&tree.root, "ext-alpha").expect("ext-alpha subtree");
    let beta = child(&tree.root, "ext-beta").expect("ext-beta subtree");
    // The prefix node itself mounts nothing — it's a namespace, not a view.
    assert!(alpha.view.is_none());
    assert!(beta.view.is_none());

    let alpha_dashboard = child(alpha, "dashboard").expect("alpha/dashboard");
    let beta_dashboard = child(beta, "dashboard").expect("beta/dashboard");
    // Same local segment ("dashboard") under two different prefixes is not
    // a collision — both survive as distinct nodes in distinct subtrees.
    assert_eq!(component_ref(alpha_dashboard), Some("alpha.dashboard"));
    assert_eq!(component_ref(beta_dashboard), Some("beta.dashboard"));
}

// ── Nesting ──────────────────────────────────────────────────────────────────

#[test]
fn nested_segments_produce_correct_parent_child_structure() {
    let views = vec![
        view("ext-alpha", "/settings", "alpha.settings"),
        view("ext-alpha", "/settings/profile", "alpha.settings.profile"),
    ];
    let tree = compose_route_tree(&views);

    let alpha = child(&tree.root, "ext-alpha").expect("ext-alpha subtree");
    let settings = child(alpha, "settings").expect("settings node");
    // The parent mounts its own view AND has a nested child — a child does
    // not require an index view at every ancestor, but one may exist.
    assert_eq!(component_ref(settings), Some("alpha.settings"));
    assert_eq!(settings.children.len(), 1);

    let profile = child(settings, "profile").expect("nested profile node");
    assert_eq!(component_ref(profile), Some("alpha.settings.profile"));
    assert!(profile.children.is_empty());
}

// ── Degenerate cases ─────────────────────────────────────────────────────────

#[test]
fn empty_view_set_yields_an_empty_tree() {
    let tree = compose_route_tree(&[]);
    assert_eq!(tree.root.segment, "");
    assert!(tree.root.view.is_none());
    assert!(tree.root.children.is_empty());
}

// ── Determinism ──────────────────────────────────────────────────────────────

#[test]
fn identical_view_inputs_compose_to_identical_bytes() {
    let views = vec![
        view("ext-beta", "/dashboard", "beta.dashboard"),
        view("ext-alpha", "/settings/profile", "alpha.settings.profile"),
        view("ext-alpha", "/settings", "alpha.settings"),
    ];
    let make = || compose_route_tree(&views);
    let first = make();
    let second = make();
    assert_eq!(
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&second).unwrap(),
    );
}

#[test]
fn declaration_order_does_not_affect_the_composed_tree() {
    let forward = vec![
        view("ext-alpha", "/dashboard", "alpha.dashboard"),
        view("ext-beta", "/dashboard", "beta.dashboard"),
        view("ext-alpha", "/settings", "alpha.settings"),
    ];
    let mut reversed = forward.clone();
    reversed.reverse();

    assert_eq!(
        serde_json::to_string(&compose_route_tree(&forward)).unwrap(),
        serde_json::to_string(&compose_route_tree(&reversed)).unwrap(),
    );
}
