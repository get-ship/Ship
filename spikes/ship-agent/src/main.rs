//! ship-agent-spike — Slice 1.
//!
//! Proves the NON-JS bones of the Ship Agents Framework runtime, against a
//! stubbed gateway (the live one is mid-refactor):
//!   - the four-facet AgentSpec (identity / disposition / capability / memory)
//!   - the ONE door: every op enters `gateway.call(op, args, auth)`
//!   - the gate: real shipd semantics (C0/C1 baseline, C2 needs the granted
//!     service, C3 operator-only) keyed on the spec's capability grants
//!   - the spine: execute -> verify, nothing accepted blindly
//!
//! Slice 1.5 layers deno_core on top: a V8 isolate whose single bound op is a
//! wire to this same `gateway.call`. The gate does not move.

mod agent_loop;
mod events;
mod skills;
mod provider;
mod auth;
mod session;
mod gate;
mod gateway;
mod loader;
mod module_loader;
mod runtime;
mod sdkgen;
mod script_executor;
mod spec;
mod spine;
mod dtsgen;
mod lsp;
use std::sync::Arc;

use serde_json::{json, Value};

use auth::AuthContext;
use gateway::{Gateway, GatewayError, InProcessGateway};
use spec::AgentSpec;
use spine::SpineError;

/// Drive one op through the spine. `execute` is the gateway call (which runs the
/// gate); `verify` confirms the result before it's accepted.
async fn op_through_spine(
    gw: &dyn Gateway,
    auth: &AuthContext,
    label: &str,
    op: &str,
    args: Value,
) -> Result<String, SpineError> {
    let execute = async {
        gw.call(op, args, auth).await.map_err(|e| match e {
            GatewayError::Denied(r) => SpineError::Denied(r),
            other => SpineError::Execute(other.to_string()),
        })
    };
    // Verify: the daemon claims success — confirm the shape before trusting it.
    spine::run(label, execute, |v: &Value| {
        if v.get("ok").and_then(Value::as_bool) == Some(true) {
            Ok(format!("verified op `{}`", v.get("op").unwrap_or(&json!(""))))
        } else {
            Err("result did not confirm ok=true".into())
        }
    })
    .await
}

fn report(label: &str, r: Result<String, SpineError>) {
    match r {
        Ok(ok) => println!("[{label}] ACCEPTED: {ok}\n"),
        Err(e) => println!("[{label}] {e}\n"),
    }
}

// deno_core's op futures are !Send and its task spawner asserts a current-thread
// runtime, so the agent runtime runs single-threaded (V8 isolates are
// single-threaded anyway; cross-isolate parallelism is a pool of these).
#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // The four-facet native spec: a collections-only agent (no runtime/mcp grant).
    let spec = AgentSpec::demo("lead-tracker", "claude-sonnet-4-6", &["collections"]);
    println!(
        "agent   = {} ({})\nmodel   = {}\ngrants  = {:?}\nmemory  = {}\n",
        spec.identity.id,
        spec.identity.version,
        spec.disposition.model,
        spec.capability.grants,
        spec.memory.enabled,
    );

    // The credential presented at the one door = the spec's grants, operator=false.
    let auth = AuthContext::in_process(&spec.identity.id, spec.capability.grants.clone());
    let gw = InProcessGateway;

    // 1. C0 read — baseline, allowed for any agent.
    report(
        "collections.read (C0 baseline)",
        op_through_spine(&gw, &auth, "read", "collections.read", json!({"collection": "leads"})).await,
    );

    // 2. C1 write — workspace mutation, baseline allowed.
    report(
        "collections.put (C1 space)",
        op_through_spine(&gw, &auth, "put", "collections.put", json!({"collection": "leads", "data": {}})).await,
    );

    // 3. C2 in an UNGRANTED service — denied at the gate (agent lacks `mcp`).
    report(
        "mcp.create (C2, not granted)",
        op_through_spine(&gw, &auth, "mcp", "mcp.create", json!({"id": "x"})).await,
    );

    // 4. C2 dispatch (spawn a sub-agent) — denied: this agent lacks `runtime`.
    report(
        "runtime.dispatch (C2, not granted) — spawning is a capability",
        op_through_spine(&gw, &auth, "dispatch", "runtime.dispatch", json!({"agent": "child"})).await,
    );

    // 5. Same dispatch, but for an agent GRANTED `runtime` — now allowed.
    let orchestrator = AuthContext::in_process("commander", vec!["collections".into(), "runtime".into()]);
    report(
        "runtime.dispatch (C2, granted) — orchestrator may spawn",
        op_through_spine(&gw, &orchestrator, "dispatch", "runtime.dispatch", json!({"agent": "child"})).await,
    );

    // 6. C3 — operator-only. No in-process agent can ever clear it.
    report(
        "account.rotate_key (C3 critical) — operator-only",
        op_through_spine(&gw, &orchestrator, "rotate", "account.rotate_key", json!({})).await,
    );

    // 7. Through deno_core: a V8 isolate whose only effect is the one-door op.
    //    JS calls `ship_call`; the gate runs in the gateway, unchanged. A denial
    //    comes back as a result the script can read — not a thrown exception.
    println!("--- via deno_core (JS -> single op -> one door -> gate) ---\n");
    let script = r#"
        (async () => {
          const ok      = await ship_call("collections.read", { collection: "leads" });
          const denied  = await ship_call("mcp.create", { id: "x" });
          return { granted_call: ok, ungranted_call: denied };
        })()
    "#
    .to_string();
    let gw = Arc::new(InProcessGateway);
    match runtime::run_script(gw, auth.clone(), script).await {
        Ok(v) => println!("[deno_core] JS result:\n{}\n", serde_json::to_string_pretty(&v)?),
        Err(e) => println!("[deno_core] ERROR: {e}\n"),
    }

    // 8. Slice 2: agent as an ES MODULE importing the SCOPED SDK.
    //    The SDK is generated from grants ["collections"] — so `collections` is
    //    importable but `mcp`/`agent` are not even in the module. The raw
    //    `op_ship_call("mcp.create")` shows the GATE still guards what the SDK
    //    narrows: soft layer (SDK) + hard layer (gate).
    println!("--- slice 2: scoped SDK module (grant=[collections]) ---");
    println!("generated SDK:\n{}", sdkgen::generate_sdk(&spec.capability.grants));
    let agent_src = r#"
        import { collections } from "ship";
        const via_sdk = await collections.read({ collection: "leads" });
        // raw attempt at an UNGRANTED service — SDK never exposed it; the gate denies it.
        const raw_ungranted = await Deno.core.ops.op_ship_call("mcp.create", { id: "x" });
        Deno.core.ops.op_emit({ via_sdk, raw_ungranted });
    "#
    .to_string();
    let gw = Arc::new(InProcessGateway);
    match runtime::run_module(gw, auth.clone(), spec.capability.grants.clone(), agent_src).await {
        Ok(emitted) => println!(
            "[slice2] agent emitted:\n{}\n",
            serde_json::to_string_pretty(&emitted)?
        ),
        Err(e) => println!("[slice2] ERROR: {e}\n"),
    }

    // 9. Deny-by-default imports: a forbidden import must fail to LOAD.
    println!("--- slice 2: deny-by-default import ---");
    let evil_src = r#"import { readFileSync } from "fs"; Deno.core.ops.op_emit({ escaped: true });"#
        .to_string();
    let gw = Arc::new(InProcessGateway);
    match runtime::run_module(gw, auth.clone(), spec.capability.grants.clone(), evil_src).await {
        Ok(_) => println!("[slice2] ✖ FORBIDDEN import was allowed — deny-by-default FAILED\n"),
        Err(e) => println!("[slice2] ✓ blocked: {e}\n"),
    }

    // 10. Slice 1.6: the boring loop. A MOCK model (no API key) is scripted to
    //     call `ship_op` then answer — proving a model turn drives a gated,
    //     verified op through the one door. Swap the mock for a BYOK provider
    //     model and the same `run_step` runs a real turn.
    println!("--- slice 1.6: model loop (mock) -> ship_op -> one door ---");
    let model = rig::test_utils::MockCompletionModel::new([
        rig::test_utils::MockTurn::tool_call(
            "1",
            "ship_op",
            json!({ "op": "collections.read", "args": { "collection": "leads" } }),
        ),
        rig::test_utils::MockTurn::text("Read the leads collection."),
    ]);
    let gw = Arc::new(InProcessGateway);
    // The VM thread (the !Send bridge) — shared by run_script across runs.
    let executor = Arc::new(script_executor::ScriptExecutor::spawn(Arc::new(InProcessGateway)));
    match agent_loop::run_step(model, &spec, gw, auth.clone(), executor.clone(), "How many leads?").await {
        Ok(out) => println!("[slice1.6] agent final: {}\n", out.text),
        Err(e) => println!("[slice1.6] ERROR: {e}\n"),
    }

    // 10b. Slice 3 (bridge): CODE-MODE in the loop. The mock model calls the
    //      `run_script` tool; the JS runs on the dedicated VM thread (deno_core
    //      is !Send), imports the scoped SDK, and its ops still pass the gate —
    //      proving code-mode composes inside rig's Send loop.
    println!("--- slice 3: code-mode in the loop (run_script via VM-thread bridge) ---");
    let coder = rig::test_utils::MockCompletionModel::new([
        rig::test_utils::MockTurn::tool_call(
            "1",
            "run_script",
            json!({ "src": "import { collections } from \"ship\"; const r = await collections.read({ collection: \"leads\" }); Deno.core.ops.op_emit(r);" }),
        ),
        rig::test_utils::MockTurn::text("Computed via code-mode."),
    ]);
    match agent_loop::run_step(coder, &spec, Arc::new(InProcessGateway), auth.clone(), executor, "summarize leads").await {
        Ok(out) => println!("[slice3] agent final: {}\n", out.text),
        Err(e) => println!("[slice3] ERROR: {e}\n"),
    }

    // 11. Slice 4: the post-edit verify gate (LSP seam). Routing is total: a
    //     `.ts` path goes to the TS backend, a `.sui` path to the SUI backend.
    //     The stub backends report no diagnostics, so a clean edit verifies.
    //     When a real backend reports an error, the same call returns
    //     `SpineError::Unverified` and the edit is refused, not accepted.
    println!("--- slice 4: post-edit verify gate (LSP seam) ---");
    for path in ["views/foo/controller.ts", "views/foo/page.sui", "weird.xyz"] {
        match lsp::verify_edit(path, "const x: number = 1;") {
            Ok(()) => println!("[slice4] {path} -> ACCEPTED (clean)"),
            Err(e) => println!("[slice4] {path} -> REFUSED: {e}"),
        }
    }
    println!();

    Ok(())
}
