//! Integration tests for the generated OpenAPI spec.
//!
//! Asserts the four invariants the spec must always satisfy:
//! 1. Global `security` requirement is non-empty (bearer token default).
//! 2. At least one public operation carries a `security: []` override.
//! 3. `ErrorResponse` schema is registered under `components.schemas`.
//! 4. Every operation declares at least one non-2xx response (regression
//!    guard — prevents handlers from advertising success-only contracts).

use super::ApiDoc;
use utoipa::OpenApi;

#[test]
fn spec_has_global_security_requirement() {
    let spec = ApiDoc::openapi();
    let security = spec.security.as_ref().expect("global security set");
    assert!(
        !security.is_empty(),
        "expected at least one global security requirement"
    );
}

#[test]
fn public_operations_override_security_to_empty() {
    let spec = ApiDoc::openapi();
    let item = spec
        .paths
        .paths
        .get("/api/health")
        .expect("health path present");
    let op = item.get.as_ref().expect("health GET");
    let sec = op.security.as_ref().expect("health has explicit security");
    assert!(sec.is_empty(), "public endpoint must carry security=[]");
}

#[test]
fn error_response_schema_is_present() {
    let spec = ApiDoc::openapi();
    let components = spec.components.as_ref().expect("components exist");
    assert!(
        components.schemas.contains_key("ErrorResponse"),
        "ErrorResponse schema must be in components.schemas"
    );
}

/// Every advertised operation must document at least one non-2xx response.
///
/// Handlers that only declare `200` let SDK codegen assume a total success
/// contract — consumers never see typed error branches and end up parsing
/// free-form strings. Enforce the rule at the spec level so new handlers
/// can't regress.
#[test]
fn every_operation_has_a_non_2xx_response() {
    let spec = ApiDoc::openapi();
    let mut violations: Vec<String> = Vec::new();

    for (path, item) in &spec.paths.paths {
        let ops: [(&str, Option<&utoipa::openapi::path::Operation>); 7] = [
            ("get", item.get.as_ref()),
            ("post", item.post.as_ref()),
            ("put", item.put.as_ref()),
            ("patch", item.patch.as_ref()),
            ("delete", item.delete.as_ref()),
            ("head", item.head.as_ref()),
            ("options", item.options.as_ref()),
        ];
        for (method, op) in ops.iter() {
            let Some(op) = op else { continue };
            let has_error = op.responses.responses.keys().any(|code| {
                // utoipa keys are strings like "200", "4XX", "default", …
                let first = code.chars().next().unwrap_or('2');
                first == '4' || first == '5' || code.eq_ignore_ascii_case("default")
            });
            if !has_error {
                violations.push(format!("{} {}", method.to_uppercase(), path));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "operations missing a 4xx/5xx response (spec consumers can't type their errors):\n  - {}",
        violations.join("\n  - ")
    );
}
