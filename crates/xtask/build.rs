// Resolve the proto/ source-of-truth location (ship-contracts SSOT) and bake
// it as the `PROTO_ROOT` compile-time env var, so tasks and tests read fixtures
// from the proto checkout. The proto/fixtures SSOT lives in ship-contracts, not
// this repo, and travels with the ship-proto dep.
//
// Resolution order:
//   1. SHIP_PROTO_DIR env override (explicit; for odd layouts — CI, alternate
//      proto checkouts). Kept as a supported escape hatch.
//   2. DEP_SHIP_PROTO_PROTO_DIR — the proto SSOT path declared by ship-proto's
//      build.rs via `cargo:proto_dir=<checkout>/proto` (links = "ship-proto"
//      metadata). The path travels with the ship-proto dep (git dep or local
//      [patch] to the sibling repo), so a fresh machine needs no env var.
//
// The same shape is used in every proto-binding consumer (ship-ops-rpc,
// ship-gateway-client, ship-settings, tests/contract) so resolution stays
// symmetric. The fixtures dir is registered for rerun-if-changed so a fixture
// edit re-bakes PROTO_ROOT without a manual re-export.

fn main() {
    let proto_root = std::env::var("DEP_SHIP_PROTO_PROTO_DIR")
        .or_else(|_| std::env::var("SHIP_PROTO_DIR"))
        .unwrap_or_default();
    let fixtures = std::path::Path::new(&proto_root).join("fixtures");
    println!("cargo:rerun-if-env-changed=SHIP_PROTO_DIR");
    println!("cargo:rerun-if-changed={}", fixtures.display());
    println!("cargo:rustc-env=PROTO_ROOT={proto_root}");
}
