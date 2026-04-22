//! Tests for the build.rs-generated `Route` URL builders + registration.

use super::*;

#[test]
fn unit_variant_full_path_is_literal() {
    assert_eq!(Route::ListJobs.full_path(), "/api/runtime/jobs");
    assert_eq!(Route::IssueSession.full_path(), "/api/runtime/session");
    assert_eq!(
        Route::AssetDatastarJs.full_path(),
        "/api/runtime/assets/datastar.js"
    );
}

#[test]
fn named_variant_substitutes_params() {
    let url = Route::UpdateSpec {
        id: "abc".to_string(),
    }
    .full_path();
    assert_eq!(url, "/api/runtime/specs/abc");

    let url = Route::DeleteSpec {
        id: "spec-42".to_string(),
    }
    .full_path();
    assert_eq!(url, "/api/runtime/specs/spec-42");

    let url = Route::GetJob {
        job_id: "job-xyz".to_string(),
    }
    .full_path();
    assert_eq!(url, "/api/runtime/jobs/job-xyz");
}

#[test]
fn multi_param_variant_substitutes_all() {
    let url = Route::SkillsPutVar {
        id: "code".to_string(),
        key: "model".to_string(),
    }
    .full_path();
    assert_eq!(url, "/api/skills/code/vars/model");
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
        "/api/runtime/specs/{id}"
    );
    assert_eq!(
        Route::SkillsPutVar {
            id: String::new(),
            key: String::new()
        }
        .template_path(),
        "/api/skills/{id}/vars/{key}"
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
