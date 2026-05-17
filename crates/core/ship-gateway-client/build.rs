// Codegen for the Ship gateway Connect-RPC client. Inputs live at
// proto/ship/gateway/v1/*.proto (repo root), shared with the TS server's
// buf pipeline. Output lands in OUT_DIR; src/lib.rs `include!`s it.
//
// protoc is sourced via `protoc-bin-vendored` so fresh clones don't need
// a system-wide install. The PROTOC env var points connectrpc-build at
// the bundled binary.
fn main() {
    let proto_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../proto");

    // SAFETY: build script runs single-threaded before any user code; setting
    // process env here is safe and only affects this build script's spawn of
    // protoc.
    unsafe {
        std::env::set_var(
            "PROTOC",
            protoc_bin_vendored::protoc_bin_path().expect("vendored protoc binary missing"),
        );
    }

    connectrpc_build::Config::new()
        .files(&[proto_root.join("ship/gateway/v1/auth.proto")])
        .includes(&[proto_root.clone()])
        .compile()
        .expect("connectrpc-build codegen failed");

    println!("cargo:rerun-if-changed=../../../proto");
}
