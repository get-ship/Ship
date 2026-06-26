//! Codegen for the internal git RPC surface. Input is the shared proto tree at
//! `proto/ship/internal/v1/git.proto` (repo root). Output (service trait +
//! client + message types) lands in OUT_DIR; `src/main.rs` `include!`s it.
//!
//! Same pattern as `ship-sidecar-proto` / `ship-ops-rpc`: protoc is sourced via
//! `protoc-bin-vendored` so a fresh clone needs no system install.

fn main() {
    // helmd lives at apps/helmd; the shared proto tree is at the repo root.
    let proto_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../proto");

    // SAFETY: build scripts run single-threaded before any user code; setting
    // process env here only affects this build script's protoc spawn.
    unsafe {
        std::env::set_var(
            "PROTOC",
            protoc_bin_vendored::protoc_bin_path().expect("vendored protoc binary missing"),
        );
    }

    connectrpc_build::Config::new()
        .files(&[proto_root.join("ship/internal/v1/git.proto")])
        .includes(std::slice::from_ref(&proto_root))
        .compile()
        .expect("connectrpc-build codegen failed");

    println!("cargo:rerun-if-changed=../../proto/ship/internal/v1");
}
