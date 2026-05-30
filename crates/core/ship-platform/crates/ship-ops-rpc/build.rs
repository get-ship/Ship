// Codegen for the Ship ops ConnectRPC surface. Inputs at proto/ship/ops/v1/
// (repo root), shared with the TS pipeline via buf. Output (the OpsService
// trait + client + message types) lands in OUT_DIR; src/lib.rs include!s it.
//
// We compile the FileDescriptorSet ourselves with protoc (--include_imports),
// because connectrpc-build can only *read* a precompiled set or run protoc to a
// throwaway temp path — and the daemon gate needs the set on disk to read the
// custom method options (capability / blast_radius / sensitive). So protoc runs
// once and feeds both: connectrpc-build (codegen) and lib.rs (include_bytes! →
// the gate). Proto is the single source for shapes + policy (decision #67a).
//
// protoc is sourced via protoc-bin-vendored so fresh clones need no install.
fn main() {
    let proto_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../../proto")
        .canonicalize()
        .expect("proto root not found");
    let descriptor =
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("ops_descriptor.bin");
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc binary missing");

    // 1. Compile the descriptor set. cwd = proto_root + `-I.` so the file names
    //    recorded in the set are proto-relative ("ship/ops/v1/ops.proto"),
    //    which is what connectrpc-build's `files()` matches against.
    let status = std::process::Command::new(&protoc)
        .current_dir(&proto_root)
        .arg("--include_imports")
        .arg(format!("--descriptor_set_out={}", descriptor.display()))
        .arg("-I.")
        .arg("ship/ops/v1/policy.proto")
        .arg("ship/ops/v1/ops.proto")
        .arg("ship/ops/v1/types.proto")
        .arg("ship/ops/v1/reads.gen.proto")
        .arg("ship/ops/v1/commands.gen.proto")
        .arg("ship/ops/v1/stream.proto")
        .arg("ship/ops/v1/analytics.proto")
        .arg("ship/ops/v1/runtime.proto")
        .arg("ship/ops/v1/rules.proto")
        .arg("ship/ops/v1/artifacts.proto")
        .arg("ship/ops/v1/collections.proto")
        .arg("ship/ops/v1/permissions.proto")
        .arg("ship/ops/v1/mcp.proto")
        .arg("ship/ops/v1/skills.proto")
        .arg("ship/ops/v1/pipelines.proto")
        .arg("ship/ops/v1/git.proto")
        .arg("ship/ops/v1/agents.proto")
        .arg("ship/ops/v1/agent.proto")
        .arg("ship/ops/v1/space.proto")
        .arg("ship/ops/v1/settings.proto")
        .arg("ship/ops/v1/sessions.proto")
        .arg("ship/ops/v1/account.proto")
        .arg("ship/ops/v1/chat.proto")
        .arg("ship/ops/v1/events.proto")
        .arg("ship/ops/v1/view.proto")
        .arg("ship/ops/v1/registry.proto")
        .arg("ship/ops/v1/sui.proto")
        .status()
        .expect("failed to spawn protoc");
    assert!(status.success(), "protoc descriptor compile failed");

    // 2. Generate Rust (service trait + client + messages) from the set. Only
    //    ops.proto needs codegen; policy.proto's enum + options live in the set
    //    and are read by the gate via reflection.
    connectrpc_build::Config::new()
        .descriptor_set(&descriptor)
        .files(&[
            "ship/ops/v1/ops.proto",
            "ship/ops/v1/types.proto",
            "ship/ops/v1/reads.gen.proto",
            "ship/ops/v1/commands.gen.proto",
            "ship/ops/v1/stream.proto",
            "ship/ops/v1/analytics.proto",
            "ship/ops/v1/runtime.proto",
            "ship/ops/v1/rules.proto",
            "ship/ops/v1/artifacts.proto",
            "ship/ops/v1/collections.proto",
            "ship/ops/v1/permissions.proto",
            "ship/ops/v1/mcp.proto",
            "ship/ops/v1/skills.proto",
            "ship/ops/v1/pipelines.proto",
            "ship/ops/v1/git.proto",
            "ship/ops/v1/agents.proto",
            "ship/ops/v1/agent.proto",
            "ship/ops/v1/space.proto",
            "ship/ops/v1/settings.proto",
            "ship/ops/v1/sessions.proto",
            "ship/ops/v1/account.proto",
            "ship/ops/v1/chat.proto",
            "ship/ops/v1/events.proto",
            "ship/ops/v1/view.proto",
            "ship/ops/v1/registry.proto",
            "ship/ops/v1/sui.proto",
        ])
        .compile()
        .expect("connectrpc-build codegen failed");

    println!("cargo:rerun-if-changed=../../../proto");
}
