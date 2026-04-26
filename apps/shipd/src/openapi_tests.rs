//! Integration tests for the generated OpenAPI spec.
//!
//! Asserts the invariants the spec must always satisfy. Wired in via
//! `mod tests` in `openapi.rs`.
//!
//! Three additional invariants are tracked but not yet enforced:
//! - global `security` requirement is non-empty (modifier sets it; test
//!   needs to inspect the served spec, not `ApiDoc::openapi()`)
//! - public operations override `security: []` (requires routes like
//!   `/api/health` to be utoipa-declared, not inline closures)
//! - every operation declares at least one non-2xx response
//! These three are tracked in `.artifacts/openapi-contract-gaps.md` and
//! become tests once the underlying spec gaps are closed.

use super::ApiDoc;
use utoipa::OpenApi;

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
