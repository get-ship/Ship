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

/// The committed spec at `apps/docs/openapi.json` carries the augmented
/// `x-codeSamples` arrays from `apps/docs/scripts/augment-openapi.ts`.
///
/// Every operation must have curl + JavaScript entries (length ≥ 2) after
/// augmentation — the drift check in CI keeps this file in sync with the
/// live daemon spec, so a regression here means the augment script failed
/// or an operation slipped past it.
#[test]
fn committed_spec_has_code_samples_on_every_operation() {
    // Path is relative to the repo root when tests run under `cargo test -p
    // shipd`. Locate it by walking up from the crate manifest dir.
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .ancestors()
        .find(|p| p.join("apps/docs/openapi.json").exists())
        .expect("could not locate apps/docs/openapi.json from CARGO_MANIFEST_DIR");
    let path = repo_root.join("apps/docs/openapi.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let spec: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");

    let mut missing: Vec<String> = Vec::new();
    let paths = spec
        .get("paths")
        .and_then(|p| p.as_object())
        .expect("paths object");
    for (route, item) in paths {
        let obj = item.as_object().expect("path-item object");
        for (method, op) in obj {
            // skip non-operation keys like `parameters` / `summary` if ever present
            if !matches!(
                method.as_str(),
                "get" | "post" | "put" | "patch" | "delete" | "head" | "options"
            ) {
                continue;
            }
            let samples = op.get("x-codeSamples").and_then(|v| v.as_array());
            match samples {
                Some(arr) if arr.len() >= 2 => continue,
                _ => missing.push(format!("{} {}", method.to_uppercase(), route)),
            }
        }
    }

    assert!(
        missing.is_empty(),
        "operations missing x-codeSamples (expected curl + javascript):\n  - {}",
        missing.join("\n  - ")
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
