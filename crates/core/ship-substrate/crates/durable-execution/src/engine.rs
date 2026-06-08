//! The durable-execution engine.
//!
//! A *program* is a deterministic sequence of journaled op-calls (in production:
//! a ship-js script or an agent turn; here: a [`Program`] impl). Every op result
//! is written to the **journal**. On resume the program **re-runs from the top**,
//! and ops that already completed return their *logged* results instead of
//! re-executing — so a recovered run never re-burns work (a model call is just a
//! journaled op; its completion replays from the log). A *suspending* op (human
//! checkpoint, delay, sub-run await) parks the run; when the external decision
//! arrives the entry is resolved and the program replays past the suspend point.
//!
//! Invariants this enforces:
//! - **Replay never re-executes a completed op** (no re-burn).
//! - **Determinism**: the Nth op-call maps to the Nth journal entry, so the
//!   program MUST be deterministic — all non-determinism (model, time, random,
//!   network) goes through an op, hence the journal.

use std::future::Future;
use std::sync::Arc;

use serde_json::Value;

use ship_js::{Dispatch, DispatchError};

/// One recorded op outcome in the journal.
#[derive(Debug, Clone, PartialEq)]
pub enum OpRecord {
    /// The op ran (or replayed) to a value.
    Completed(Value),
    /// The op suspended the run (human checkpoint / delay / await), with a
    /// human-readable reason. Resolved later via [`Journal::resolve_suspended`].
    Suspended { reason: String },
}

/// A single journal entry: the op name + its outcome, in execution order.
#[derive(Debug, Clone, PartialEq)]
pub struct JournalEntry {
    pub seq: u64,
    pub op: String,
    pub record: OpRecord,
}

/// The append-only, ordered op log — the durable truth a run replays from.
/// (In production this graduates to the persisted event-log; here it is the
/// in-memory core.)
#[derive(Debug, Clone, Default)]
pub struct Journal {
    entries: Vec<JournalEntry>,
}

impl Journal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// Is the run currently parked on a suspend?
    pub fn is_suspended(&self) -> bool {
        self.entries
            .iter()
            .any(|e| matches!(e.record, OpRecord::Suspended { .. }))
    }

    fn get(&self, idx: usize) -> Option<&JournalEntry> {
        self.entries.get(idx)
    }

    fn append(&mut self, op: &str, record: OpRecord) -> u64 {
        let seq = self.entries.len() as u64;
        self.entries.push(JournalEntry { seq, op: op.to_string(), record });
        seq
    }

    /// Resolve the first parked suspend with the decision value — the external
    /// signal (a human approval, a timer firing) arriving. The next replay will
    /// return `value` at that step and continue past it.
    pub fn resolve_suspended(&mut self, value: Value) -> bool {
        for e in &mut self.entries {
            if matches!(e.record, OpRecord::Suspended { .. }) {
                e.record = OpRecord::Completed(value);
                return true;
            }
        }
        false
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("op `{op}` failed at the door: {reason}")]
    Dispatch { op: String, reason: String },
    /// Not a failure — the run parked on a suspend. The driver catches this.
    #[error("suspended")]
    Suspended,
}

/// The terminal state of one execution attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum RunStatus {
    Completed(Value),
    Suspended,
    Failed(String),
}

/// The context threaded through a program. Each `op`/`suspend` call advances a
/// cursor that maps it to a journal slot; on replay the logged outcome is
/// returned, on the live frontier the op runs through the one door and is logged.
pub struct ExecCtx {
    journal: Journal,
    dispatch: Arc<dyn Dispatch>,
    cursor: usize,
}

impl ExecCtx {
    fn new(journal: Journal, dispatch: Arc<dyn Dispatch>) -> Self {
        Self { journal, dispatch, cursor: 0 }
    }

    /// A journaled op call. Replay → logged result (no re-execution). Live
    /// frontier → execute through `dispatch` (the one door) and log it.
    pub async fn op(&mut self, op: &str, args: Value) -> Result<Value, EngineError> {
        let idx = self.cursor;
        self.cursor += 1;
        if let Some(entry) = self.journal.get(idx) {
            return match &entry.record {
                OpRecord::Completed(v) => Ok(v.clone()),
                OpRecord::Suspended { .. } => Err(EngineError::Suspended),
            };
        }
        let result = self
            .dispatch
            .call(op, args)
            .await
            .map_err(|e| EngineError::Dispatch { op: op.to_string(), reason: e.to_string() })?;
        self.journal.append(op, OpRecord::Completed(result.clone()));
        Ok(result)
    }

    /// A suspending op (human checkpoint / delay / await). Replay → if the entry
    /// has been resolved to a value, return it and continue; else still parked.
    /// Live frontier → journal a `Suspended` entry and park the run.
    pub async fn suspend(&mut self, reason: &str) -> Result<Value, EngineError> {
        let idx = self.cursor;
        self.cursor += 1;
        if let Some(entry) = self.journal.get(idx) {
            return match &entry.record {
                OpRecord::Completed(v) => Ok(v.clone()),
                OpRecord::Suspended { .. } => Err(EngineError::Suspended),
            };
        }
        self.journal
            .append(reason, OpRecord::Suspended { reason: reason.to_string() });
        Err(EngineError::Suspended)
    }

    /// Fan-out: run K ops concurrently as one node, journaling results in INPUT
    /// order (deterministic → replay returns the same vector, re-executing
    /// nothing). All-or-nothing: a failed lane fails the node (it re-runs on
    /// resume, with nothing journaled). For min-successful tolerance, pair the
    /// returned values with [`aggregate`].
    pub async fn parallel(&mut self, calls: Vec<(String, Value)>) -> Result<Vec<Value>, EngineError> {
        let k = calls.len();
        let start = self.cursor;
        self.cursor += k;
        // Replay: the K slots are already journaled → return them in order.
        if k > 0 && self.journal.get(start).is_some() {
            let mut out = Vec::with_capacity(k);
            for i in 0..k {
                match self.journal.get(start + i).map(|e| &e.record) {
                    Some(OpRecord::Completed(v)) => out.push(v.clone()),
                    _ => return Err(EngineError::Suspended),
                }
            }
            return Ok(out);
        }
        // Live frontier: execute all lanes concurrently (the dispatch is shared).
        let dispatch = self.dispatch.clone();
        let futs = calls.into_iter().map(|(op, args)| {
            let dispatch = dispatch.clone();
            async move {
                let res = dispatch.call(&op, args).await;
                (op, res)
            }
        });
        let settled = futures_util::future::join_all(futs).await;
        // Collect all first (fail-fast, NO partial journaling), then journal in
        // input order so replay is deterministic.
        let mut vals = Vec::with_capacity(k);
        for (op, res) in settled {
            match res {
                Ok(v) => vals.push((op, v)),
                Err(e) => return Err(EngineError::Dispatch { op, reason: e.to_string() }),
            }
        }
        let out: Vec<Value> = vals.iter().map(|(_, v)| v.clone()).collect();
        for (op, v) in vals {
            self.journal.append(&op, OpRecord::Completed(v));
        }
        Ok(out)
    }

    /// A journaled op with retry. On the live frontier, retry a TRANSIENT failure
    /// up to `max_attempts`; a terminal failure (a denial, an unknown op) is never
    /// retried (a denial is data). Only the final success is journaled — replay
    /// returns it directly and never replays the retries.
    pub async fn op_with_retry(
        &mut self,
        op: &str,
        args: Value,
        max_attempts: u32,
    ) -> Result<Value, EngineError> {
        let idx = self.cursor;
        self.cursor += 1;
        if let Some(entry) = self.journal.get(idx) {
            return match &entry.record {
                OpRecord::Completed(v) => Ok(v.clone()),
                OpRecord::Suspended { .. } => Err(EngineError::Suspended),
            };
        }
        let mut last = String::new();
        for _ in 0..max_attempts.max(1) {
            match self.dispatch.call(op, args.clone()).await {
                Ok(v) => {
                    self.journal.append(op, OpRecord::Completed(v.clone()));
                    return Ok(v);
                }
                // Transient: retry. Terminal: stop (a denial is data).
                Err(DispatchError::Transient(reason)) => last = reason,
                Err(DispatchError::Terminal(reason)) => {
                    return Err(EngineError::Dispatch { op: op.to_string(), reason });
                }
            }
        }
        Err(EngineError::Dispatch { op: op.to_string(), reason: format!("exhausted retries: {last}") })
    }

    /// Journal a non-dispatch, non-deterministic step — THE model call, a clock
    /// read, a random draw. The determinism boundary made concrete: on the live
    /// frontier it runs `compute` and logs the result; on replay it returns the
    /// logged value WITHOUT running `compute` again. This is why a resumed agent
    /// turn never re-calls the model — the model is just a journaled record.
    pub async fn record<F, Fut>(&mut self, key: &str, compute: F) -> Result<Value, EngineError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Value>,
    {
        let idx = self.cursor;
        self.cursor += 1;
        if let Some(entry) = self.journal.get(idx) {
            return match &entry.record {
                OpRecord::Completed(v) => Ok(v.clone()),
                OpRecord::Suspended { .. } => Err(EngineError::Suspended),
            };
        }
        let v = compute().await;
        self.journal.append(key, OpRecord::Completed(v.clone()));
        Ok(v)
    }
}

/// A program the engine can run and replay. Implemented by a ship-js script or an
/// agent turn in production; by a concrete type in tests. MUST be deterministic
/// given the journal (non-determinism goes through `ctx.op`/`ctx.record`).
#[async_trait::async_trait]
pub trait Program: Send + Sync {
    async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError>;
}

/// Run a program against a journal: execute the live frontier, replay the rest.
/// Returns the terminal status plus the (advanced) journal to persist.
pub async fn execute(
    journal: Journal,
    dispatch: Arc<dyn Dispatch>,
    program: &dyn Program,
) -> (RunStatus, Journal) {
    let mut ctx = ExecCtx::new(journal, dispatch);
    let status = match program.run(&mut ctx).await {
        Ok(v) => RunStatus::Completed(v),
        Err(EngineError::Suspended) => RunStatus::Suspended,
        Err(e) => RunStatus::Failed(e.to_string()),
    };
    (status, ctx.journal)
}

/// Resume a suspended run: resolve the parked suspend with `decision`, then
/// replay. Completed ops return logged results (no re-burn); the resolved
/// suspend now returns `decision` and the program continues.
pub async fn resume(
    mut journal: Journal,
    decision: Value,
    dispatch: Arc<dyn Dispatch>,
    program: &dyn Program,
) -> (RunStatus, Journal) {
    journal.resolve_suspended(decision);
    execute(journal, dispatch, program).await
}

/// The min-successful fan-out verdict (mirrors
/// `ship-pipelines::aggregate_lane_outcomes`): advance when at least `min_ok`
/// lanes succeeded, else fail.
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateDecision {
    Advance { ok: usize },
    Fail { ok: usize, needed: usize },
}

/// Pure fan-out decision over per-lane outcomes.
pub fn aggregate(results: &[Result<Value, String>], min_ok: usize) -> AggregateDecision {
    let ok = results.iter().filter(|r| r.is_ok()).count();
    if ok >= min_ok {
        AggregateDecision::Advance { ok }
    } else {
        AggregateDecision::Fail { ok, needed: min_ok }
    }
}

/// An [`Dispatch`] decorator that turns any externally-driven op stream into a
/// durable one: it journals each op result and replays logged ones without
/// re-executing. Wrap the host dispatch a ship-js `run_module` is handed and the
/// script becomes replayable with **no change to the JS layer** (`op_ship_call`
/// just calls whatever dispatch it was given). This is the bridge between
/// code-mode and durable execution.
///
/// The journal lives behind interior mutability because `Dispatch::call` is
/// `&self`; ops within one program are sequential (the script `await`s each), so
/// the cursor advances deterministically. (Concurrent sub-runs each get their own
/// `JournalingDispatch` — the context-compartmentalization story.)
pub struct JournalingDispatch {
    inner: Arc<dyn Dispatch>,
    state: std::sync::Mutex<JournalCursor>,
}

struct JournalCursor {
    journal: Journal,
    cursor: usize,
}

impl JournalingDispatch {
    pub fn new(inner: Arc<dyn Dispatch>, journal: Journal) -> Self {
        Self { inner, state: std::sync::Mutex::new(JournalCursor { journal, cursor: 0 }) }
    }

    /// Recover the (advanced) journal to persist / replay.
    pub fn into_journal(self) -> Journal {
        self.state.into_inner().unwrap().journal
    }
}

#[async_trait::async_trait]
impl Dispatch for JournalingDispatch {
    async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
        // Reserve this op's journal slot and peek it (lock dropped before await).
        let replayed = {
            let mut st = self.state.lock().unwrap();
            let idx = st.cursor;
            st.cursor += 1;
            st.journal.get(idx).map(|e| e.record.clone())
        };
        if let Some(rec) = replayed {
            return match rec {
                OpRecord::Completed(v) => Ok(v),
                // A replay that reaches an unresolved suspend cannot proceed — and
                // a suspend is not a flake, so it is terminal to this attempt.
                OpRecord::Suspended { reason } => {
                    Err(DispatchError::Terminal(format!("suspended: {reason}")))
                }
            };
        }
        // Live frontier: execute through the inner door, journal the result.
        let r = self.inner.call(op, args).await?;
        self.state.lock().unwrap().journal.append(op, OpRecord::Completed(r.clone()));
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Mutex;

    /// Baseline host: echoes the op as a result value (stands in for a gateway
    /// that allowed and dispatched the op). The engine's job is sequencing +
    /// journaling, not policy — so a trivial echo is the whole host it needs.
    struct Echo;
    #[async_trait]
    impl Dispatch for Echo {
        async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
            Ok(json!({ "op": op, "args": args, "ok": true }))
        }
    }

    /// A host that fails every op TERMINALLY (a denial as the engine sees it).
    struct DenyAll;
    #[async_trait]
    impl Dispatch for DenyAll {
        async fn call(&self, op: &str, _args: Value) -> Result<Value, DispatchError> {
            Err(DispatchError::Terminal(format!("denied: {op}")))
        }
    }

    /// Counts real dispatches + records the op order, to prove replay does not
    /// re-execute.
    struct Counting {
        inner: Arc<dyn Dispatch>,
        calls: Arc<AtomicU32>,
        seen: Arc<Mutex<Vec<String>>>,
    }
    #[async_trait]
    impl Dispatch for Counting {
        async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.seen.lock().unwrap().push(op.to_string());
            self.inner.call(op, args).await
        }
    }

    fn counting() -> (Arc<Counting>, Arc<AtomicU32>, Arc<Mutex<Vec<String>>>) {
        let calls = Arc::new(AtomicU32::new(0));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let c = Arc::new(Counting { inner: Arc::new(Echo), calls: calls.clone(), seen: seen.clone() });
        (c, calls, seen)
    }

    /// Program: read two ops in sequence.
    struct TwoReads;
    #[async_trait]
    impl Program for TwoReads {
        async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
            let a = ctx.op("collections.read", json!({})).await?;
            let b = ctx.op("collections.get", json!({})).await?;
            Ok(json!({ "a": a, "b": b }))
        }
    }

    // The core property: a completed op is NOT re-executed on replay — the logged
    // result comes back, the dispatch is never called again.
    #[tokio::test]
    async fn replay_does_not_re_execute_completed_ops() {
        let (gw, calls, _) = counting();

        // First run: live frontier — both ops execute.
        let (s1, journal) = execute(Journal::new(), gw.clone(), &TwoReads).await;
        assert!(matches!(s1, RunStatus::Completed(_)));
        assert_eq!(calls.load(Ordering::SeqCst), 2, "live run executes both ops");

        // Re-run against the SAME journal: pure replay — zero new executions.
        let (s2, _) = execute(journal, gw.clone(), &TwoReads).await;
        assert_eq!(calls.load(Ordering::SeqCst), 2, "replay re-executes NOTHING");
        assert_eq!(s1, s2, "replay reproduces the same result from the log");
    }

    /// Program: read, then a human checkpoint, then a write — proving the suspend
    /// lands BEFORE the write and the read isn't re-run on resume.
    struct ReadApproveWrite;
    #[async_trait]
    impl Program for ReadApproveWrite {
        async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
            let _read = ctx.op("collections.read", json!({})).await?;
            let decision = ctx.suspend("approve the write?").await?;
            let wrote = ctx.op("collections.put", json!({ "approved": decision })).await?;
            Ok(wrote)
        }
    }

    #[tokio::test]
    async fn suspend_parks_then_resume_continues_without_re_executing() {
        let (gw, calls, seen) = counting();

        // Run to the checkpoint: the read executes, then the run parks.
        let (s1, journal) = execute(Journal::new(), gw.clone(), &ReadApproveWrite).await;
        assert!(matches!(s1, RunStatus::Suspended), "parks at the checkpoint");
        assert_eq!(calls.load(Ordering::SeqCst), 1, "only the read ran; the write was not reached");
        assert!(journal.is_suspended());

        // The human approves → resolve + replay. The read is memoized (NOT
        // re-executed); the write now runs.
        let (s2, journal2) = resume(journal, json!(true), gw.clone(), &ReadApproveWrite).await;
        assert!(matches!(s2, RunStatus::Completed(_)), "completes after approval");
        assert_eq!(calls.load(Ordering::SeqCst), 2, "read NOT re-run; only the write executed on resume");
        assert!(!journal2.is_suspended());
        // Order proves it: read, then put — the read once, the put once.
        assert_eq!(*seen.lock().unwrap(), vec!["collections.read", "collections.put"]);
    }

    // A terminal dispatch failure (a denial, as the engine sees it) surfaces as a
    // failed run — recorded as data, not a panic.
    #[tokio::test]
    async fn terminal_dispatch_failure_fails_the_run_as_data() {
        struct OneDeniedOp;
        #[async_trait]
        impl Program for OneDeniedOp {
            async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
                ctx.op("account.rotate_key", json!({})).await
            }
        }
        let (status, _) = execute(Journal::new(), Arc::new(DenyAll), &OneDeniedOp).await;
        assert!(matches!(status, RunStatus::Failed(_)), "a terminal failure fails the run, surfaced as data");
    }

    // Fan-out: K lanes journal in INPUT order (deterministic, not completion
    // order) and replay re-executes nothing.
    #[tokio::test]
    async fn parallel_journals_in_order_and_replays() {
        let (gw, calls, _) = counting();
        struct ThreeReads;
        #[async_trait]
        impl Program for ThreeReads {
            async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
                let vs = ctx
                    .parallel(vec![
                        ("collections.read".into(), json!({})),
                        ("collections.get".into(), json!({})),
                        ("collections.items".into(), json!({})),
                    ])
                    .await?;
                Ok(json!(vs))
            }
        }
        let (s1, journal) = execute(Journal::new(), gw.clone(), &ThreeReads).await;
        assert!(matches!(s1, RunStatus::Completed(_)));
        assert_eq!(calls.load(Ordering::SeqCst), 3, "three lanes execute live");
        let ops: Vec<&str> = journal.entries().iter().map(|e| e.op.as_str()).collect();
        assert_eq!(ops, vec!["collections.read", "collections.get", "collections.items"], "journaled in input order");

        let (s2, _) = execute(journal, gw, &ThreeReads).await;
        assert_eq!(calls.load(Ordering::SeqCst), 3, "replay re-executes nothing");
        assert_eq!(s1, s2);
    }

    #[test]
    fn aggregate_min_successful() {
        let ok = || Ok::<Value, String>(json!(1));
        let err = || Err::<Value, String>("boom".into());
        assert_eq!(aggregate(&[ok(), ok(), err()], 2), AggregateDecision::Advance { ok: 2 });
        assert_eq!(aggregate(&[ok(), err(), err()], 2), AggregateDecision::Fail { ok: 1, needed: 2 });
    }

    // Retry recovers a transient blip and journals ONLY the final result; replay
    // never replays the retries.
    #[tokio::test]
    async fn retry_recovers_transient_then_replays_clean() {
        struct Flaky {
            inner: Arc<dyn Dispatch>,
            fail_until: u32,
            calls: Arc<AtomicU32>,
        }
        #[async_trait]
        impl Dispatch for Flaky {
            async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                if n < self.fail_until {
                    return Err(DispatchError::Transient("flaky".into()));
                }
                self.inner.call(op, args).await
            }
        }
        let calls = Arc::new(AtomicU32::new(0));
        let gw = Arc::new(Flaky { inner: Arc::new(Echo), fail_until: 2, calls: calls.clone() });
        struct RetryRead;
        #[async_trait]
        impl Program for RetryRead {
            async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
                ctx.op_with_retry("collections.read", json!({}), 5).await
            }
        }
        let (s1, journal) = execute(Journal::new(), gw.clone(), &RetryRead).await;
        assert!(matches!(s1, RunStatus::Completed(_)), "recovers after 2 transient failures");
        assert_eq!(calls.load(Ordering::SeqCst), 3, "2 failures + 1 success");
        assert_eq!(journal.entries().len(), 1, "only the final result journaled");

        let (_s2, _) = execute(journal, gw, &RetryRead).await;
        assert_eq!(calls.load(Ordering::SeqCst), 3, "replay re-executes nothing — retries are not replayed");
    }

    // A terminal failure is NOT retried — a denial is data, re-attempting would
    // mask a real signal.
    #[tokio::test]
    async fn retry_does_not_re_attempt_a_terminal_failure() {
        let calls = Arc::new(AtomicU32::new(0));
        struct CountingDeny(Arc<AtomicU32>);
        #[async_trait]
        impl Dispatch for CountingDeny {
            async fn call(&self, op: &str, _args: Value) -> Result<Value, DispatchError> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Err(DispatchError::Terminal(format!("denied: {op}")))
            }
        }
        struct RetryDenied;
        #[async_trait]
        impl Program for RetryDenied {
            async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
                ctx.op_with_retry("account.rotate_key", json!({}), 5).await
            }
        }
        let gw = Arc::new(CountingDeny(calls.clone()));
        let (status, _) = execute(Journal::new(), gw, &RetryDenied).await;
        assert!(matches!(status, RunStatus::Failed(_)));
        assert_eq!(calls.load(Ordering::SeqCst), 1, "a terminal failure is attempted exactly once — never retried");
    }

    // The thesis connector at engine altitude: `JournalingDispatch` makes an
    // EXTERNALLY-driven op stream (a ship-js script's `op_ship_call`s) durable.
    // Replaying the same ops against the seeded journal calls the inner dispatch
    // ZERO times — code-mode IS a durable program, with nothing JS-specific here.
    #[tokio::test]
    async fn journaling_dispatch_replays_without_re_calling_inner() {
        let (inner, calls, _) = counting();

        // Live: two ops execute through the inner dispatch (the "host gateway").
        let jg = JournalingDispatch::new(inner.clone(), Journal::new());
        let a1 = jg.call("collections.read", json!({})).await.unwrap();
        let b1 = jg.call("collections.get", json!({})).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2, "live run executes both ops");
        let journal = jg.into_journal();
        assert_eq!(journal.entries().len(), 2, "two ops journaled");

        // Replay: same ops, seeded journal → memoized, inner NOT called.
        let jg2 = JournalingDispatch::new(inner, journal);
        let a2 = jg2.call("collections.read", json!({})).await.unwrap();
        let b2 = jg2.call("collections.get", json!({})).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2, "replay re-calls the inner ZERO times");
        assert_eq!((a1, b1), (a2, b2), "replay reproduces the same results");
    }

    // The agent loop is a durable program: the model call is a journaled `record`,
    // so a turn that suspends mid-reasoning RESUMES WITHOUT re-calling the model.
    // The no-token-re-burn guarantee, proven.
    #[tokio::test]
    async fn agent_turn_is_durable_model_not_recalled_on_resume() {
        let model_calls = Arc::new(AtomicU32::new(0));
        struct Turn {
            model_calls: Arc<AtomicU32>,
        }
        #[async_trait]
        impl Program for Turn {
            async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
                // The model decides the next action — a journaled record (NOT a
                // dispatch op; the model is reasoning, not an effect).
                let mc = self.model_calls.clone();
                let decision = ctx
                    .record("model.turn", move || async move {
                        mc.fetch_add(1, Ordering::SeqCst);
                        json!({ "op": "collections.read" })
                    })
                    .await?;
                let op = decision["op"].as_str().unwrap_or("collections.read").to_string();
                ctx.op(&op, json!({})).await?; // act on the decision
                ctx.suspend("approve the write?").await?; // human checkpoint mid-turn
                ctx.op("collections.put", json!({})).await?;
                Ok(json!("turn done"))
            }
        }
        let prog = Turn { model_calls: model_calls.clone() };

        let (s1, journal) = execute(Journal::new(), Arc::new(Echo), &prog).await;
        assert!(matches!(s1, RunStatus::Suspended), "parks at the checkpoint");
        assert_eq!(model_calls.load(Ordering::SeqCst), 1, "model called once on the live turn");

        let (s2, _) = resume(journal, json!(true), Arc::new(Echo), &prog).await;
        assert!(matches!(s2, RunStatus::Completed(_)), "completes after approval");
        assert_eq!(model_calls.load(Ordering::SeqCst), 1, "resume does NOT re-call the model — no token re-burn");
    }
}
