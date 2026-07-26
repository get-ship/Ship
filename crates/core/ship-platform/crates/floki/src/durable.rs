//! Durable code-mode — run a ship-js script as a journaled, replayable program.
//!
//! This is the first slice of Floki's durability story (the journal-wiring design,
//! Stage 1). A code-mode run is a sequence of ops the script dispatches through
//! the one door; by routing them through [`durable_execution::JournalingDispatch`]
//! every op outcome is logged to a [`durable_execution::Journal`]. Re-running the
//! same script against a *populated* journal **replays**: each op returns its
//! logged result instead of re-executing — no re-dispatch, no re-applied effect.
//!
//! The layering is transparent because both halves implement the one
//! [`ship_js::Dispatch`] seam:
//!
//! ```text
//! ship-js isolate ──op──► JournalingDispatch ──(live: miss)──► GatewayDispatch ──► gate
//!                              │  (replay: hit) ──► logged result, inner never called
//!                              ▼
//!                           Journal (the durable truth)
//! ```
//!
//! What is and isn't durable yet: the **ops** a code-mode script runs are
//! journaled here (deterministic JS replays them for free; the journal makes the
//! *effects* replay-safe). Making the **model turn itself** durable (so a
//! recovered run doesn't re-burn tokens) requires expressing the rig loop as a
//! `durable_execution::Program` — that's Stage 2, and it needs design pairing.

use std::sync::Arc;

use durable_execution::{Journal, JournalingDispatch};
use serde_json::Value;
use ship_js::Dispatch;
use ship_js::deno::ScriptExecutor;

/// Run a code-mode script DURABLY. Every op the script dispatches through `base`
/// (e.g. a [`crate::dispatch::GatewayDispatch`] capturing the gateway + auth) is
/// journaled.
///
/// Returns `(result, journal)` — the **journal is returned even on error**,
/// mirroring the engine's `execute() -> (RunStatus, Journal)`: the journal is the
/// durable truth, so the ops that ran before a script-level crash are still
/// persistable for a later resume. The `Result` is the script outcome (the values
/// it `op_emit`ted, or the ship-js execution error).
///
/// Pass `Journal::new()` for a first run; pass a populated journal to **replay**
/// — completed ops return their logged results and `base` is never called for
/// them; a divergent op (a non-deterministic branch) is rejected, never silently
/// mis-replayed. The script must dispatch the same ops in the same order.
///
/// SCOPE: only the **ops** are durable here (Stage 1). The model turn is NOT
/// journaled by this function — a code-mode script is deterministic JS, so its op
/// stream replays; making the surrounding model turn replay-safe (no token
/// re-burn on resume) is Stage 2 and requires expressing the rig loop as a
/// [`durable_execution`] `Program`.
pub async fn run_durable_script(
    executor: &ScriptExecutor,
    base: Arc<dyn Dispatch>,
    journal: Journal,
    sdk_src: String,
    main_src: String,
) -> (anyhow::Result<Vec<Value>>, Journal) {
    let journaling = Arc::new(JournalingDispatch::new(base, journal));
    let dispatch: Arc<dyn Dispatch> = journaling.clone();
    let result = executor.run(dispatch, sdk_src, main_src).await;
    // Snapshot (not into_journal): the executor briefly co-owns the Arc on the VM
    // thread, so by-value recovery would race; snapshot clones through the lock.
    // Taken unconditionally so a partial run's journal survives a script error.
    (result, journaling.snapshot())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;
    use ship_js::DispatchError;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A dispatch that counts how many ops it actually executed and returns a
    /// deterministic echo. The count is what proves replay doesn't re-dispatch.
    struct Counting {
        calls: Arc<AtomicU32>,
    }
    #[async_trait]
    impl Dispatch for Counting {
        async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(json!({ "op": op, "args": args, "ok": true }))
        }
    }

    const SDK: &str =
        r#"export const call = (op, args) => Deno.core.ops.op_ship_call(op, args ?? {});"#;
    // Two ops in a fixed order, then emit both results.
    const SRC: &str = r#"
        import { call } from "ship";
        const a = await call("collections.read", { collection: "leads" });
        const b = await call("collections.put", { collection: "leads", data: {} });
        Deno.core.ops.op_emit({ a, b });
    "#;

    // The live frontier: each op the script dispatches is journaled in order.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn code_mode_ops_are_journaled_in_order() {
        let exec = ScriptExecutor::spawn();
        let calls = Arc::new(AtomicU32::new(0));
        let base: Arc<dyn Dispatch> = Arc::new(Counting {
            calls: calls.clone(),
        });
        let (emitted, journal) =
            run_durable_script(&exec, base, Journal::new(), SDK.into(), SRC.into()).await;
        let emitted = emitted.expect("durable run completes");
        assert_eq!(emitted.len(), 1, "the script emitted once");
        assert_eq!(journal.entries().len(), 2, "both ops are journaled");
        assert_eq!(journal.entries()[0].op, "collections.read");
        assert_eq!(journal.entries()[1].op, "collections.put");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "the live run dispatched both ops"
        );
    }

    // Replay: re-running against the populated journal returns logged results and
    // NEVER re-dispatches — the durable-execution thesis at code-mode altitude.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn replay_returns_logged_results_without_re_dispatching() {
        let exec = ScriptExecutor::spawn();

        // Run 1 — live, populate the journal.
        let calls1 = Arc::new(AtomicU32::new(0));
        let base1: Arc<dyn Dispatch> = Arc::new(Counting {
            calls: calls1.clone(),
        });
        let (emitted1, journal) =
            run_durable_script(&exec, base1, Journal::new(), SDK.into(), SRC.into()).await;
        let emitted1 = emitted1.expect("live run completes");
        assert_eq!(calls1.load(Ordering::SeqCst), 2);

        // Run 2 — replay with the populated journal. The JS re-executes, but every
        // op is served from the journal: the inner dispatch must be hit ZERO times.
        let calls2 = Arc::new(AtomicU32::new(0));
        let base2: Arc<dyn Dispatch> = Arc::new(Counting {
            calls: calls2.clone(),
        });
        let (emitted2, journal2) =
            run_durable_script(&exec, base2, journal, SDK.into(), SRC.into()).await;
        let emitted2 = emitted2.expect("replay completes");

        assert_eq!(
            calls2.load(Ordering::SeqCst),
            0,
            "replay must NOT re-dispatch any op — they come from the journal",
        );
        assert_eq!(
            emitted1, emitted2,
            "replay reproduces the same emitted values from the logged results",
        );
        assert_eq!(
            journal2.entries().len(),
            2,
            "the journal is unchanged on replay"
        );
    }

    // The journal is returned even when the script crashes: ops that ran before
    // the crash are journaled, so a resume can pick up from there.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn partial_journal_survives_a_script_error() {
        let exec = ScriptExecutor::spawn();
        let calls = Arc::new(AtomicU32::new(0));
        let base: Arc<dyn Dispatch> = Arc::new(Counting {
            calls: calls.clone(),
        });
        // One op runs, then the script throws — the run errors mid-flight.
        let crashing = r#"
            import { call } from "ship";
            await call("collections.read", { collection: "leads" });
            throw new Error("boom");
        "#;
        let (result, journal) =
            run_durable_script(&exec, base, Journal::new(), SDK.into(), crashing.into()).await;
        assert!(result.is_err(), "the throwing script surfaces as an Err");
        assert_eq!(
            journal.entries().len(),
            1,
            "the op that ran before the crash is journaled (partial progress survives)",
        );
        assert_eq!(journal.entries()[0].op, "collections.read");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "exactly the pre-crash op dispatched"
        );
    }
}
