//! Codegen for the launch-internal RPC surface. The proto SSOT lives in
//! the ship-contracts repo (`/home/madvib/dev/Ship/ship-contracts/proto/`),
//! so the proto root is read from the `SHIP_PROTO_DIR` env var when set
//! (or a relative path when the env var is not set, for in-tree dev).
//! Default: a relative path to the ship-contracts SSOT for the common
//! case where ship-contracts is checked out as a sibling of ship. The
//! env var is the supported override for CI / non-standard layouts.
//!
//! Output (service trait + client + message types) lands in OUT_DIR;
//! `src/main.rs` `include!`s it. Same pattern as `ship-sidecar-proto` /
//! `ship-ops-rpc`: protoc is sourced via `protoc-bin-vendored` so a
//! fresh clone needs no system install.

fn main() {
    // The proto SSOT is the ship-contracts repo. Allow override via env
    // (e.g. CI sets SHIP_PROTO_DIR to an absolute path); the default is
    // a sibling-repo relative path.
    let proto_root = std::path::PathBuf::from(
        std::env::var("SHIP_PROTO_DIR")
            .unwrap_or_else(|_| "../../../ship-contracts/proto".to_string()),
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
