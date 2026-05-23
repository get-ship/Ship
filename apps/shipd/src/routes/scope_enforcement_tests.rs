//! Route-level scope enforcement tests.
//!
//! This file closes the structural gap that #80 plastered handler by
//! handler: a new variant added to `Route` cannot ship without an
//! explicit `@auth` annotation, and the router's per-route middleware
//! enforces that declaration before any handler body runs.
//!
//! Three invariants are asserted by walking `Route::all()`:
//!
//! 1. `route_scope_enforcement::enum_is_well_formed` — every variant
//!    resolves a sensible `RequiredAuth`. Mutation methods (POST/PUT/
//!    PATCH/DELETE) are not `Public` unless the path matches a small
//!    intentional allowlist (`/_sui/mutation/{name}`,
//!    `/_sui/views/{name}/patch-elements` is `Scope::Full` already).
//! 2. `route_scope_enforcement::handler_tag_means_handler_enforces` —
//!    every `RequiredAuth::Handler` route's handler module contains a
//!    `require_scope` or `can_*_artifacts` call in source. This is the
//!    "Shape B" grep test the spec proposed, scoped to the small set of
//!    dynamic-scope handlers.
//! 3. `route_scope_enforcement::router_rejects_insufficient_scope` —
//!    end-to-end: build a tiny `axum::Router` with `ship_route(route,
//!    dummy)`, inject an `AuthContext` with no useful scopes, send a
//!    request through the resulting service, and assert 403 for every
//!    `Scope(_)` route. Public routes assert 200. Cli routes assert 403
//!    for `CallerKind::Mcp`. Handler routes pass through (no router
//!    check — handler enforces).

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt;

use crate::auth::{AuthContext, RequiredAuth, Scope};
use crate::routes::{Route, ShipRouterExt};
use ship_substrate::events::CallerKind;

// ────────────────────────────────────────────────────────────────────────────
// Static invariants
// ────────────────────────────────────────────────────────────────────────────

/// Paths that are intentionally public + mutating. Both currently:
/// - `POST /_sui/mutation/{name}` — view-bound mutation handler that
///   delegates to per-view authorization (not scope-based).
const PUBLIC_MUTATION_ALLOWLIST: &[&str] = &["/_sui/mutation/{name}"];

#[test]
fn enum_is_well_formed() {
    let routes = Route::all();
    assert!(
        !routes.is_empty(),
        "Route::all() is empty — build.rs did not generate route enumeration"
    );

    for r in &routes {
        let auth = r.required_auth();
        let method = r.method();
        let path = r.template_path();

        // Mutation routes must not be Public unless explicitly allowlisted.
        if matches!(method, "POST" | "PUT" | "PATCH" | "DELETE")
            && matches!(auth, RequiredAuth::Public)
            && !PUBLIC_MUTATION_ALLOWLIST.contains(&path)
        {
            panic!(
                "Route::{} ({} {}) declares @auth public but is a mutation method. \
                 Add `@auth scope <Scope>` or `@auth cli` to routes.rs, or extend \
                 PUBLIC_MUTATION_ALLOWLIST if the route genuinely needs to stay open.",
                r.variant_name(),
                method,
                path
            );
        }
    }
}

#[test]
fn handler_tag_means_handler_enforces() {
    // For every `RequiredAuth::Handler` route, look at the handler module
    // sources and assert SOME scope check exists. Without this the
    // `Handler` tag would be a silent escape hatch.
    let handler_routes: Vec<&'static str> = Route::all()
        .into_iter()
        .filter(|r| matches!(r.required_auth(), RequiredAuth::Handler))
        .map(|r| r.variant_name())
        .collect();

    // Today the only Handler routes are the dynamic-scope artifact ops.
    // Their checks live in `apps/shipd/src/routes/artifacts_api.rs` via
    // `can_read_artifacts` / `can_write_artifacts` / `can_delete_artifacts`.
    let artifacts_src = include_str!("artifacts_api.rs");
    let mentions_check = artifacts_src.contains("can_read_artifacts")
        && artifacts_src.contains("can_write_artifacts")
        && artifacts_src.contains("can_delete_artifacts");
    assert!(
        mentions_check,
        "Handler-tagged routes ({handler_routes:?}) require dynamic scope \
         checks in apps/shipd/src/routes/artifacts_api.rs (can_*_artifacts), \
         but at least one is missing. A `Handler` annotation isn't a \
         license to skip enforcement."
    );
}

// ────────────────────────────────────────────────────────────────────────────
// End-to-end router enforcement
// ────────────────────────────────────────────────────────────────────────────

fn mcp_no_scope() -> AuthContext {
    AuthContext {
        caller_kind: CallerKind::Mcp,
        agent_id: Some("agent.attacker".into()),
        workspace_id: None,
        session_id: None,
        scopes: vec![],
        grants: vec![],
    }
}

fn mcp_with_full() -> AuthContext {
    AuthContext {
        caller_kind: CallerKind::Mcp,
        agent_id: Some("agent.full".into()),
        workspace_id: None,
        session_id: None,
        scopes: vec![Scope::Full],
        grants: vec![],
    }
}

fn cli_with_full() -> AuthContext {
    AuthContext {
        caller_kind: CallerKind::Cli,
        agent_id: None,
        workspace_id: None,
        session_id: None,
        scopes: vec![Scope::Full],
        grants: vec![],
    }
}

/// Build a single-route router with a dummy handler and an extension layer
/// that injects the given `AuthContext`. `Some(auth)` means "authenticated";
/// `None` means "no AuthContext present" (simulates the public group, where
/// the auth middleware never runs).
fn router_for(route: Route, auth: Option<AuthContext>) -> axum::Router {
    let r = axum::Router::<()>::new();
    let r = if matches!(route.required_auth(), RequiredAuth::Public) {
        r.ship_public_route(route, dummy_handler)
    } else {
        r.ship_route(route, dummy_handler)
    };
    match auth {
        Some(a) => r.layer(axum::Extension(a)),
        None => r,
    }
}

async fn dummy_handler() -> &'static str {
    "ok"
}

fn req_for(method: &str, path: &str) -> Request<Body> {
    // Replace any `{name}` segments with synthetic values so the request
    // matches the axum pattern.
    let concrete = synthesize(path);
    Request::builder()
        .method(Method::from_bytes(method.as_bytes()).unwrap())
        .uri(concrete)
        .body(Body::empty())
        .unwrap()
}

fn synthesize(template: &str) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after.find('}').expect("matching brace");
        out.push_str("x");
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

#[tokio::test(flavor = "multi_thread")]
async fn router_rejects_insufficient_scope() {
    for route in Route::all() {
        let method = route.method();
        let path = route.template_path();
        let auth = route.required_auth();
        let variant = route.variant_name();

        match auth {
            RequiredAuth::Public => {
                // Public routes accept requests with no auth context.
                let svc = router_for(clone_route(&route), None);
                let resp = svc.oneshot(req_for(method, path)).await.unwrap();
                assert_eq!(
                    resp.status(),
                    StatusCode::OK,
                    "Route::{variant} ({method} {path}) is Public but \
                     returned {} for an unauthenticated request",
                    resp.status()
                );
            }
            RequiredAuth::Cli => {
                // MCP caller with Full scope must still be denied.
                let svc = router_for(clone_route(&route), Some(mcp_with_full()));
                let resp = svc.oneshot(req_for(method, path)).await.unwrap();
                assert_eq!(
                    resp.status(),
                    StatusCode::FORBIDDEN,
                    "Route::{variant} ({method} {path}) is Cli but accepted \
                     an MCP caller with Scope::Full"
                );

                // CLI caller passes through (handler returns 200).
                let svc_ok = router_for(clone_route(&route), Some(cli_with_full()));
                let resp_ok = svc_ok.oneshot(req_for(method, path)).await.unwrap();
                assert_eq!(
                    resp_ok.status(),
                    StatusCode::OK,
                    "Route::{variant} ({method} {path}) is Cli but rejected \
                     a CLI caller"
                );
            }
            RequiredAuth::Scope(required) => {
                // No-scope MCP caller denied.
                let svc = router_for(clone_route(&route), Some(mcp_no_scope()));
                let resp = svc.oneshot(req_for(method, path)).await.unwrap();
                assert_eq!(
                    resp.status(),
                    StatusCode::FORBIDDEN,
                    "Route::{variant} ({method} {path}) requires Scope::{required:?} \
                     but accepted a caller with no scopes"
                );

                // Full-scope caller passes.
                let svc_ok = router_for(clone_route(&route), Some(mcp_with_full()));
                let resp_ok = svc_ok.oneshot(req_for(method, path)).await.unwrap();
                assert_eq!(
                    resp_ok.status(),
                    StatusCode::OK,
                    "Route::{variant} ({method} {path}) requires Scope::{required:?} \
                     but rejected a caller with Scope::Full"
                );
            }
            RequiredAuth::Handler => {
                // Handler-tagged routes pass through the route layer — the
                // handler itself enforces. Just confirm no router-side
                // rejection happens.
                let svc = router_for(clone_route(&route), Some(mcp_with_full()));
                let resp = svc.oneshot(req_for(method, path)).await.unwrap();
                assert_ne!(
                    resp.status(),
                    StatusCode::FORBIDDEN,
                    "Route::{variant} ({method} {path}) is Handler but the \
                     router layer rejected a Full-scope caller — the layer \
                     should pass through"
                );
            }
        }
    }
}

#[test]
fn ship_public_route_rejects_non_public_at_registration() {
    let result = std::panic::catch_unwind(|| {
        let _: axum::Router = axum::Router::new()
            .ship_public_route(Route::PostFeedback, dummy_handler);
    });
    assert!(
        result.is_err(),
        "ship_public_route must panic when given a non-Public route \
         (Route::PostFeedback declares Scope(LogProgress))"
    );
}

#[test]
fn ship_route_rejects_public_at_registration() {
    let result = std::panic::catch_unwind(|| {
        let _: axum::Router = axum::Router::new()
            .ship_route(Route::AssetThemeCss, dummy_handler);
    });
    assert!(
        result.is_err(),
        "ship_route must panic when given a Public route \
         (Route::AssetThemeCss declares Public)"
    );
}

// `Route` doesn't derive Clone (its variants own `String` fields). For
// per-iteration test setup we rebuild each variant the same way `all()`
// does. `all()` already returns a fresh Vec, so we can `into_iter()` it
// once and consume each. `clone_route` is here for cases where we need
// the same variant twice (Cli path).
fn clone_route(r: &Route) -> Route {
    // Match each variant and rebuild it. For Named variants we copy the
    // String fields; for Unit variants the construction is trivial. The
    // generated `Route::variant_name()` covers identification but doesn't
    // give us reflection over fields, so we hand-roll this with a macro
    // that lifts the matching from `routes.rs`.
    //
    // Simpler approach: serialize through `(method, path)` and look up
    // again. Since `template_path()` is unique per variant, scanning
    // `all()` is O(N) per call but the test runs once.
    let target_path = r.template_path();
    let target_method = r.method();
    for cand in Route::all() {
        if cand.template_path() == target_path && cand.method() == target_method {
            return cand;
        }
    }
    panic!("clone_route: no match for {} {}", target_method, target_path);
}
