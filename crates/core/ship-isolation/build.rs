//! Codegen for the v1 sidecar protocol. Inputs at
//! `proto/ship/sidecar/v1/*.proto` (workspace root). Output lands in
//! OUT_DIR; `src/sidecar/proto.rs` `include!`s it.
//!
//! Same pattern as `ship-gateway-client` — protoc via
//! `protoc-bin-vendored` so fresh clones don't need a system install.

fn main() {
    let proto_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../proto");

    // SAFETY: build script runs single-threaded before any user code;
    // setting process env here only affects this build script's spawn.
    unsafe {
        std::env::set_var(
            "PROTOC",
            protoc_bin_vendored::protoc_bin_path().expect("vendored protoc binary missing"),
        );
    }

    connectrpc_build::Config::new()
        .files(&[proto_root.join("ship/sidecar/v1/sidecar.proto")])
        .includes(std::slice::from_ref(&proto_root))
        .compile()
        .expect("connectrpc-build codegen failed");

    println!("cargo:rerun-if-changed=../../../proto/ship/sidecar/v1");
}
