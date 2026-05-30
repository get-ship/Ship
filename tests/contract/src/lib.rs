//! Contract conformance test helpers.
//!
//! The actual tests live in `src/roundtrip.rs` (integration test binary).
//! This lib exposes shared helpers used by the test binary.

use std::path::{Path, PathBuf};

/// Locate the `proto/fixtures/` directory relative to this crate.
pub fn fixtures_dir() -> PathBuf {
    // tests/contract/ → tests/ → repo root → proto/fixtures/
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // tests/
        .and_then(|p| p.parent()) // repo root
        .expect("cannot locate repo root from tests/contract/")
        .join("proto/fixtures")
}

/// Load + parse a fixture JSON file. Panics with a clear message on failure.
pub fn load_fixture(path: &Path) -> serde_json::Value {
    let data = std::fs::read(path)
        .unwrap_or_else(|e| panic!("cannot read fixture {}: {e}", path.display()));
    serde_json::from_slice(&data)
        .unwrap_or_else(|e| panic!("cannot parse fixture {}: {e}", path.display()))
}

/// Assert JSON roundtrip: serialize `v` to string then parse back and compare.
/// Returns the re-parsed value so callers can do field assertions.
pub fn assert_json_roundtrip(label: &str, v: &serde_json::Value) -> serde_json::Value {
    let serialized = serde_json::to_string(v)
        .unwrap_or_else(|e| panic!("{label}: serialize failed: {e}"));
    serde_json::from_str(&serialized)
        .unwrap_or_else(|e| panic!("{label}: re-parse failed: {e}"))
}
