// Resolve the proto/ source-of-truth location (ship-contracts SSOT) and
// bake it as a compile-time env var so the contract tests can use
// `include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/...json"))` without
// re-hardcoding the path at every test site.
//
// SHIP_PROTO_DIR overrides the default. The default is the ship-contracts
// sibling-repo absolute path (the SSOT). This build.rs is the same shape
// in all three consumers (ship-gateway-client, ship-ops-rpc, ship-config)
// so the operator only has to set the env var once for the whole workspace.
//
// rerun-if-changed is registered for the resolved fixtures path so a
// fixture edit triggers a recompile (without the operator needing to
// re-export SHIP_PROTO_DIR).

fn main() {
    let proto_root = std::env::var("SHIP_PROTO_DIR")
        .unwrap_or_else(|_| "/home/madvib/dev/Ship/ship-contracts/proto".to_string());
    let fixtures = std::path::Path::new(&proto_root).join("fixtures");
    println!("cargo:rerun-if-env-changed=SHIP_PROTO_DIR");
    println!("cargo:rerun-if-changed={}", fixtures.display());
    println!("cargo:rustc-env=PROTO_ROOT={proto_root}");
}
