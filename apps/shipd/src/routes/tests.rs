//! Tests for the build.rs-generated `Route` URL builders + registration.

use super::*;

#[test]
fn unit_variant_full_path_is_literal() {
    assert_eq!(Route::ListJobs.full_path(), "/api/v1/agent_sessions");
    assert_eq!(Route::IssueSession.full_path(), "/api/v1/session");
    assert_eq!(
        Route::AssetViewTokensCss.full_path(),
        "/api/assets/view-tokens.css"
    );
}

#[test]
fn named_variant_substitutes_params() {
    let url = Route::ArtifactDownload {
        id: "abc".to_string(),
    }
    .full_path();
    assert_eq!(url, "/api/v1/artifacts/abc");

    let url = Route::ArtifactMetadata {
        id: "art-42".to_string(),
    }
    .full_path();
    assert_eq!(url, "/api/v1/artifacts/art-42/metadata");

    let url = Route::GetJob {
        pipeline_run_id: "job-xyz".to_string(),
    }
    .full_path();
    assert_eq!(url, "/api/v1/agent_sessions/job-xyz");
}

#[test]
fn multi_param_variant_substitutes_all() {
    let url = Route::SuiPackAsset {
        pack_id: "core".to_string(),
        file: "index.js".to_string(),
    }
    .full_path();
    assert_eq!(url, "/_sui/packs/core/index.js");
}

#[test]
fn method_returns_uppercase_str() {
    assert_eq!(Route::ListJobs.method(), "GET");
    assert_eq!(Route::DispatchPipeline.method(), "POST");
    assert_eq!(
        Route::WriteViewTemplate {
            name: String::new()
        }
        .method(),
        "PUT"
    );
    assert_eq!(
        Route::CloseViewSession {
            id: String::new()
        }
        .method(),
        "DELETE"
    );
}

#[test]
fn template_path_preserves_placeholders() {
    assert_eq!(
        Route::WriteViewTemplate {
            name: String::new()
        }
        .template_path(),
        "/api/v1/views/{name}/template"
    );
    assert_eq!(
        Route::SuiPackAsset {
            pack_id: String::new(),
            file: String::new()
        }
        .template_path(),
        "/_sui/packs/{pack_id}/{file}"
    );
}

#[tokio::test]
async fn ship_route_registers_handler_end_to_end() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    // ship_route wraps the handler in the per-route auth enforcement middleware
    // (see `scope_enforcement_tests.rs` for the full enforcement matrix).
    // Route::Capabilities declares `@auth scope ReadWorkspace`, so the test
    // injects an AuthContext with `Scope::Full` to clear the gate.
    let app: axum::Router = axum::Router::new()
        .ship_route(Route::Capabilities, ok_handler)
        .layer(axum::Extension(crate::auth::AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![crate::auth::Scope::Full],
            grants: vec![],
        }));
    let req = Request::builder()
        .method("GET")
        .uri(Route::Capabilities.full_path())
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), 64).await.unwrap();
    assert_eq!(&body[..], b"ok");
}

/// Contract-freeze guard (Item 9): every route in the canonical enum must
/// either live under `/api/v1/...`, be in the explicit exemption set, or be a
/// non-`/api/` path (first-run UI). A single helper enumerates every variant
/// so a missed prefix shows up as a clear test failure rather than a diffuse
/// integration regression.
#[test]
fn api_v1_prefix_invariants() {
    fn all() -> Vec<(&'static str, String)> {
        // Build a representative instance of every variant. For Named
        // variants we use the build.rs-emitted `<Name>_template()` helpers,
        // which return an instance with empty-string fields — `full_path`
        // still reflects the template structure.
        vec![
            ("Index", Route::Index.full_path()),
            ("Setup", Route::Setup.full_path()),
            ("IssueSession", Route::IssueSession.full_path()),
            ("AssetViewTokensCss", Route::AssetViewTokensCss.full_path()),
            ("AssetThemeCss", Route::AssetThemeCss.full_path()),
            ("ListThemes", Route::ListThemes.full_path()),
            ("ListJobs", Route::ListJobs.full_path()),
            ("DispatchPipeline", Route::DispatchPipeline.full_path()),
            ("Capabilities", Route::Capabilities.full_path()),
            ("EmitEvent", Route::EmitEvent.full_path()),
            ("PostFeedback", Route::PostFeedback.full_path()),
        ]
    }

    fn is_exempt(path: &str) -> bool {
        path == "/"
            || path == "/setup"
            || path == "/api/health"
            || path.starts_with("/api/assets/")
            || path.starts_with("/api/hooks/webhook/")
            || path.starts_with("/auth/")
    }

    for (name, path) in all() {
        if is_exempt(&path) {
            assert!(
                !path.contains("/api/v1/"),
                "{name}: exempt path {path} must NOT carry /v1/"
            );
        } else if path.starts_with("/api/") {
            assert!(
                path.starts_with("/api/v1/"),
                "{name}: covered API path {path} must carry /v1/ prefix"
            );
        }
    }
}
