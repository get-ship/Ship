// Resolve the proto/ source-of-truth location (ship-contracts SSOT) and
// bake it as a compile-time env var so the contract tests can use
// `include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/...json"))` without
// re-hardcoding the path at every test site.
//
// Resolution order:
//   1. SHIP_PROTO_DIR env override (explicit; for odd layouts — CI, alternate
//      proto checkouts, etc.). Kept as a supported escape hatch.
//   2. DEP_SHIP_PROTO_PROTO_DIR — the proto SSOT path declared by ship-proto's
//      build.rs via `cargo:proto_dir=<checkout>/proto` (links = "ship-proto"
//      metadata). The path travels with the ship-proto dep (git dep or local
//      [patch] to the sibling repo), so a fresh machine needs no env var to
//      build the workspace — the SSOT checkout IS the proto root.
//
// `cargo:rerun-if-env-changed=DEP_SHIP_PROTO_PROTO_DIR` is intentionally NOT
// emitted: changing the dep (which mutates that metadata) already invalidates
// this build script via the normal dependency-rebuild path. SHIP_PROTO_DIR is
// still tracked (rare manual override).
//
// The fixtures dir is registered for rerun-if-changed so a fixture edit
// triggers a recompile of every binary embedding `include_str!(...)` against
// PROTO_ROOT — without the operator needing to re-export anything.
//
// The same shape is used in all four consumers (ship-gateway-client,
// ship-ops-rpc, ship-config, tests/contract) so the resolution logic stays
// symmetric.

fn main() {
    // Default: the proto SSOT path published by the ship-proto dep.
    let proto_root = std::env::var("DEP_SHIP_PROTO_PROTO_DIR")
        .or_else(|_| std::env::var("SHIP_PROTO_DIR"))
        .unwrap_or_else(|_| String::new());
    let fixtures = std::path::Path::new(&proto_root).join("fixtures");
    println!("cargo:rerun-if-env-changed=SHIP_PROTO_DIR");
    println!("cargo:rerun-if-changed={}", fixtures.display());
    println!("cargo:rustc-env=PROTO_ROOT={proto_root}");
}
