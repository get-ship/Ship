//! Codegen for the launch-internal RPC surface (ship.internal.v1.* — GitService,
//! HarnessService). The proto SSOT lives in the ship-contracts sibling repo.
//!
//! Proto root resolution:
//!   1. `SHIP_PROTO_DIR` env override — for odd layouts (CI alt checkouts, etc.).
//!      Kept as a supported escape hatch for operators who can't use the standard
//!      sibling-repo layout.
//!   2. `DEP_SHIP_PROTO_PROTO_DIR` — the canonical default, propagated via
//!      ship-proto's `cargo:proto_dir=<checkout>/proto` metadata (the crate has
//!      `links = "ship-proto"`). The path travels with the ship-proto dep
//!      (git dep or local [patch] to the sibling repo), so a fresh clone + clean
//!      workspace needs no env var to build — helmd is a direct dep of ship-proto
//!      (see [dependencies] in Cargo.toml) precisely so this env var exists here.
//!
//! `cargo:rerun-if-env-changed=DEP_SHIP_PROTO_PROTO_DIR` is intentionally NOT
//! emitted (the dep metadata change already invalidates this script via the
//! dependency-rebuild path).
//!
//! Output (service trait + client + message types) lands in OUT_DIR;
//! `src/main.rs` `include!`s it. Same pattern as `ship-sidecar-proto` /
//! `ship-ops-rpc`: protoc is sourced via `protoc-bin-vendored` so a
//! fresh clone needs no system install.

fn main() {
    // Default: the proto SSOT path published by the ship-proto dep. SHIP_PROTO_DIR
    // is the explicit override — same shape the workspace's other four proto-
    // binding consumers use.
    let proto_root = std::path::PathBuf::from(
        std::env::var("DEP_SHIP_PROTO_PROTO_DIR")
            .or_else(|_| std::env::var("SHIP_PROTO_DIR"))
            .unwrap_or_default(),
    );
    // SAFETY: build scripts run single-threaded before any user code; setting
    // process env here only affects this build script's protoc spawn.
    unsafe {
        std::env::set_var(
            "PROTOC",
            protoc_bin_vendored::protoc_bin_path().expect("vendored protoc binary missing"),
        );
    }

    connectrpc_build::Config::new()
        .files(&[
            proto_root.join("ship/internal/v1/git.proto"),
            // The harness-compiler wire (ship.internal.v1.HarnessService) lands in
            // the SAME package mod as GitService, so `src/main.rs`'s single
            // `include!` of `ship.internal.v1.mod.rs` picks up both services.
            proto_root.join("ship/internal/v1/harness.proto"),
        ])
        .includes(std::slice::from_ref(&proto_root))
        .compile()
        .expect("connectrpc-build codegen failed");

    println!("cargo:rerun-if-changed={}", proto_root.join("ship/internal/v1").display());
    println!("cargo:rerun-if-env-changed=SHIP_PROTO_DIR");
}
