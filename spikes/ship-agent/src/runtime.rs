//! The deno_core runtime — a V8 isolate whose ONLY bound effect is a wire to the
//! gateway. This is the JS half of the agent; the gate does not live here.
//!
//! `op_ship_call(op, args)` is the single host function. It carries the agent's
//! AuthContext (placed in OpState) into `gateway.call`, where the gate runs. A
//! gate denial returns as `{ error }` in the result value — a denial is a result
//! the agent handles, not a JS exception. There is exactly one door; this op is
//! the wire to it, with no policy of its own.
//!
//! Slice 1.5 runs a script via `execute_script`. The deny-by-default
//! `ModuleLoader` (for `import` of the scoped SDK) is the next slice.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use deno_core::{
    extension, op2, JsRuntime, ModuleSpecifier, OpState, PollEventLoopOptions, RuntimeOptions,
};
use serde_json::Value;

use crate::auth::AuthContext;
use crate::gateway::Gateway;
use crate::module_loader::ShipModuleLoader;
use crate::sdkgen;

/// Per-run environment, placed in OpState so the op can reach the gateway + the
/// agent's credential.
struct AgentEnv {
    gateway: Arc<dyn Gateway>,
    auth: AuthContext,
}

/// Collects values the agent emits via `op_emit`, so a headless run has an
/// observable result (a module exports, it doesn't "return").
#[derive(Default, Clone)]
struct Collector(std::rc::Rc<std::cell::RefCell<Vec<Value>>>);

/// The single bound op: a wire from JS to the gateway. Infallible at the op
/// layer — gateway errors (incl. gate denials) serialize into the result.
#[op2(async(lazy))]
#[serde]
async fn op_ship_call(
    state: Rc<RefCell<OpState>>,
    #[string] op: String,
    #[serde] args: serde_json::Value,
) -> serde_json::Value {
    let (gateway, auth) = {
        let s = state.borrow();
        let env = s.borrow::<AgentEnv>();
        (env.gateway.clone(), env.auth.clone())
    };
    match gateway.call(&op, args, &auth, crate::gateway::GatewayCallKind::First).await {
        Ok(v) => v,
        Err(e) => serde_json::json!({ "error": e.to_string() }),
    }
}

/// Headless result channel — the agent emits values it wants surfaced.
#[op2]
fn op_emit(state: &mut OpState, #[serde] value: serde_json::Value) {
    state.borrow::<Collector>().0.borrow_mut().push(value);
}

extension!(ship_ops, ops = [op_ship_call, op_emit]);

/// Wire the op to a friendly global. The op surface a script sees is exactly
/// this — one call into the one door.
const BOOTSTRAP: &str =
    "globalThis.ship_call = (op, args) => Deno.core.ops.op_ship_call(op, args ?? {});";

/// Build an isolate for one agent run and execute `script` to completion,
/// returning its (awaited) result value.
pub async fn run_script(
    gateway: Arc<dyn Gateway>,
    auth: AuthContext,
    script: String,
) -> anyhow::Result<Value> {
    let mut rt = JsRuntime::new(RuntimeOptions {
        extensions: vec![ship_ops::init()],
        ..Default::default()
    });
    rt.op_state().borrow_mut().put(AgentEnv { gateway, auth });

    rt.execute_script("[ship:bootstrap]", BOOTSTRAP)
        .map_err(|e| anyhow::anyhow!("bootstrap: {e}"))?;

    let promise = rt
        .execute_script("[ship:agent]", script)
        .map_err(|e| anyhow::anyhow!("execute: {e}"))?;

    // `resolve` returns an owned future (no self-borrow), so it composes with
    // `with_event_loop_promise`, which drives the event loop until the agent's
    // top-level promise settles (its op calls resolve through the gateway).
    let resolve = rt.resolve(promise);
    let global = rt
        .with_event_loop_promise(resolve, PollEventLoopOptions::default())
        .await?;

    deno_core::scope!(scope, &mut rt);
    let local = deno_core::v8::Local::new(scope, global);
    let value: Value = deno_core::serde_v8::from_v8(scope, local)?;
    Ok(value)
}

/// Slice 2: run an agent as an ES MODULE that imports the scoped SDK.
///
/// The SDK is generated from `grants` and is the ONLY importable module (the
/// deny-by-default loader refuses everything else). The agent's `op_emit` calls
/// are collected and returned. The gate still runs in the gateway for every op,
/// whether reached through the SDK or a raw `op_ship_call`.
pub async fn run_module(
    gateway: Arc<dyn Gateway>,
    auth: AuthContext,
    grants: Vec<String>,
    agent_src: String,
) -> anyhow::Result<Vec<Value>> {
    let sdk_src = sdkgen::generate_sdk(&grants);
    let loader = std::rc::Rc::new(ShipModuleLoader::new(sdk_src, agent_src));

    let mut rt = JsRuntime::new(RuntimeOptions {
        module_loader: Some(loader),
        extensions: vec![ship_ops::init()],
        ..Default::default()
    });
    let collector = Collector::default();
    {
        let op_state = rt.op_state();
        let mut state = op_state.borrow_mut();
        state.put(AgentEnv { gateway, auth });
        state.put(collector.clone());
    }

    let main = ModuleSpecifier::parse(crate::module_loader::MAIN)?;
    let mod_id = rt.load_main_es_module(&main).await?;
    let eval = rt.mod_evaluate(mod_id);
    rt.run_event_loop(PollEventLoopOptions::default()).await?;
    eval.await?;

    let out = collector.0.borrow().clone();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthContext;
    use crate::gateway::InProcessGateway;

    fn gw() -> Arc<dyn Gateway> { Arc::new(InProcessGateway::default()) }
    fn auth(grants: &[&str]) -> AuthContext {
        AuthContext::in_process("t", grants.iter().map(|s| s.to_string()).collect())
    }

    // Deny-by-default: importing anything but `ship` must fail to load.
    #[tokio::test(flavor = "current_thread")]
    async fn forbidden_import_is_blocked() {
        let src = r#"import x from "fs"; Deno.core.ops.op_emit({ escaped: true });"#.to_string();
        let r = run_module(gw(), auth(&["collections"]), vec!["collections".into()], src).await;
        assert!(r.is_err(), "non-ship import must be refused by the loader");
    }

    // The scoped SDK reaches granted ops; a RAW call to an ungranted op is still
    // denied by the gate. Soft layer (SDK) narrows; hard layer (gate) enforces.
    #[tokio::test(flavor = "current_thread")]
    async fn sdk_grants_through_gate_blocks_raw_ungranted() {
        let src = r#"
            import { collections } from "ship";
            const granted = await collections.read({});
            // raw attempt at an UNGRANTED C2 service — SDK never exposed
            // it; the gate denies it.
            const raw_ungranted = await Deno.core.ops.op_ship_call("runtime.dispatch", {});
            Deno.core.ops.op_emit({ granted, raw_ungranted });
        "#
        .to_string();
        let out = run_module(gw(), auth(&["collections"]), vec!["collections".into()], src)
            .await
            .expect("module should run");
        assert!(!out.is_empty(), "agent should emit a result");
        let rec = out.last().unwrap();
        assert!(rec["granted"].get("error").is_none(), "granted op must not be denied");
        assert!(rec["raw_ungranted"].get("error").is_some(), "ungranted op must be denied by the gate");
    }
}
