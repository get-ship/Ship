//! Golden tests for the Phase-4 views composition law (`compiler::compose`).
//!
//! The law under test (census-views.md Part 3): views compose by
//! namespace-prefix merge, not `compose_noun`'s replace-by-id — the same
//! local route from two extensions coexists under distinct prefixes; only a
//! genuine same-prefix collision resolves by rank and records a
//! `ShadowRecord`.

use compiler::compose::{ComposedLibrary, NounKind, compose_space};
use compiler::resolve::{ExtSource, ExtensionDescriptor, ExtensionLibrary, RollupInputs};
use compiler::types::ViewDescriptor;

fn view(prefix: &str, route: &str, component_ref: &str) -> ViewDescriptor {
    ViewDescriptor {
        prefix: prefix.to_string(),
        route: route.to_string(),
        component_ref: component_ref.to_string(),
    }
}

fn ext(source: ExtSource, hash: &str, descriptor: ExtensionLibrary) -> ExtensionDescriptor {
    ExtensionDescriptor {
        source,
        space_root: std::path::PathBuf::from("/x"),
        descriptor,
        content_hash: hash.to_string(),
    }
}

fn inputs(extensions: Vec<ExtensionDescriptor>) -> RollupInputs {
    RollupInputs {
        extensions,
        precedence: RollupInputs::default_precedence(),
    }
}

fn compose(extensions: Vec<ExtensionDescriptor>) -> ComposedLibrary {
    compose_space(&inputs(extensions)).expect("compose must succeed")
}

// ── Namespace-prefix merge ───────────────────────────────────────────────────

#[test]
fn same_local_route_from_two_extensions_coexists_under_distinct_prefixes() {
    let alpha = ExtensionLibrary {
        views: vec![view("ext-alpha", "/dashboard", "alpha.dashboard")],
        ..Default::default()
    };
    let beta = ExtensionLibrary {
        views: vec![view("ext-beta", "/dashboard", "beta.dashboard")],
        ..Default::default()
    };
    let composed = compose(vec![
        ext(ExtSource::UserGlobal, "hash-alpha", alpha),
        ext(ExtSource::Space, "hash-beta", beta),
    ]);

    let mounted: Vec<(&str, &str, &str)> = composed
        .library
        .views
        .iter()
        .map(|v| {
            (
                v.prefix.as_str(),
                v.route.as_str(),
                v.component_ref.as_str(),
            )
        })
        .collect();
    assert_eq!(
        mounted,
        vec![
            ("ext-alpha", "/dashboard", "alpha.dashboard"),
            ("ext-beta", "/dashboard", "beta.dashboard"),
        ]
    );
    // Not a collision — no shadow recorded.
    assert!(composed.shadows.is_empty());
}

#[test]
fn same_prefix_collision_resolves_by_rank_and_records_a_shadow() {
    let user = ExtensionLibrary {
        views: vec![view("ext-alpha", "/dashboard", "user.dashboard")],
        ..Default::default()
    };
    let space = ExtensionLibrary {
        views: vec![view("ext-alpha", "/dashboard", "space.dashboard")],
        ..Default::default()
    };
    let composed = compose(vec![
        ext(ExtSource::UserGlobal, "hash-user", user),
        ext(ExtSource::Space, "hash-space", space),
    ]);

    assert_eq!(composed.library.views.len(), 1);
    assert_eq!(composed.library.views[0].component_ref, "space.dashboard");

    assert_eq!(composed.shadows.len(), 1);
    let shadow = &composed.shadows[0];
    assert_eq!(shadow.noun, NounKind::View);
    assert_eq!(shadow.losing_source, ExtSource::UserGlobal);
    assert_eq!(shadow.losing_tier_hash, "hash-user");
    assert_eq!(shadow.winning_source, ExtSource::Space);
}

// ── Degenerate / identity cases ──────────────────────────────────────────────

#[test]
fn empty_view_tiers_compose_to_the_empty_set() {
    let composed = compose(vec![
        ext(ExtSource::UserGlobal, "h-user", ExtensionLibrary::default()),
        ext(ExtSource::Space, "h-space", ExtensionLibrary::default()),
    ]);
    assert!(composed.library.views.is_empty());
    assert!(composed.shadows.is_empty());
}

#[test]
fn no_extensions_compose_views_to_the_empty_set() {
    let composed = compose(vec![]);
    assert!(composed.library.views.is_empty());
}

// ── Pass-through fence unaffected ────────────────────────────────────────────

#[test]
fn permissions_and_hooks_pass_through_unaffected_by_views() {
    let space = ExtensionLibrary {
        views: vec![view("ext-alpha", "/dashboard", "alpha.dashboard")],
        ..Default::default()
    };
    let composed = compose(vec![ext(ExtSource::Space, "h-space", space.clone())]);
    assert_eq!(
        serde_json::to_string(&composed.library.permissions).unwrap(),
        serde_json::to_string(&space.permissions).unwrap(),
    );
    assert_eq!(
        serde_json::to_string(&composed.library.hooks).unwrap(),
        serde_json::to_string(&space.hooks).unwrap(),
    );
}

// ── Determinism ──────────────────────────────────────────────────────────────

#[test]
fn identical_view_inputs_compose_to_identical_bytes() {
    let alpha = ExtensionLibrary {
        views: vec![
            view("ext-alpha", "/dashboard", "alpha.dashboard"),
            view("ext-alpha", "/settings", "alpha.settings"),
        ],
        ..Default::default()
    };
    let beta = ExtensionLibrary {
        views: vec![
            view("ext-beta", "/dashboard", "beta.dashboard"),
            view("ext-alpha", "/settings", "beta.settings-override"),
        ],
        ..Default::default()
    };
    let make = || {
        compose(vec![
            ext(ExtSource::UserGlobal, "hash-alpha", alpha.clone()),
            ext(ExtSource::Space, "hash-beta", beta.clone()),
        ])
    };
    let first = make();
    let second = make();
    assert_eq!(
        serde_json::to_string(&first.library.views).unwrap(),
        serde_json::to_string(&second.library.views).unwrap(),
    );
    assert_eq!(
        serde_json::to_string(&first.shadows).unwrap(),
        serde_json::to_string(&second.shadows).unwrap(),
    );
}
