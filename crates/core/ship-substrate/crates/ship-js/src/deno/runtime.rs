//! The deno_core runtime — a V8 isolate whose ONLY bound effect is a wire to the
//! host. This is the JS engine; no policy lives here.
//!
//! `op_ship_call(op, args)` is the single host function. It reads the run's
//! [`Dispatch`] from `OpState` and forwards `(op, args)`. A host failure is
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use deno_core::{
    extension, op2, JsRuntime, ModuleSpecifier, OpState, PollEventLoopOptions, RuntimeOptions,
};
use serde_json::Value;

use super::module_loader::{ShipModuleLoader, MAIN};
use crate::dispatch::Dispatch;

/// Bounds on a single JS run — wall-clock deadline and V8 heap cap. A run that
/// exceeds either is **terminated** (the daemon stays up); the bound fires,
/// `run_module` / `run_script` returns an `Err`, the SSR / agent layer falls
/// back. Without this, an infinite loop, a stuck host op, or unbounded
/// allocation in a controller can wedge or OOM the process (see the lineage
/// `41c2339f6` live() leak + `cbfc22028` reconnect storm).
///
/// Both fields are `0`-disabled (no deadline / no heap cap). The defaults
/// (`Default::default()`) are conservative for first-paint SSR.
#[derive(Debug, Clone, Copy)]
pub struct RunLimits {
    /// Wall-clock deadline for the run. On expiry, execution is terminated via
    /// `IsolateHandle::terminate_execution()`; the caller's future returns an
    /// `Err(RunError::Terminated { reason: WallClockDeadline, .. })` instead of
    /// hanging. `Duration::ZERO` disables the deadline.
    pub deadline: Duration,
    /// V8 heap cap in bytes. Initial limit is `max_heap_bytes`; near it, V8
    /// fires `add_near_heap_limit_callback`, which terminates the isolate
    /// instead of letting V8 OOM the host. `0` disables the cap.
    pub max_heap_bytes: usize,
}

impl Default for RunLimits {
    /// 5 s wall-clock + 256 MiB heap — sane for first-paint SSR. Hosts that
    /// need different values override via `FromEnv` or construct directly.
    fn default() -> Self {
        Self {
            deadline: Duration::from_secs(5),
            max_heap_bytes: 256 * 1024 * 1024,
        }
    }
}

impl RunLimits {
    /// Read overrides from the environment. `SHIP_JS_DEADLINE_MS` and
    /// `SHIP_JS_MAX_HEAP_MB`. Missing / unparseable → keep `self`'s value.
    /// Used by `run_module` / `run_script` callers that want host-wide
    /// tunability without plumbing a config through.
    pub fn from_env_or(self) -> Self {
        let mut out = self;
        if let Ok(ms) = std::env::var("SHIP_JS_DEADLINE_MS")
            && let Ok(ms) = ms.parse::<u64>()
        {
            out.deadline = Duration::from_millis(ms);
        }
        if let Ok(mb) = std::env::var("SHIP_JS_MAX_HEAP_MB")
            && let Ok(mb) = mb.parse::<usize>()
        {
            out.max_heap_bytes = mb * 1024 * 1024;
        }
        out
    }

    /// True if any bound is configured (non-zero). When false, callers may
    /// skip the bound machinery for raw speed.
    pub fn is_enabled(&self) -> bool {
        !self.deadline.is_zero() || self.max_heap_bytes > 0
    }
}

/// Why a JS run was terminated by the bound (vs. a normal V8 error). The SSR
/// / agent host treats these as a *clean* failure — the daemon is up, the
/// isolate was killed, and the caller falls back / retries. They are NOT a
/// reason to panic the process or 500 the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminateReason {
    /// Wall-clock deadline expired. The isolate was killed via
    /// `IsolateHandle::terminate_execution()` from a `tokio::time::sleep`.
    WallClockDeadline,
    /// V8 approached the configured heap cap; the near-limit callback
    /// terminated the isolate to prevent a host OOM.
    NearHeapLimit,
}

/// The error a bounded run returns. Wraps V8 errors AND the bound's kill.
#[derive(Debug)]
pub enum RunError {
    /// A normal V8 / module / op failure — the program erred. Maps to a 500
    /// or a retry per the caller's policy.
    V8(anyhow::Error),
    /// The bound fired — the isolate was killed. `elapsed` is wall-clock time
    /// spent in the run before termination; `reason` says which bound fired.
    /// Maps to a clean fallback (e.g. CSR shell for SSR).
    Terminated {
        reason: TerminateReason,
        elapsed: Duration,
        /// V8's "execution terminated" diagnostic, if surfaced — useful in
        /// `tracing` / `eprintln` for the failure log; not user-facing.
        source: Option<anyhow::Error>,
    },
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunError::V8(e) => write!(f, "v8: {e:#}"),
            RunError::Terminated { reason, elapsed, .. } => {
                let r = match reason {
                    TerminateReason::WallClockDeadline => "wall-clock deadline",
                    TerminateReason::NearHeapLimit => "heap cap",
                };
                write!(f, "{r} terminated after {elapsed:?}")
            }
        }
    }
}

impl std::error::Error for RunError {}

impl From<anyhow::Error> for RunError {
    fn from(e: anyhow::Error) -> Self {
        RunError::V8(e)
    }
}

/// Per-run environment in OpState: the one wire back to the host.
struct HostEnv {
    dispatch: Arc<dyn Dispatch>,
}

/// Collects values the isolate emits via `op_emit`, so a headless run has an
/// observable result (a module exports, it doesn't "return").
#[derive(Default, Clone)]
struct Collector(Rc<RefCell<Vec<Value>>>);

/// The single bound op: a wire from JS to the host. Infallible at the op layer —
/// a host `Err(DispatchError)` is mapped uniformly to `{ error }` in the result.
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
    match host.call(&op, args).await {
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

/// Apply the heap cap (`max_heap_bytes > 0`) and the near-heap-limit terminate
/// callback. Returns `true` if the cap was registered; the callback captures a
/// shared `AtomicBool` so the caller can tell heap-pressure from wall-clock
/// after the run.
fn install_heap_cap(rt: &mut JsRuntime, max_heap_bytes: usize) -> Option<Arc<AtomicBool>> {
    if max_heap_bytes == 0 {
        return None;
    }
    let fired = Arc::new(AtomicBool::new(false));
    let fired_inner = fired.clone();
    let handle = rt.v8_isolate().thread_safe_handle();
    rt.add_near_heap_limit_callback(move |current, _initial| {
        fired_inner.store(true, Ordering::SeqCst);
        // Terminate execution instead of letting V8 raise the limit again.
        // The host OOMs have to be prevented, not absorbed.
        let _ = handle.terminate_execution();
        // Returning a higher limit is required by V8's callback contract even
        // though we already terminated — V8 reads this value.
        current.saturating_mul(2).max(max_heap_bytes)
    });
    Some(fired)
}

/// Race an isolate-work future against the wall-clock deadline. The deadline
/// is enforced by a DEDICATED OS thread that calls
/// `IsolateHandle::terminate_execution()` when the timer fires — necessary
/// because V8 runs synchronously inside the tokio poll, so a
/// `tokio::time::sleep` on the same runtime is starved and never observed.
/// After the work future resolves, the deadline thread's `terminate_execution`
/// becomes a no-op on an already-terminated isolate.
///
/// `heap_fired`, if `Some`, is set by the near-heap-limit callback; checked
/// AFTER the run to attribute the kill to the right reason.
async fn run_with_deadline<F, T>(
    deadline: Duration,
    handle: deno_core::v8::IsolateHandle,
    heap_fired: Option<Arc<AtomicBool>>,
    work: F,
) -> Result<T, RunError>
where
    F: std::future::Future<Output = anyhow::Result<T>>,
{
    if deadline.is_zero() {
        return work.await.map_err(RunError::V8);
    }
    let start = Instant::now();
    // is `Send + Sync`; terminate_execution is callable from any thread, and
    // V8 observes the flag at the next statement boundary.
    //
    // We also hand the thread a `Sender<()>` so a clean (non-terminated) run
    // can wake it up and let it exit EARLY — otherwise it sleeps the full
    // deadline then fires a no-op terminate, wasting up to `deadline` ms.
    let (cancel_tx, cancel_rx) = std::sync::mpsc::channel::<()>();
    let deadline_handle = handle;
    let _deadline_thread = std::thread::Builder::new()
        .name("ship-js-deadline".into())
        .spawn(move || {
            // `recv_timeout` returns `Err(RecvTimeoutError::Timeout)` if no
            // signal arrives in time → fire terminate. Otherwise the run
            // completed cleanly; exit silently.
            match cancel_rx.recv_timeout(deadline) {
                Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    let _ = deadline_handle.terminate_execution();
                }
            }
        })
        .expect("spawn deadline thread");
    let outcome = work.await;
    // Signal the deadline thread to exit (no-op if it already fired).
    let _ = cancel_tx.send(());
    // The thread handle is dropped here; the thread either already exited
    // (sent case or already-fired case) or is exiting now (recv_timeout
    // returns Ok). We don't join — no value, and it must not block `await`.
    drop(_deadline_thread);
    drop(cancel_tx);
    match outcome {
        Ok(t) => Ok(t),
        Err(e) => {
            // Distinguish "terminated by bound" from a normal V8 error: a
            // well-behaved run that completes without firing the bound will
            // have Ok(_); an infinite loop / OOM will be Err(_). The bound is
            // the only reason execution halts abruptly mid-eval — deno_core
            // reports such kills as `mod_evaluate` returning Err.
            let elapsed = start.elapsed();
            let reason = match heap_fired {
                Some(f) if f.load(Ordering::SeqCst) => TerminateReason::NearHeapLimit,
                _ => TerminateReason::WallClockDeadline,
            };
            Err(RunError::Terminated { reason, elapsed, source: Some(e) })
        }
    }
}

/// Bounded script execution: wall-clock `deadline` + V8 heap `max_heap_bytes`.
/// The bound fires → kill the isolate → return `Err(RunError::Terminated)`.
pub async fn run_script_with_limits(
    dispatch: Arc<dyn Dispatch>,
    script: String,
    limits: RunLimits,
) -> Result<Value, RunError> {
    let limits = limits.from_env_or();
    let mut rt = JsRuntime::new(RuntimeOptions {
        extensions: vec![ship_ops::init()],
        create_params: limits.max_heap_bytes_max_heap(),
        ..Default::default()
    });
    let heap_fired = install_heap_cap(&mut rt, limits.max_heap_bytes);
    rt.op_state().borrow_mut().put(HostEnv { dispatch });

    let handle = rt.v8_isolate().thread_safe_handle();
    run_with_deadline(limits.deadline, handle, heap_fired, async {
        rt.execute_script("[ship:bootstrap]", BOOTSTRAP)
            .map_err(|e| anyhow::anyhow!("bootstrap: {e}"))?;
        let promise = rt
            .execute_script("[ship:agent]", script)
            .map_err(|e| anyhow::anyhow!("execute: {e}"))?;
        // `resolve` returns an owned future (no self-borrow), so it composes
        // with `with_event_loop_promise`, which drives the event loop until
        // the top-level promise settles (op calls resolve through the host).
        let resolve = rt.resolve(promise);
        let global = rt
            .with_event_loop_promise(resolve, PollEventLoopOptions::default())
            .await?;
        deno_core::scope!(scope, &mut rt);
        let local = deno_core::v8::Local::new(scope, global);
        let value: Value = deno_core::serde_v8::from_v8(scope, local)?;
        Ok(value)
    })
    .await
}

/// Back-compat wrapper: same as `run_script_with_limits(.., RunLimits::default())`.
/// Existing callers (incl. unit tests) get the **bounded** behavior for free —
/// infinite loops / unbounded allocation can no longer wedge the engine.
pub async fn run_script(dispatch: Arc<dyn Dispatch>, script: String) -> anyhow::Result<Value> {
    run_script_with_limits(dispatch, script, RunLimits::default())
        .await
        .map_err(|e| match e {
            RunError::V8(inner) => inner,
            RunError::Terminated { source: Some(inner), .. } => inner,
            RunError::Terminated { reason, elapsed, .. } => {
                anyhow::anyhow!("script terminated by {} after {:?}", match reason {
                    TerminateReason::WallClockDeadline => "wall-clock deadline",
                    TerminateReason::NearHeapLimit => "heap cap",
                }, elapsed)
            }
        })
}

impl RunLimits {
    /// Build the `v8::CreateParams` carrying the heap cap (or `None` if disabled).
    /// V8 distinguishes the **total** heap limit (set by `heap_limits`) from
    /// the **old generation** limit (set by `set_max_old_generation_size`).
    /// A long-lived runaway (e.g. a string concat that grows forever) lives
    /// in the old generation; without constraining it separately, the script
    /// can push past the old-gen default (~2 GB) and trigger V8's hard OOM
    /// abort even though the total heap cap was respected. We set BOTH.
    fn max_heap_bytes_max_heap(&self) -> Option<deno_core::v8::CreateParams> {
        if self.max_heap_bytes == 0 {
            return None;
        }
        Some(
            deno_core::v8::Isolate::create_params()
                .heap_limits(0, self.max_heap_bytes)
                .set_max_old_generation_size_in_bytes(self.max_heap_bytes),
        )
    }
}

/// Run a program as an ES MODULE that may `import "ship"` for the scoped SDK.
///
/// `sdk_src` is the module served for `import "ship"` (the host decides its
/// contents — e.g. scoping it to grants); `main_src` is the entry module. The
/// deny-by-default loader refuses every other import. The program's `op_emit`
/// calls are collected and returned. Every op still routes through the host's
/// `Dispatch`, whether reached through the SDK or a raw `op_ship_call`.
/// Bounded module execution: wall-clock `deadline` + V8 heap `max_heap_bytes`.
/// The bound fires → kill the isolate → return `Err(RunError::Terminated)`.
pub async fn run_module_with_limits(
    dispatch: Arc<dyn Dispatch>,
    sdk_src: String,
    main_src: String,
    limits: RunLimits,
) -> Result<Vec<Value>, RunError> {
    let limits = limits.from_env_or();
    let loader = Rc::new(ShipModuleLoader::new(sdk_src, main_src));

    let mut rt = JsRuntime::new(RuntimeOptions {
        module_loader: Some(loader),
        extensions: vec![ship_ops::init()],
        create_params: limits.max_heap_bytes_max_heap(),
        ..Default::default()
    });
    let heap_fired = install_heap_cap(&mut rt, limits.max_heap_bytes);
    let collector = Collector::default();
    {
        let op_state = rt.op_state();
        let mut state = op_state.borrow_mut();
        state.put(HostEnv { dispatch });
        state.put(collector.clone());
    }

    let handle = rt.v8_isolate().thread_safe_handle();
    run_with_deadline(limits.deadline, handle, heap_fired, async {
        let main = ModuleSpecifier::parse(MAIN)?;
        let mod_id = rt.load_main_es_module(&main).await?;
        let eval = rt.mod_evaluate(mod_id);
        rt.run_event_loop(PollEventLoopOptions::default()).await?;
        eval.await?;
        let out = collector.0.borrow().clone();
        Ok(out)
    })
    .await
}

/// Back-compat wrapper: bounded by `RunLimits::default()`. Existing callers
/// (incl. the deck-ssr `run::render` and the `ScriptExecutor` bridge) get the
/// bounded behavior automatically — a runaway controller no longer wedges the
/// isolate or OOMs the host.
pub async fn run_module(
    dispatch: Arc<dyn Dispatch>,
    sdk_src: String,
    main_src: String,
) -> anyhow::Result<Vec<Value>> {
    run_module_with_limits(dispatch, sdk_src, main_src, RunLimits::default())
        .await
        .map_err(|e| match e {
            RunError::V8(inner) => inner,
            RunError::Terminated { source: Some(inner), .. } => inner,
            RunError::Terminated { reason, elapsed, .. } => {
                anyhow::anyhow!("module terminated by {} after {:?}", match reason {
                    TerminateReason::WallClockDeadline => "wall-clock deadline",
                    TerminateReason::NearHeapLimit => "heap cap",
                }, elapsed)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::DispatchError;
    use serde_json::json;

    /// A stub host: echoes the op + args back as data, so a test can prove the
    /// wire end-to-end (JS → op → host → JS) WITHOUT any gate or auth. The whole
    /// point of the seam is that ship-js needs nothing more than this to run.
    struct Echo;
    #[async_trait::async_trait]
    impl Dispatch for Echo {
        async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
            Ok(json!({ "op": op, "args": args, "ok": true }))
        }
    }

    /// A host that denies a specific op (the rest echo). Models the gate's
    /// "deny one ungranted op, allow the rest" — as the isolate observes it.
    struct DenyOp(&'static str);
    #[async_trait::async_trait]
    impl Dispatch for DenyOp {
        async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
            if op == self.0 {
                Err(DispatchError::Terminal(format!("denied: {op}")))
            } else {
                Ok(json!({ "op": op, "args": args, "ok": true }))
            }
        }
    }

    fn arc(d: impl Dispatch + 'static) -> Arc<dyn Dispatch> {
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

    // Bound: a script that infinite-loops is terminated by the wall-clock
    // deadline. The bound's whole reason for existing — without it, the
    // controller hangs forever. We give it 200 ms and assert it returns within
    // 5 s with a clean `Terminated { reason: WallClockDeadline }`.
    #[tokio::test(flavor = "current_thread")]
    async fn run_script_wall_clock_deadline_terminates_infinite_loop() {
        let src = "while (true) {}".to_string();
        let t0 = std::time::Instant::now();
        let r = run_script_with_limits(
            arc(Echo),
            src,
            RunLimits { deadline: Duration::from_millis(200), max_heap_bytes: 0 },
        )
        .await;
        let dt = t0.elapsed();
        // The bound must have fired — not hung. 5 s is generous; in practice
        // this returns in ~200–300 ms.
        assert!(dt < Duration::from_secs(5), "bounded run hung: {dt:?}");
        match r {
            Err(RunError::Terminated { reason, .. }) => {
                assert_eq!(
                    reason,
                    TerminateReason::WallClockDeadline,
                    "expected wall-clock kill, got {reason:?}"
                );
            }
            other => panic!("expected Terminated(WallClockDeadline), got {other:?}"),
        }
    }

    // Bound: an allocation-heavy loop is terminated by the heap cap. We cap
    // V8 at 8 MiB and let the script grow a string until V8 fires the near-
    // heap-limit callback (which calls `terminate_execution`). The result
    // must be `Terminated { reason: NearHeapLimit }` within a few seconds.
    #[tokio::test(flavor = "current_thread")]
    async fn run_script_heap_cap_terminates_unbounded_allocation() {
        let src = r#"let s = ""; while (true) { s += "x".repeat(1024); }"#.to_string();
        let t0 = std::time::Instant::now();
        let r = run_script_with_limits(
            arc(Echo),
            src,
            RunLimits { deadline: Duration::from_secs(30), max_heap_bytes: 8 * 1024 * 1024 },
        )
        .await;
        let dt = t0.elapsed();
        assert!(dt < Duration::from_secs(10), "heap-cap run took too long: {dt:?}");
        match r {
            Err(RunError::Terminated { reason, .. }) => {
                assert_eq!(
                    reason,
                    TerminateReason::NearHeapLimit,
                    "expected heap-cap kill, got {reason:?}"
                );
            }
            other => panic!("expected Terminated(NearHeapLimit), got {other:?}"),
        }
    }

    // Bound: a well-behaved script completes normally even WITH the bound
    // configured (proves the bound doesn't poison well-behaved runs).
    #[tokio::test(flavor = "current_thread")]
    async fn run_script_well_behaved_completes_with_bound_active() {
        let src = r#"(async () => { return 1 + 1; })()"#.to_string();
        let v = run_script_with_limits(
            arc(Echo),
            src,
            RunLimits { deadline: Duration::from_secs(2), max_heap_bytes: 8 * 1024 * 1024 },
        )
        .await
        .expect("well-behaved script must complete");
        assert_eq!(v, json!(2));
    }

    // Bound: `run_module` with an infinite-loop main module terminates by
    // wall-clock deadline — the SSR-relevant path (deck-ssr's `render`
    // goes through this code).
    #[tokio::test(flavor = "current_thread")]
    async fn run_module_wall_clock_deadline_terminates_infinite_main() {
        let main = r#"while (true) {}"#.to_string();
        let t0 = std::time::Instant::now();
        let r = run_module_with_limits(
            arc(Echo),
            SDK.to_string(),
            main,
            RunLimits { deadline: Duration::from_millis(200), max_heap_bytes: 0 },
        )
        .await;
        let dt = t0.elapsed();
        assert!(dt < Duration::from_secs(5), "bounded module run hung: {dt:?}");
        match r {
            Err(RunError::Terminated { reason, .. }) => {
                assert_eq!(reason, TerminateReason::WallClockDeadline);
            }
            other => panic!("expected Terminated(WallClockDeadline), got {other:?}"),
        }
    }

    // Bound: `run_module` with an unbounded-allocation main module terminates
    // by heap cap (V8 fires near-heap-limit, callback terminates). This is
    // the deck-ssr `render` path's engine layer.
    #[tokio::test(flavor = "current_thread")]
    async fn run_module_heap_cap_terminates_unbounded_allocation() {
        let main = r#"
            let s = "";
            while (true) {
                s += "x".repeat(4096);
            }
        "#
        .to_string();
        let t0 = std::time::Instant::now();
        let r = run_module_with_limits(
            arc(Echo),
            SDK.to_string(),
            main,
            RunLimits { deadline: Duration::from_secs(30), max_heap_bytes: 4 * 1024 * 1024 },
        )
        .await;
        let dt = t0.elapsed();
        assert!(dt < Duration::from_secs(10), "heap-cap module run took too long: {dt:?}");
        match r {
            Err(RunError::Terminated { reason, .. }) => {
                assert_eq!(
                    reason,
                    TerminateReason::NearHeapLimit,
                    "expected heap-cap kill, got {reason:?}"
                );
            }
            other => panic!("expected Terminated(NearHeapLimit), got {other:?}"),
        }
    }

    // Bound: `from_env_or` reads `SHIP_JS_DEADLINE_MS` when present.
    #[test]
    fn run_limits_from_env_or_reads_overrides() {
        // SAFETY: single-threaded test; we set/restore env vars under a guard.
        // SAFETY: set_var/unset_var are safe in single-threaded test context.
        unsafe {
            std::env::set_var("SHIP_JS_DEADLINE_MS", "777");
            std::env::set_var("SHIP_JS_MAX_HEAP_MB", "16");
        }
        let l = RunLimits {
            deadline: Duration::from_secs(5),
            max_heap_bytes: 256 * 1024 * 1024,
        }
        .from_env_or();
        assert_eq!(l.deadline, Duration::from_millis(777));
        assert_eq!(l.max_heap_bytes, 16 * 1024 * 1024);
        // SAFETY: cleanup
        unsafe {
            std::env::remove_var("SHIP_JS_DEADLINE_MS");
            std::env::remove_var("SHIP_JS_MAX_HEAP_MB");
        }
    }
}
