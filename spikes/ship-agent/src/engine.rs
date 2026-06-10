//! The durable-execution engine (S16.b prototype).
//!
//! A *program* is a deterministic sequence of journaled op-calls (in production:
//! a ship-js script; here: a `Program` impl). Every op result is written to the
//! **journal**. On resume the program **re-runs from the top**, and ops that
//! already completed return their *logged* results instead of re-executing — so
//! a recovered run never re-burns work (an LLM call is just a journaled op; its
//! completion replays from the log). A *suspending* op (human checkpoint, delay,
//! sub-run await) parks the run; when the external decision arrives the entry is
//! resolved and the program replays past the suspend point.
//!
//! Invariants this enforces:
//! - **Replay never re-executes a completed op** (no re-burn).
//! - **Determinism**: the Nth op-call maps to the Nth journal entry, so the
//!   program MUST be deterministic (all non-determinism — model, time, random,
//!   network — is an op, hence journaled). This is the spike's existing
//!   `Date.now`/`Math.random` ban, generalized.

use std::future::Future;
use std::sync::Arc;

use serde_json::Value;

use crate::auth::AuthContext;
use crate::gateway::{Gateway, GatewayCallKind, GatewayError};

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
/// in-memory prototype, sibling to `events::EventLog`.)
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
    Gateway { op: String, reason: String },
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
    gateway: Arc<dyn Gateway>,
    auth: AuthContext,
    cursor: usize,
}

impl ExecCtx {
    fn new(journal: Journal, gateway: Arc<dyn Gateway>, auth: AuthContext) -> Self {
        Self { journal, gateway, auth, cursor: 0 }
    }

    /// A journaled op call. Replay → logged result (no re-execution). Live
    /// frontier → execute through `gateway.call` (the one door) and log it.
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
            .gateway
            .call(op, args, &self.auth, GatewayCallKind::First)
            .await
            .map_err(|e| EngineError::Gateway { op: op.to_string(), reason: e.to_string() })?;
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
        // Live frontier: execute all lanes concurrently (the gateway is shared).
        let gw = self.gateway.clone();
        let auth = self.auth.clone();
        let futs = calls.into_iter().map(|(op, args)| {
            let gw = gw.clone();
            let auth = auth.clone();
            async move {
                let res = gw.call(&op, args, &auth, GatewayCallKind::First).await;
                (op, res)
            }
        });
        let settled = futures::future::join_all(futs).await;
        // Collect all first (fail-fast, NO partial journaling), then journal in
        // input order so replay is deterministic.
        let mut vals = Vec::with_capacity(k);
        for (op, res) in settled {
            match res {
                Ok(v) => vals.push((op, v)),
                Err(e) => return Err(EngineError::Gateway { op, reason: e.to_string() }),
            }
        }
        let out: Vec<Value> = vals.iter().map(|(_, v)| v.clone()).collect();
        for (op, v) in vals {
            self.journal.append(&op, OpRecord::Completed(v));
        }
        Ok(out)
    }

    /// A journaled op with retry. On the live frontier, retry a TRANSIENT failure
    /// (a `Dispatch` blip) up to `max_attempts`; a terminal `Denied`/`UnknownOp`
    /// is never retried (a denial is data). Only the final success is journaled —
    /// replay returns it directly and never replays the retries.
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
            match self.gateway.call(op, args.clone(), &self.auth, GatewayCallKind::First).await {
                Ok(v) => {
                    self.journal.append(op, OpRecord::Completed(v.clone()));
                    return Ok(v);
                }
                // Transient: retry. Terminal: stop (a denial is data).
                Err(GatewayError::Dispatch(reason)) => last = reason,
                Err(e) => return Err(EngineError::Gateway { op: op.to_string(), reason: e.to_string() }),
            }
        }
        Err(EngineError::Gateway { op: op.to_string(), reason: format!("exhausted retries: {last}") })
    }

    /// Journal a non-gateway, non-deterministic step — THE model call, a clock
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

/// A program the engine can run and replay. Implemented by a ship-js script in
/// production; by a concrete type in tests. MUST be deterministic given the
/// journal (non-determinism goes through `ctx.op`).
#[async_trait::async_trait]
pub trait Program: Send + Sync {
    async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError>;
}

/// Run a program against a journal: execute the live frontier, replay the rest.
/// Returns the terminal status plus the (advanced) journal to persist.
pub async fn execute(
    journal: Journal,
    gateway: Arc<dyn Gateway>,
    auth: AuthContext,
    program: &dyn Program,
) -> (RunStatus, Journal) {
    let mut ctx = ExecCtx::new(journal, gateway, auth);
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
    gateway: Arc<dyn Gateway>,
    auth: AuthContext,
    program: &dyn Program,
) -> (RunStatus, Journal) {
    journal.resolve_suspended(decision);
    execute(journal, gateway, auth, program).await
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

/// A `Gateway` decorator that turns any script into a durable program: it
/// journals each op result and replays logged ones without re-executing — so a
/// ship-js `run_module` script becomes replayable with **no change to the deno
/// layer** (`op_ship_call` just calls whatever gateway it was handed). This is
/// the bridge between code-mode and durable execution.
///
/// The journal lives behind interior mutability because `Gateway::call` is
/// `&self`; ops within one program are sequential (the script `await`s each), so
/// the cursor advances deterministically. (Concurrent sub-runs each get their
/// own `JournalingGateway` — the context-compartmentalization story.)
pub struct JournalingGateway {
    inner: Arc<dyn Gateway>,
    state: std::sync::Mutex<JournalCursor>,
}

struct JournalCursor {
    journal: Journal,
    cursor: usize,
}

impl JournalingGateway {
    pub fn new(inner: Arc<dyn Gateway>, journal: Journal) -> Self {
        Self { inner, state: std::sync::Mutex::new(JournalCursor { journal, cursor: 0 }) }
    }

    /// Recover the (advanced) journal to persist / replay.
    pub fn into_journal(self) -> Journal {
        self.state.into_inner().unwrap().journal
    }
}

#[async_trait::async_trait]
impl Gateway for JournalingGateway {
    async fn call(
        &self,
        op: &str,
        args: Value,
        auth: &AuthContext,
        kind: GatewayCallKind,
    ) -> Result<Value, GatewayError> {
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
                OpRecord::Suspended { reason } => {
                    Err(GatewayError::Dispatch(format!("suspended: {reason}")))
                }
            };
        }
        // Live frontier: execute through the inner door, journal the result.
        let r = self.inner.call(op, args, auth, kind).await?;
        self.state.lock().unwrap().journal.append(op, OpRecord::Completed(r.clone()));
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::{GatewayError, InProcessGateway};
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Mutex;

    fn agent(grants: &[&str]) -> AuthContext {
        AuthContext::in_process("t", grants.iter().map(|s| s.to_string()).collect())
    }

    /// Gateway wrapper that counts real op executions, to prove replay does not
    /// re-execute.
    struct CountingGateway {
        inner: InProcessGateway,
        calls: Arc<AtomicU32>,
        seen: Arc<Mutex<Vec<String>>>,
    }
    #[async_trait]
    impl Gateway for CountingGateway {
        async fn call(
            &self,
            op: &str,
            args: Value,
            a: &AuthContext,
            kind: GatewayCallKind,
        ) -> Result<Value, GatewayError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.seen.lock().unwrap().push(op.to_string());
            self.inner.call(op, args, a, kind).await
        }
    }

    /// Program: read two collections ops in sequence.
    struct TwoReads;
    #[async_trait]
    impl Program for TwoReads {
        async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
            let a = ctx.op("collections.read", json!({})).await?;
            let b = ctx.op("collections.get", json!({})).await?;
            Ok(json!({ "a": a, "b": b }))
        }
    }

    // The core property: a completed op is NOT re-executed on replay — the
    // logged result comes back, the gateway is never called again.
    #[tokio::test]
    async fn replay_does_not_re_execute_completed_ops() {
        let calls = Arc::new(AtomicU32::new(0));
        let gw = Arc::new(CountingGateway {
            inner: InProcessGateway::default(),
            calls: calls.clone(),
            seen: Arc::new(Mutex::new(Vec::new())),
        });

        // First run: live frontier — both ops execute.
        let (s1, journal) = execute(Journal::new(), gw.clone(), agent(&["collections"]), &TwoReads).await;
        assert!(matches!(s1, RunStatus::Completed(_)));
        assert_eq!(calls.load(Ordering::SeqCst), 2, "live run executes both ops");

        // Re-run against the SAME journal: pure replay — zero new executions.
        let (s2, _) = execute(journal, gw.clone(), agent(&["collections"]), &TwoReads).await;
        assert_eq!(calls.load(Ordering::SeqCst), 2, "replay re-executes NOTHING");
        assert_eq!(s1, s2, "replay reproduces the same result from the log");
    }

    /// Program: read, then a human checkpoint, then a write — proving the
    /// suspend lands BEFORE the write and the read isn't re-run on resume.
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
        let calls = Arc::new(AtomicU32::new(0));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let gw = Arc::new(CountingGateway {
            inner: InProcessGateway::default(),
            calls: calls.clone(),
            seen: seen.clone(),
        });
        let auth = agent(&["collections"]);

        // Run to the checkpoint: the read executes, then the run parks.
        let (s1, journal) = execute(Journal::new(), gw.clone(), auth.clone(), &ReadApproveWrite).await;
        assert!(matches!(s1, RunStatus::Suspended), "parks at the checkpoint");
        assert_eq!(calls.load(Ordering::SeqCst), 1, "only the read ran; the write was not reached");
        assert!(journal.is_suspended());

        // The human approves → resolve + replay. The read is memoized (NOT
        // re-executed); the write now runs.
        let (s2, journal2) = resume(journal, json!(true), gw.clone(), auth, &ReadApproveWrite).await;
        assert!(matches!(s2, RunStatus::Completed(_)), "completes after approval");
        assert_eq!(calls.load(Ordering::SeqCst), 2, "read NOT re-run; only the write executed on resume");
        assert!(!journal2.is_suspended());
        // Order proves it: read, then put — the read once, the put once.
        assert_eq!(*seen.lock().unwrap(), vec!["collections.read", "collections.put"]);
    }

    // A denial inside a program surfaces as a failed run (denial is data,
    // recorded), not a panic.
    #[tokio::test]
    async fn ungranted_op_fails_the_run_as_data() {
        struct OperatorOnlyDenied;
        #[async_trait]
        impl Program for OperatorOnlyDenied {
            async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
                // account.rotate_key is C3 (operator-only) → denied for any
                // in-process agent (operator_cred=false by construction).
                ctx.op("account.rotate_key", json!({})).await
            }
        }
        let gw = Arc::new(InProcessGateway::default());
        let (status, _) = execute(Journal::new(), gw, agent(&[]), &OperatorOnlyDenied).await;
        assert!(matches!(status, RunStatus::Failed(_)), "a gate denial fails the run, surfaced as data");
    }

    // Fan-out: K lanes journal in INPUT order (deterministic, not completion
    // order) and replay re-executes nothing.
    #[tokio::test]
    async fn parallel_journals_in_order_and_replays() {
        let calls = Arc::new(AtomicU32::new(0));
        let gw = Arc::new(CountingGateway {
            inner: InProcessGateway::default(),
            calls: calls.clone(),
            seen: Arc::new(Mutex::new(Vec::new())),
        });
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
        let (s1, journal) = execute(Journal::new(), gw.clone(), agent(&["collections"]), &ThreeReads).await;
        assert!(matches!(s1, RunStatus::Completed(_)));
        assert_eq!(calls.load(Ordering::SeqCst), 3, "three lanes execute live");
        let ops: Vec<&str> = journal.entries().iter().map(|e| e.op.as_str()).collect();
        assert_eq!(ops, vec!["collections.read", "collections.get", "collections.items"], "journaled in input order");

        let (s2, _) = execute(journal, gw, agent(&["collections"]), &ThreeReads).await;
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
        struct FlakyGateway {
            inner: InProcessGateway,
            fail_until: u32,
            calls: Arc<AtomicU32>,
        }
        #[async_trait]
        impl Gateway for FlakyGateway {
            async fn call(
                &self,
                op: &str,
                args: Value,
                a: &AuthContext,
                kind: GatewayCallKind,
            ) -> Result<Value, GatewayError> {
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                if n < self.fail_until {
                    return Err(GatewayError::Dispatch("flaky".into()));
                }
                self.inner.call(op, args, a, kind).await
            }
        }
        let calls = Arc::new(AtomicU32::new(0));
        let gw = Arc::new(FlakyGateway {
            inner: InProcessGateway::default(),
            fail_until: 2,
            calls: calls.clone(),
        });
        struct RetryRead;
        #[async_trait]
        impl Program for RetryRead {
            async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
                ctx.op_with_retry("collections.read", json!({}), 5).await
            }
        }
        let (s1, journal) = execute(Journal::new(), gw.clone(), agent(&["collections"]), &RetryRead).await;
        assert!(matches!(s1, RunStatus::Completed(_)), "recovers after 2 transient failures");
        assert_eq!(calls.load(Ordering::SeqCst), 3, "2 failures + 1 success");
        assert_eq!(journal.entries().len(), 1, "only the final result journaled");

        let (_s2, _) = execute(journal, gw, agent(&["collections"]), &RetryRead).await;
        assert_eq!(calls.load(Ordering::SeqCst), 3, "replay re-executes nothing — retries are not replayed");
    }

    // A4 — the thesis connector: a ship-js script IS a durable program. Its ops
    // route through a JournalingGateway; replaying the same script against the
    // journal re-executes NOTHING (the inner gateway is never called again).
    #[tokio::test(flavor = "current_thread")]
    async fn ship_js_script_is_a_durable_program() {
        let calls = Arc::new(AtomicU32::new(0));
        let inner: Arc<dyn Gateway> = Arc::new(CountingGateway {
            inner: InProcessGateway::default(),
            calls: calls.clone(),
            seen: Arc::new(Mutex::new(Vec::new())),
        });
        let src = r#"
            import { collections } from "ship";
            const a = await collections.read({});
            const b = await collections.get({});
            Deno.core.ops.op_emit({ a, b });
        "#
        .to_string();

        // Live run: the script's two ops execute through the inner gateway.
        let jg = Arc::new(JournalingGateway::new(inner.clone(), Journal::new()));
        let out1 = crate::runtime::run_module(
            jg.clone(),
            agent(&["collections"]),
            vec!["collections".into()],
            src.clone(),
        )
        .await
        .expect("script runs");
        assert_eq!(calls.load(Ordering::SeqCst), 2, "live run executes both ops");
        let journal = Arc::try_unwrap(jg).ok().expect("sole owner").into_journal();
        assert_eq!(journal.entries().len(), 2, "two ops journaled");

        // Replay run: same script, seeded journal → ops memoized, inner NOT called.
        let jg2 = Arc::new(JournalingGateway::new(inner, journal));
        let out2 = crate::runtime::run_module(jg2, agent(&["collections"]), vec!["collections".into()], src)
            .await
            .expect("script replays");
        assert_eq!(calls.load(Ordering::SeqCst), 2, "replay re-executes NOTHING — code-mode is a durable program");
        assert_eq!(out1, out2, "replay reproduces the same emitted output");
    }

    // A6 — the agent loop is a durable program: the model call is a journaled
    // `record`, so a turn that suspends mid-reasoning RESUMES WITHOUT re-calling
    // the model. The no-token-re-burn guarantee, proven.
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
                // gateway op; the model is reasoning, not an effect).
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
        let gw = Arc::new(InProcessGateway::default());
        let prog = Turn { model_calls: model_calls.clone() };

        let (s1, journal) = execute(Journal::new(), gw.clone(), agent(&["collections"]), &prog).await;
        assert!(matches!(s1, RunStatus::Suspended), "parks at the checkpoint");
        assert_eq!(model_calls.load(Ordering::SeqCst), 1, "model called once on the live turn");

        let (s2, _) = resume(journal, json!(true), gw, agent(&["collections"]), &prog).await;
        assert!(matches!(s2, RunStatus::Completed(_)), "completes after approval");
        assert_eq!(model_calls.load(Ordering::SeqCst), 1, "resume does NOT re-call the model — no token re-burn");
    }
}
