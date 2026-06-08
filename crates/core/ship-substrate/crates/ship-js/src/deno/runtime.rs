//! The deno_core runtime — a V8 isolate whose ONLY bound effect is a wire to the
//! host. This is the JS engine; no policy lives here.
//!
//! `op_ship_call(op, args)` is the single host function. It reads the run's
//! [`HostDispatch`] from `OpState` and forwards `(op, args)`. A host failure is
//! mapped to `{ error }` in the result `Value` — a denial is data the program
//! handles, not a JS exception. There is exactly one door, host-side, and this
//! op is the wire to it with no policy of its own.
//!
//! Two entry points: [`run_script`] runs an inline script to its top-level
//! promise; [`run_module`] runs an ES module that may `import "ship"` (the
//! deny-by-default [`ShipModuleLoader`] refuses everything else) and surfaces the
//! values it emits via `op_emit`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use deno_core::{
    extension, op2, JsRuntime, ModuleSpecifier, OpState, PollEventLoopOptions, RuntimeOptions,
};
use serde_json::Value;

use super::module_loader::{ShipModuleLoader, MAIN};
use super::seam::HostDispatch;

/// Per-run environment in OpState: the one wire back to the host.
struct HostEnv {
    dispatch: Arc<dyn HostDispatch>,
}

/// Collects values the isolate emits via `op_emit`, so a headless run has an
/// observable result (a module exports, it doesn't "return").
#[derive(Default, Clone)]
struct Collector(Rc<RefCell<Vec<Value>>>);

/// The single bound op: a wire from JS to the host. Infallible at the op layer —
/// a host `Err(HostError)` is mapped uniformly to `{ error }` in the result.
#[op2(async(lazy))]
#[serde]
async fn op_ship_call(
    state: Rc<RefCell<OpState>>,
    #[string] op: String,
    // op2's `#[serde]` matches the type literally — it must read the
    // fully-qualified `serde_json::Value`, not the bare imported alias.
    #[serde] args: serde_json::Value,
) -> serde_json::Value {
    let host = {
        let s = state.borrow();
        s.borrow::<HostEnv>().dispatch.clone()
    };
    match host.dispatch(&op, args).await {
        Ok(v) => v,
        Err(e) => serde_json::json!({ "error": e.to_string() }),
    }
}

/// Headless result channel — the isolate emits values it wants surfaced.
#[op2]
fn op_emit(state: &mut OpState, #[serde] value: serde_json::Value) {
    state.borrow::<Collector>().0.borrow_mut().push(value);
}

extension!(ship_ops, ops = [op_ship_call, op_emit]);

/// Wire the op to a friendly global. The op surface a script sees is exactly
/// this — one call into the one door.
const BOOTSTRAP: &str =
    "globalThis.ship_call = (op, args) => Deno.core.ops.op_ship_call(op, args ?? {});";

/// Build an isolate for one run and execute `script` to completion, returning
/// its (awaited) top-level value. `ship_call(op, args)` is bound as a global.
pub async fn run_script(dispatch: Arc<dyn HostDispatch>, script: String) -> anyhow::Result<Value> {
    let mut rt = JsRuntime::new(RuntimeOptions {
        extensions: vec![ship_ops::init()],
        ..Default::default()
    });
    rt.op_state().borrow_mut().put(HostEnv { dispatch });

    rt.execute_script("[ship:bootstrap]", BOOTSTRAP)
        .map_err(|e| anyhow::anyhow!("bootstrap: {e}"))?;

    let promise = rt
        .execute_script("[ship:agent]", script)
        .map_err(|e| anyhow::anyhow!("execute: {e}"))?;

    // `resolve` returns an owned future (no self-borrow), so it composes with
    // `with_event_loop_promise`, which drives the event loop until the top-level
    // promise settles (its op calls resolve through the host).
    let resolve = rt.resolve(promise);
    let global = rt
        .with_event_loop_promise(resolve, PollEventLoopOptions::default())
        .await?;

    deno_core::scope!(scope, &mut rt);
    let local = deno_core::v8::Local::new(scope, global);
    let value: Value = deno_core::serde_v8::from_v8(scope, local)?;
    Ok(value)
}

/// Run a program as an ES MODULE that may `import "ship"` for the scoped SDK.
///
/// `sdk_src` is the module served for `import "ship"` (the host decides its
/// contents — e.g. scoping it to grants); `main_src` is the entry module. The
/// deny-by-default loader refuses every other import. The program's `op_emit`
/// calls are collected and returned. Every op still routes through the host's
/// `HostDispatch`, whether reached through the SDK or a raw `op_ship_call`.
pub async fn run_module(
    dispatch: Arc<dyn HostDispatch>,
    sdk_src: String,
    main_src: String,
) -> anyhow::Result<Vec<Value>> {
    let loader = Rc::new(ShipModuleLoader::new(sdk_src, main_src));

    let mut rt = JsRuntime::new(RuntimeOptions {
        module_loader: Some(loader),
        extensions: vec![ship_ops::init()],
        ..Default::default()
    });
    let collector = Collector::default();
    {
        let op_state = rt.op_state();
        let mut state = op_state.borrow_mut();
        state.put(HostEnv { dispatch });
        state.put(collector.clone());
    }

    let main = ModuleSpecifier::parse(MAIN)?;
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
    use crate::deno::seam::HostError;
    use serde_json::json;

    /// A stub host: echoes the op + args back as data, so a test can prove the
    /// wire end-to-end (JS → op → host → JS) WITHOUT any gate or auth. The whole
    /// point of the seam is that ship-js needs nothing more than this to run.
    struct Echo;
    #[async_trait::async_trait]
    impl HostDispatch for Echo {
        async fn dispatch(&self, op: &str, args: Value) -> Result<Value, HostError> {
            Ok(json!({ "op": op, "args": args, "ok": true }))
        }
    }

    /// A host that denies a specific op (the rest echo). Models the gate's
    /// "deny one ungranted op, allow the rest" — as the isolate observes it.
    struct DenyOp(&'static str);
    #[async_trait::async_trait]
    impl HostDispatch for DenyOp {
        async fn dispatch(&self, op: &str, args: Value) -> Result<Value, HostError> {
            if op == self.0 {
                Err(HostError::new(format!("denied: {op}")))
            } else {
                Ok(json!({ "op": op, "args": args, "ok": true }))
            }
        }
    }

    fn arc(d: impl HostDispatch + 'static) -> Arc<dyn HostDispatch> {
        Arc::new(d)
    }

    // A trivial SDK module: a thin wrapper over the raw op. The host decides what
    // this contains; the loader just serves it for `import "ship"`.
    const SDK: &str = r#"export const call = (op, args) => Deno.core.ops.op_ship_call(op, args ?? {});"#;

    // run_script: the bound `ship_call` global reaches the host and the awaited
    // result is the host's Value — proving the one-wire effect channel works.
    #[tokio::test(flavor = "current_thread")]
    async fn run_script_reaches_the_host_and_returns_its_value() {
        let src = r#"(async () => { return await ship_call("demo.op", { x: 1 }); })()"#.to_string();
        let v = run_script(arc(Echo), src).await.expect("script runs");
        assert_eq!(v["op"], "demo.op");
        assert_eq!(v["args"]["x"], 1);
        assert_eq!(v["ok"], true);
    }

    // run_module: an ES module imports the SDK, calls an op, and emits the
    // result — proving import + op + emit-collection compose. The emitted value
    // is exactly what the host returned.
    #[tokio::test(flavor = "current_thread")]
    async fn run_module_imports_sdk_calls_op_and_collects_emits() {
        let main = r#"
            import { call } from "ship";
            const r = await call("collections.read", { id: 7 });
            Deno.core.ops.op_emit(r);
        "#
        .to_string();
        let out = run_module(arc(Echo), SDK.to_string(), main)
            .await
            .expect("module runs");
        assert_eq!(out.len(), 1, "exactly one emit");
        assert_eq!(out[0]["op"], "collections.read");
        assert_eq!(out[0]["args"]["id"], 7);
    }

    // Deny-by-default imports: anything but `ship` must fail to LOAD (the module
    // never evaluates), surfacing as an error from run_module — the engine's
    // import-side closed world, independent of any host policy.
    #[tokio::test(flavor = "current_thread")]
    async fn forbidden_import_is_blocked() {
        let main = r#"import x from "fs"; Deno.core.ops.op_emit({ escaped: true });"#.to_string();
        let r = run_module(arc(Echo), SDK.to_string(), main).await;
        assert!(r.is_err(), "non-ship import must be refused by the loader");
    }

    // The two layers, at the engine boundary: the SDK (soft) narrows what the
    // program can name, but a RAW `op_ship_call` to a host-denied op is still
    // denied (hard) — surfaced as `{ error }` DATA, not an exception, so the
    // program keeps running and observes it. This is the engine-level twin of
    // the spike's `sdk_grants_through_gate_blocks_raw_ungranted`; the real gate
    // semantics stay host-side (the DenyOp stub stands in for the gate here).
    #[tokio::test(flavor = "current_thread")]
    async fn raw_call_to_a_denied_op_is_observable_data_not_an_exception() {
        let main = r#"
            import { call } from "ship";
            const granted = await call("collections.read", {});
            // raw attempt at the host-denied op — the program SEES the denial.
            const denied = await Deno.core.ops.op_ship_call("account.rotate_key", {});
            Deno.core.ops.op_emit({ granted, denied });
        "#
        .to_string();
        let out = run_module(arc(DenyOp("account.rotate_key")), SDK.to_string(), main)
            .await
            .expect("module runs to completion despite the denial");
        assert_eq!(out.len(), 1);
        assert!(out[0]["granted"]["error"].is_null(), "allowed op carries no error");
        assert!(out[0]["denied"]["error"].is_string(), "denied op surfaces `{{ error }}` as data");
    }
}
