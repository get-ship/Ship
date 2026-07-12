//! Canonical JSON for the resolver's descriptor output.
//!
//! The Phase-2 invariant (`inv-fmt-deterministic-eval`,
//! `ship-stack-format.json:30-33`): "Config resolves deterministically in a
//! closed-world isolate: Date.now/Math.random blocked, only the vendored
//! `@ship/sdk` resolves (no fs/network), output is byte-stable canonical
//! (sorted-key) JSON. Determinism is what makes the descriptor cacheable —
//! warm reads must equal a JSONC parse."
//!
//! This module is the canonical-JSON sink. Two surface helpers:
//!
//! - [`to_canonical_json`] — serialize a `ExtensionLibrary` with sorted keys,
//!   no whitespace, so two runs over the same input yield the same bytes.
//!   The shape of the value is the resolver's wire contract.
//! - [`extension_library_content_hash`] — the resolver's content-hash helper,
//!   used both for the cache key (J5) and for the
//!   `ModuleDescriptor::content_hash` field the descriptor carries. SHA-256
//!   hex of the canonical bytes; the descriptor treats it as an opaque
//!   string so the leaf stays hash-agnostic (per
//!   `compiler::ModuleDescriptor` doc).
//!
//! Note: only `ExtensionLibrary` has a canonical form here. `ResolvedConfig`
//! is a runtime value (it doesn't derive `Serialize` — the resolved
//! config is what the harness compiles, not a wire format). The
//! descriptor cache (J5) keys on the `ExtensionLibrary`'s content hash;
//! the `ResolvedConfig` is built from the cached library on each read.
//!
//! No `std::env` / no fs — pure functions over the input value. The
//! `!Send` V8 isolate future crosses a `spawn_blocking` boundary upstream
//! in `ts_resolver::resolve_ts_files`; by the time values reach THIS module
//! they are plain data.

#[cfg(test)]
use compiler::RollupInputs;
use compiler::{ComposedLibrary, ExtensionLibrary};
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Serialize any `Serialize` value to canonical (sorted-key) JSON bytes.
///
/// `serde_json` does NOT sort keys by default; for the descriptor cache to
/// work, two reads of the same input MUST yield byte-identical output
/// regardless of insertion order. The `preserve_order` feature on
/// `serde_json` (enabled in this crate's Cargo.toml) keeps the in-memory map
/// order — so we sort via a `BTreeMap`-shaped intermediate by going through
/// `serde_json::Value::Object` and re-keying, then emitting with sorted keys.
pub fn to_canonical_json<T: Serialize>(value: &T) -> serde_json::Result<Vec<u8>> {
    let v = serde_json::to_value(value)?;
    let sorted = sort_value_keys(v);
    serde_json::to_vec(&sorted)
}

/// Serialize a `ExtensionLibrary` specifically, surfacing the typed error
/// shape that callers (the cache writer, the test fixtures) prefer.
///
/// Equivalent to [`to_canonical_json`] — this is the resolver's stable
/// surface for the space library's canonical form.
pub fn extension_library_canonical(library: &ExtensionLibrary) -> serde_json::Result<Vec<u8>> {
    to_canonical_json(library)
}

/// SHA-256 hex digest of a `ExtensionLibrary`'s canonical JSON form.
///
/// The exact algorithm is the resolver's choice (the descriptor carries the
/// hash as an opaque string per `compiler::ModuleDescriptor::content_hash`
/// doc). SHA-256 matches the existing `content_hash` in `AgentDefRecord`
/// (`ship-config/src/agents_index.rs:111-125`) so the cache invalidation
/// key is consistent with the rest of the config tier.
pub fn extension_library_content_hash(library: &ExtensionLibrary) -> serde_json::Result<String> {
    let bytes = extension_library_canonical(library)?;
    Ok(sha256_hex(&bytes))
}

/// SHA-256 hex digest of a [`ComposedLibrary`]'s canonical JSON form —
/// the cache key the J5 descriptor cache uses for the roll-up.
///
/// Because the composed `ExtensionLibrary.library.rollup_inputs` is
/// populated (Phase-3 J2) and that field embeds every extension's
/// `content_hash`, hashing the composed library's canonical form
/// transitively covers the source's contents AND every cached
/// package's content hash. A change to ANY extension's content
/// invalidates this hash; the cache subscriber evicts on the next
/// invalidation tick.
///
/// This is the roll-up's cache key — the spec's "Cache key extends
/// over the cache tier's content hashes" acceptance is this fn.
pub fn composed_library_content_hash(composed: &ComposedLibrary) -> serde_json::Result<String> {
    extension_library_content_hash(&composed.library)
}

// ─── Internals ────────────────────────────────────────────────────────────────

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Recursively walk a `serde_json::Value` and sort every map's keys.
///
/// Only `Value::Object` is reordered; arrays preserve their order, and
/// scalars pass through. This is the exact contract the descriptor cache
/// needs: same input value → same output bytes, regardless of the order
/// keys were inserted.
fn sort_value_keys(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut sorted = serde_json::Map::new();
            for (k, v) in entries {
                sorted.insert(k, sort_value_keys(v));
            }
            Value::Object(sorted)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_value_keys).collect()),
        other => other,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use compiler::ExtensionLibrary;
    use serde_json::json;

    #[test]
    fn canonical_json_sorts_object_keys() {
        // Insertion-order vs alphabetical: same value, two key orders.
        let a = json!({ "b": 1, "a": 2 });
        let b = json!({ "a": 2, "b": 1 });
        let ca = to_canonical_json(&a).unwrap();
        let cb = to_canonical_json(&b).unwrap();
        assert_eq!(ca, cb);
        assert_eq!(ca, br#"{"a":2,"b":1}"#);
    }

    #[test]
    fn canonical_json_sorts_nested_object_keys() {
        let v = json!({ "z": { "y": 1, "x": 2 }, "a": [{ "c": 1, "b": 2 }] });
        let bytes = to_canonical_json(&v).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert_eq!(text, r#"{"a":[{"b":2,"c":1}],"z":{"x":2,"y":1}}"#);
    }

    #[test]
    fn canonical_json_arrays_preserve_order() {
        let v = json!({ "list": [3, 1, 2] });
        let bytes = to_canonical_json(&v).unwrap();
        assert_eq!(bytes, br#"{"list":[3,1,2]}"#);
    }

    #[test]
    fn extension_library_canonical_is_byte_stable_across_runs() {
        let mut lib = ExtensionLibrary::default();
        lib.skills_dir = Some("skills/".to_string());
        lib.rules_dir = None;
        lib.skill_paths = vec!["skills/".to_string(), "docs/".to_string()];
        let a = extension_library_canonical(&lib).unwrap();
        let b = extension_library_canonical(&lib).unwrap();
        assert_eq!(a, b, "two reads of the same library must be byte-identical");
    }

    #[test]
    fn extension_library_content_hash_is_stable() {
        let mut lib = ExtensionLibrary::default();
        lib.skills_dir = Some("custom-skills/".to_string());
        lib.skill_paths = vec!["skills/".to_string(), "docs/".to_string()];
        let h1 = extension_library_content_hash(&lib).unwrap();
        let h2 = extension_library_content_hash(&lib).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64, "SHA-256 hex is 64 chars");
    }

    #[test]
    fn extension_library_content_hash_changes_with_field() {
        let mut a = ExtensionLibrary::default();
        let b = ExtensionLibrary::default();
        a.skills_dir = Some("custom/".to_string());
        let ha = extension_library_content_hash(&a).unwrap();
        let hb = extension_library_content_hash(&b).unwrap();
        assert_ne!(ha, hb, "content hash must change when the library changes");
    }

    #[test]
    fn phase3_rollup_placeholder_omitted_when_none() {
        // The Phase-3 seam stays `None` in Phase 2 — the field is
        // `#[serde(default, skip_serializing_if = "Option::is_none")]` on
        // ExtensionLibrary, so the canonical JSON must not contain a
        // `"rollup_inputs": null` entry.
        let lib = ExtensionLibrary::default();
        let bytes = extension_library_canonical(&lib).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(
            !text.contains("rollup_inputs"),
            "RollupInputs placeholder must not appear in Phase-2 output, got: {text}",
        );
    }

    #[test]
    fn rollup_inputs_serialize_when_populated() {
        // The Phase-3 wire contract: when the field IS populated, it must
        // serialize end-to-end (verifies the type-only Phase-3 placeholder
        // has a working serde impl, so Phase 3 won't have to fight a typo).
        let rollup = RollupInputs::default();
        let bytes = to_canonical_json(&rollup).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(text.contains("\"extensions\""));
        assert!(text.contains("\"precedence\""));
    }
}
