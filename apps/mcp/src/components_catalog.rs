//! SUI component catalog exposed as `ship://components` MCP resources.
//!
//! The aggregated Custom Elements Manifest (CEM) is built by `bun run
//! --filter @ship/ui-components build` (sui-build's `aggregateCemTo`) from
//! `packages/ui-components/src/<tag>/<tag>.custom-elements.json` and
//! written to `crates/sui/ship-sui/assets/sui-custom-elements.json`. Both
//! the daemon's HTTP endpoint and the MCP resource read from that same
//! compile-time embedded string so the two contracts can never drift.

use serde_json::Value;

const SUI_ELEMENTS_MANIFEST: &str =
    include_str!("../../../crates/sui/ship-sui/assets/sui-custom-elements.json");

pub fn full_catalog() -> &'static str {
    SUI_ELEMENTS_MANIFEST
}

/// Return a CEM containing only modules that declare `tag`. `None` if the
/// tag is unknown. The returned JSON keeps the top-level CEM shape so
/// clients can use one parser for both the full and sliced catalogs.
pub fn component_catalog(tag: &str) -> Option<String> {
    let mut catalog: Value = serde_json::from_str(SUI_ELEMENTS_MANIFEST).ok()?;
    let modules = catalog.get_mut("modules")?.as_array_mut()?;
    let filtered: Vec<Value> = modules
        .iter()
        .filter(|m| module_declares_tag(m, tag))
        .cloned()
        .collect();
    if filtered.is_empty() {
        return None;
    }
    *modules = filtered;
    serde_json::to_string(&catalog).ok()
}

fn module_declares_tag(module: &Value, tag: &str) -> bool {
    let Some(decls) = module.get("declarations").and_then(Value::as_array) else {
        return false;
    };
    decls
        .iter()
        .any(|d| d.get("tagName").and_then(Value::as_str) == Some(tag))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_catalog() -> String {
        serde_json::json!({
            "schemaVersion": "1.0.0",
            "readme": "",
            "modules": [
                {
                    "kind": "javascript-module",
                    "path": "sui-button.js",
                    "declarations": [
                        { "kind": "class", "name": "SuiButton", "tagName": "sui-button" }
                    ]
                },
                {
                    "kind": "javascript-module",
                    "path": "sui-card.js",
                    "declarations": [
                        { "kind": "class", "name": "SuiCard", "tagName": "sui-card" }
                    ]
                }
            ]
        })
        .to_string()
    }

    fn slice(catalog_json: &str, tag: &str) -> Option<String> {
        let mut catalog: Value = serde_json::from_str(catalog_json).ok()?;
        let modules = catalog.get_mut("modules")?.as_array_mut()?;
        let filtered: Vec<Value> = modules
            .iter()
            .filter(|m| module_declares_tag(m, tag))
            .cloned()
            .collect();
        if filtered.is_empty() {
            return None;
        }
        *modules = filtered;
        serde_json::to_string(&catalog).ok()
    }

    #[test]
    fn slice_known_tag_returns_only_that_module() {
        let out = slice(&sample_catalog(), "sui-button").unwrap();
        let parsed: Value = serde_json::from_str(&out).unwrap();
        let modules = parsed["modules"].as_array().unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(
            modules[0]["declarations"][0]["tagName"].as_str(),
            Some("sui-button"),
        );
    }

    #[test]
    fn slice_unknown_tag_returns_none() {
        assert!(slice(&sample_catalog(), "sui-missing").is_none());
    }

    #[test]
    fn full_catalog_is_valid_json() {
        let parsed: Result<Value, _> = serde_json::from_str(full_catalog());
        assert!(parsed.is_ok(), "embedded catalog must parse as JSON");
    }
}
