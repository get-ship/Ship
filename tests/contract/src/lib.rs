//! Contract conformance test helpers.
//!
//! The actual tests live in `src/roundtrip.rs` (integration test binary).
//! This lib exposes shared helpers used by the test binary.

use std::path::{Path, PathBuf};

/// Locate the `fixtures/` directory under the proto SSOT (ship-contracts
/// repo). The path is baked at build time by tests/contract/build.rs: default
/// = `DEP_SHIP_PROTO_PROTO_DIR` (the `cargo:proto_dir` metadata published by
/// the ship-proto crate's build.rs), set when this crate's direct ship-proto
/// dep is built. `SHIP_PROTO_DIR` is the explicit override for alternate layouts.
/// Post-multi-repo-cleanup, the fixtures live in ship-contracts/proto, not in
/// the ship repo — the build.rs default already points at the ship-contracts
/// checkout via the dep graph (git dep or local [patch] to the sibling repo),
/// so no env var is required for a normal build.
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("PROTO_ROOT")).join("fixtures")
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
