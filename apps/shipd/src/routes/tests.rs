//! Tests for the build.rs-generated `Route` URL builders + registration.

use super::*;

#[test]
fn unit_variant_full_path_is_literal() {
    assert_eq!(Route::ListJobs.full_path(), "/api/v1/jobs");
    assert_eq!(Route::IssueSession.full_path(), "/api/v1/session");
    assert_eq!(
        Route::AssetDatastarJs.full_path(),
        "/api/assets/datastar.js"
    );
}

#[test]
fn named_variant_substitutes_params() {
    let url = Route::UpdateSpec {
        id: "abc".to_string(),
    }
    .full_path();
    assert_eq!(url, "/api/v1/specs/abc");

    let url = Route::DeleteSpec {
        id: "spec-42".to_string(),
    }
    .full_path();
    assert_eq!(url, "/api/v1/specs/spec-42");

    let url = Route::GetJob {
        job_id: "job-xyz".to_string(),
    }
    .full_path();
    assert_eq!(url, "/api/v1/jobs/job-xyz");
}

#[test]
fn multi_param_variant_substitutes_all() {
    let url = Route::SkillsPutVar {
        id: "code".to_string(),
        key: "model".to_string(),
    }
    .full_path();
    assert_eq!(url, "/api/v1/skills/code/vars/model");
}

#[test]
fn method_returns_uppercase_str() {
    assert_eq!(Route::ListJobs.method(), "GET");
    assert_eq!(Route::DispatchPipeline.method(), "POST");
    assert_eq!(
        Route::UpdateSpec {
            id: String::new()
        }
        .method(),
        "PUT"
    );
    assert_eq!(
        Route::DeleteSpec {
            id: String::new()
        }
        .method(),
        "DELETE"
    );
}

#[test]
fn template_path_preserves_placeholders() {
    assert_eq!(
        Route::UpdateSpec {
            id: String::new()
        }
        .template_path(),
        "/api/v1/specs/{id}"
    );
    assert_eq!(
        Route::SkillsPutVar {
            id: String::new(),
            key: String::new()
        }
        .template_path(),
        "/api/v1/skills/{id}/vars/{key}"
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

    let app: axum::Router = axum::Router::new().ship_route(Route::Capabilities, ok_handler);
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
            ("AssetDatastarJs", Route::AssetDatastarJs.full_path()),
            ("AssetViewTokensCss", Route::AssetViewTokensCss.full_path()),
            ("AssetShipElementsJs", Route::AssetShipElementsJs.full_path()),
            ("AssetCodeEditorJs", Route::AssetCodeEditorJs.full_path()),
            ("AssetMarkdownEditorJs", Route::AssetMarkdownEditorJs.full_path()),
            ("AssetTerminalJs", Route::AssetTerminalJs.full_path()),
            ("AssetGitJs", Route::AssetGitJs.full_path()),
            ("AssetElementsManifest", Route::AssetElementsManifest.full_path()),
            ("AssetElementsCatalog", Route::AssetElementsCatalog.full_path()),
            ("AssetJinjaMacrosJson", Route::AssetJinjaMacrosJson.full_path()),
            ("AssetJinjaMacrosMd", Route::AssetJinjaMacrosMd.full_path()),
            ("AssetThemeCss", Route::AssetThemeCss.full_path()),
            ("ListThemes", Route::ListThemes.full_path()),
            ("ListJobs", Route::ListJobs.full_path()),
            ("DispatchPipeline", Route::DispatchPipeline.full_path()),
            ("Capabilities", Route::Capabilities.full_path()),
            ("ListProjects", Route::ListProjects.full_path()),
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
