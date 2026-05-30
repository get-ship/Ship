//! Integration tests for the competing-approaches fanout primitive.
//!
//! Spec §7 requires three scenarios covered end-to-end:
//!
//! 1. All lanes succeed — the judge's `resolve_consumes` returns every lane's
//!    output.
//! 2. One lane fails — the judge sees the surviving outputs plus a resolvable
//!    `<lane_id>.failed` marker carrying the failure reason.
//! 3. All lanes fail — the aggregator produces `AggregateDecision::Fail`, so
//!    the fanout step fails and the judge is never dispatched.
//!
//! Shared harness (mock ports, event/artifact seeding) lives in
//! `tests_fanout_harness.rs`.

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::artifacts::ArtifactScope;
use ship_substrate::events::pipeline::{
    FanoutJudge, FanoutSpec, OutputScope, Pipeline, PipelineStep, StepOutput, StepVerdict,
};
use crate::services::orchestrator::fanout_aggregate::{
    AggregateDecision, FAILED_MARKER_NAME, aggregate_lane_outcomes, complete_fanout_lane,
};
use crate::services::orchestrator::step_io::{parse_lane_scoped_ref, resolve_consumes};
use crate::services::orchestrator::tests_fanout_harness::{
    fresh_ship_dir, make_octx, mk_lane, mk_lane_ref, mk_observed, mk_pipeline_with_three_lanes,
    mk_step, seed_pipeline_run_started, seed_lane_success,
};
use crate::services::orchestrator::LaneOutcome;

// ── Test 1: all lanes succeed ─────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fanout_all_lanes_succeed_judge_sees_every_lane_output() {
    let ship_dir = fresh_ship_dir();
    let pipeline_run_id = "job-fanout-all-ok";
    let run_scope = ArtifactScope::Run {
        space_id: "test-proj".into(),
        run_id: pipeline_run_id.into(),
    };

    let kernel = Arc::new(Mutex::new(
        ship_substrate::events::kernel_router::KernelRouter::new(ship_dir.clone()).expect("kernel"),
    ));

    let pipeline = mk_pipeline_with_three_lanes();
    seed_pipeline_run_started(&ship_dir, pipeline_run_id, &pipeline);

    let a_id = seed_lane_success(&ship_dir, &kernel, pipeline_run_id, &run_scope, "lane-a", b"A").await;
    let b_id = seed_lane_success(&ship_dir, &kernel, pipeline_run_id, &run_scope, "lane-b", b"BB").await;
    let c_id = seed_lane_success(&ship_dir, &kernel, pipeline_run_id, &run_scope, "lane-c", b"CCC").await;

    // Drive lane completion exactly the way the router does.
    let octx = make_octx();
    let mut outcomes: Vec<LaneOutcome> = Vec::new();
    for lane_id in ["lane-a", "lane-b", "lane-c"] {
        let lane_slug = format!("test-slug-draft-{lane_id}");
        let (tx, rx) = tokio::sync::oneshot::channel::<LaneOutcome>();
        octx.fanout_lane_txs.lock().await.insert(lane_slug.clone(), tx);

        let mut payload =
            mk_observed(&lane_slug, &format!("draft.{lane_id}"), StepVerdict::Advance, None);
        payload.pipeline_run_id = pipeline_run_id.into();
        let lane_ref = mk_lane_ref(pipeline_run_id, "test-slug", lane_id);

        complete_fanout_lane(&kernel, &payload, &lane_ref, &octx).await;
        outcomes.push(rx.await.expect("lane outcome"));
    }

    // Aggregator says Advance with 3/3 successful lanes.
    match aggregate_lane_outcomes(&outcomes, 1, 3) {
        AggregateDecision::Advance { ok_count } => assert_eq!(ok_count, 3),
        other => panic!("expected Advance, got {other:?}"),
    }

    // Judge resolves its three lane-scoped consumes.
    let wt = tempfile::tempdir().unwrap();
    let judge = pipeline.steps.iter().find(|s| s.id == "choose").unwrap();
    let resolved = resolve_consumes(judge, &pipeline, pipeline_run_id, wt.path())
        .expect("judge must resolve all three lane outputs");

    assert_eq!(resolved.len(), 3, "judge sees three inputs");
    let ids: std::collections::HashSet<_> =
        resolved.iter().map(|r| r.artifact_id.clone()).collect();
    assert!(ids.contains(&a_id));
    assert!(ids.contains(&b_id));
    assert!(ids.contains(&c_id));
    for r in &resolved {
        assert!(
            r.failure_reason.is_none(),
            "successful lane must not carry a failure_reason: {:?}",
            r.name
        );
        let (_, output) = parse_lane_scoped_ref(&r.name).unwrap();
        assert_eq!(output, "draft");
    }
}

// ── Test 2: one lane fails ────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fanout_one_lane_fails_judge_sees_survivors_plus_failed_marker() {
    let ship_dir = fresh_ship_dir();
    let pipeline_run_id = "job-fanout-one-fails";
    let run_scope = ArtifactScope::Run {
        space_id: "test-proj".into(),
        run_id: pipeline_run_id.into(),
    };

    let kernel = Arc::new(Mutex::new(
        ship_substrate::events::kernel_router::KernelRouter::new(ship_dir.clone()).expect("kernel"),
    ));

    // Pipeline: judge consumes two drafts + one .failed marker.
    let mut fanout_step = mk_step("draft", "implementer");
    fanout_step.fanout = Some(FanoutSpec {
        lanes: vec![mk_lane("lane-a"), mk_lane("lane-b"), mk_lane("lane-c")],
        judge: FanoutJudge { step: "choose".into() },
        min_successful_lanes: 1,
    });
    fanout_step.produces = vec![StepOutput {
        name: "draft".into(),
        scope: OutputScope::Run,
        required: false,
    }];
    let mut judge: PipelineStep = mk_step("choose", "judge");
    judge.after = vec!["draft".into()];
    judge.consumes = vec![
        "lane-a.draft".into(),
        "lane-b.draft".into(),
        "lane-c.failed".into(),
    ];
    let pipeline = Pipeline {
        steps: vec![fanout_step, judge],
        auto_merge: false,
    };
    seed_pipeline_run_started(&ship_dir, pipeline_run_id, &pipeline);

    let _a_id = seed_lane_success(&ship_dir, &kernel, pipeline_run_id, &run_scope, "lane-a", b"A").await;
    let _b_id = seed_lane_success(&ship_dir, &kernel, pipeline_run_id, &run_scope, "lane-b", b"BB").await;

    // Drive lane completions: a+b succeed, c fails — the failure path writes
    // the `lane-c.failed` marker artifact + its synthetic step.done_declared.
    let octx = make_octx();
    let mut outcomes: Vec<LaneOutcome> = Vec::new();

    for lane_id in ["lane-a", "lane-b"] {
        let lane_slug = format!("test-slug-draft-{lane_id}");
        let (tx, rx) = tokio::sync::oneshot::channel::<LaneOutcome>();
        octx.fanout_lane_txs.lock().await.insert(lane_slug.clone(), tx);
        let mut payload =
            mk_observed(&lane_slug, &format!("draft.{lane_id}"), StepVerdict::Advance, None);
        payload.pipeline_run_id = pipeline_run_id.into();
        let lane_ref = mk_lane_ref(pipeline_run_id, "test-slug", lane_id);
        complete_fanout_lane(&kernel, &payload, &lane_ref, &octx).await;
        outcomes.push(rx.await.expect("lane outcome"));
    }

    let failed_reason = "agent crashed after 30s";
    {
        let lane_slug = "test-slug-draft-lane-c".to_string();
        let (tx, rx) = tokio::sync::oneshot::channel::<LaneOutcome>();
        octx.fanout_lane_txs.lock().await.insert(lane_slug.clone(), tx);
        let mut payload =
            mk_observed(&lane_slug, "draft.lane-c", StepVerdict::Failed, Some(failed_reason));
        payload.pipeline_run_id = pipeline_run_id.into();
        let lane_ref = mk_lane_ref(pipeline_run_id, "test-slug", "lane-c");
        complete_fanout_lane(&kernel, &payload, &lane_ref, &octx).await;
        let outcome = rx.await.expect("lane outcome");
        assert!(!outcome.success, "lane-c outcome must be marked failed");
        assert!(
            outcome.outputs.contains_key(FAILED_MARKER_NAME),
            "lane-c outcome must carry the `failed` marker id in its outputs map"
        );
        outcomes.push(outcome);
    }

    // 2 of 3 lanes succeeded, min_ok = 1 → Advance with ok_count = 2.
    match aggregate_lane_outcomes(&outcomes, 1, 3) {
        AggregateDecision::Advance { ok_count } => assert_eq!(ok_count, 2),
        other => panic!("expected Advance, got {other:?}"),
    }

    // Judge resolves its three consumes: lane-a.draft, lane-b.draft, lane-c.failed.
    // The failure path slots into the same resolve_consumes mechanism.
    let wt = tempfile::tempdir().unwrap();
    let judge = pipeline.steps.iter().find(|s| s.id == "choose").unwrap();
    let resolved = resolve_consumes(judge, &pipeline, pipeline_run_id, wt.path())
        .expect("judge must resolve survivors + failed marker");

    assert_eq!(resolved.len(), 3);

    let failed = resolved
        .iter()
        .find(|r| r.name == "lane-c.failed")
        .expect("judge resolves lane-c.failed marker");
    let reason = failed
        .failure_reason
        .as_deref()
        .expect("failed marker must carry failure_reason for the prompt composer");
    assert!(
        reason.contains("agent crashed"),
        "failure_reason must be extracted from the marker body: {reason}"
    );

    let a = resolved.iter().find(|r| r.name == "lane-a.draft").unwrap();
    let b = resolved.iter().find(|r| r.name == "lane-b.draft").unwrap();
    assert!(a.failure_reason.is_none());
    assert!(b.failure_reason.is_none());
}

// ── Test 3: all lanes fail ────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fanout_all_lanes_fail_aggregator_fails_fanout_judge_skipped() {
    let ship_dir = fresh_ship_dir();
    let pipeline_run_id = "job-fanout-all-fail";

    let kernel = Arc::new(Mutex::new(
        ship_substrate::events::kernel_router::KernelRouter::new(ship_dir.clone()).expect("kernel"),
    ));

    let pipeline = mk_pipeline_with_three_lanes();
    seed_pipeline_run_started(&ship_dir, pipeline_run_id, &pipeline);

    // Drive all three lanes through the failure path.
    let octx = make_octx();
    let mut outcomes: Vec<LaneOutcome> = Vec::new();
    for lane_id in ["lane-a", "lane-b", "lane-c"] {
        let lane_slug = format!("test-slug-draft-{lane_id}");
        let (tx, rx) = tokio::sync::oneshot::channel::<LaneOutcome>();
        octx.fanout_lane_txs.lock().await.insert(lane_slug.clone(), tx);
        let mut payload =
            mk_observed(&lane_slug, &format!("draft.{lane_id}"), StepVerdict::Failed, Some("boom"));
        payload.pipeline_run_id = pipeline_run_id.into();
        let lane_ref = mk_lane_ref(pipeline_run_id, "test-slug", lane_id);
        complete_fanout_lane(&kernel, &payload, &lane_ref, &octx).await;
        let outcome = rx.await.expect("lane outcome");
        assert!(!outcome.success);
        outcomes.push(outcome);
    }

    // Aggregator with min_ok = 1, total = 3, zero successes → Fail.
    match aggregate_lane_outcomes(&outcomes, 1, 3) {
        AggregateDecision::Fail { ok_count, reason } => {
            assert_eq!(ok_count, 0, "no lanes succeeded");
            assert!(
                reason.contains("0/1") && reason.contains("3"),
                "failure reason must cite ok/min and total: {reason}"
            );
        }
        other => panic!("expected Fail, got {other:?}"),
    }

    // Judge is skipped: its `consumes` would resolve against `draft` outputs
    // from successful lanes, but all three lanes failed. Resolve against the
    // default pipeline (consumes `.draft`) must return Err — the judge would
    // never dispatch in production when the aggregator emits step_failed.
    let wt = tempfile::tempdir().unwrap();
    let judge = pipeline.steps.iter().find(|s| s.id == "choose").unwrap();
    let err = match resolve_consumes(judge, &pipeline, pipeline_run_id, wt.path()) {
        Err(e) => e,
        Ok(_) => panic!("judge must not resolve when all lanes failed and it consumes .draft"),
    };
    assert!(
        err.contains("not found") || err.contains("lane-"),
        "resolver must explain the missing lane output: {err}"
    );
}

// ── Pure aggregate_lane_outcomes coverage ─────────────────────────────────────

#[test]
fn aggregate_advances_when_ok_count_meets_min() {
    let outcomes = vec![
        LaneOutcome { lane_id: "a".into(), success: true, outputs: Default::default() },
        LaneOutcome { lane_id: "b".into(), success: false, outputs: Default::default() },
        LaneOutcome { lane_id: "c".into(), success: true, outputs: Default::default() },
    ];
    assert_eq!(
        aggregate_lane_outcomes(&outcomes, 2, 3),
        AggregateDecision::Advance { ok_count: 2 }
    );
}

#[test]
fn aggregate_fails_when_ok_count_below_min() {
    let outcomes = vec![
        LaneOutcome { lane_id: "a".into(), success: false, outputs: Default::default() },
        LaneOutcome { lane_id: "b".into(), success: true, outputs: Default::default() },
    ];
    let decision = aggregate_lane_outcomes(&outcomes, 2, 2);
    match decision {
        AggregateDecision::Fail { ok_count, reason } => {
            assert_eq!(ok_count, 1);
            assert!(reason.contains("1/2"));
        }
        _ => panic!("expected Fail"),
    }
}
