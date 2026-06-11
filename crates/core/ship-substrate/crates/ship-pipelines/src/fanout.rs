//! Fanout aggregation — pure decision logic.
//!
//! Once every lane in a fanout step has reported a `LaneOutcome`, the
//! aggregator decides whether the fanout advances or fails based on
//! `min_successful_lanes`. The decision is pure: same outcomes + same
//! threshold → same answer, in the daemon, the cloud worker, or the
//! browser embed.

use std::collections::BTreeMap;

/// Outcome from a single fanout lane after its step monitor fires.
/// Serde: journaled by the durable-execution FanoutProgram (C6/P2.4).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LaneOutcome {
    pub lane_id: String,
    pub success: bool,
    /// Output name → artifact id, populated from the lane's
    /// `step.done_declared` event (or the `<lane>.failed` sentinel
    /// when the lane errored).
    pub outputs: BTreeMap<String, String>,
}

/// Decision produced by the aggregator after every lane has reported.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AggregateDecision {
    Advance { ok_count: usize },
    Fail { ok_count: usize, reason: String },
}

/// Decide whether the fanout step should advance or fail given the lane
/// outcomes and the `min_successful_lanes` threshold.
pub fn aggregate_lane_outcomes(
    outcomes: &[LaneOutcome],
    min_ok: usize,
    total: usize,
) -> AggregateDecision {
    let ok_count = outcomes.iter().filter(|o| o.success).count();
    if ok_count < min_ok {
        AggregateDecision::Fail {
            ok_count,
            reason: format!(
                "fanout: {ok_count}/{min_ok} required lanes succeeded (of {total} total)"
            ),
        }
    } else {
        AggregateDecision::Advance { ok_count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(lane_id: &str, success: bool) -> LaneOutcome {
        LaneOutcome {
            lane_id: lane_id.into(),
            success,
            outputs: BTreeMap::new(),
        }
    }

    #[test]
    fn all_succeed_advances() {
        let outcomes = vec![outcome("a", true), outcome("b", true), outcome("c", true)];
        assert_eq!(
            aggregate_lane_outcomes(&outcomes, 2, 3),
            AggregateDecision::Advance { ok_count: 3 }
        );
    }

    #[test]
    fn min_ok_satisfied_advances_with_partial_success() {
        let outcomes = vec![outcome("a", true), outcome("b", false), outcome("c", true)];
        assert_eq!(
            aggregate_lane_outcomes(&outcomes, 2, 3),
            AggregateDecision::Advance { ok_count: 2 }
        );
    }

    #[test]
    fn below_min_ok_fails_with_reason() {
        let outcomes = vec![outcome("a", true), outcome("b", false), outcome("c", false)];
        match aggregate_lane_outcomes(&outcomes, 2, 3) {
            AggregateDecision::Fail { ok_count, reason } => {
                assert_eq!(ok_count, 1);
                assert!(reason.contains("1/2"));
                assert!(reason.contains("of 3 total"));
            }
            _ => panic!("expected Fail"),
        }
    }

    #[test]
    fn empty_outcomes_zero_min_advances() {
        assert_eq!(
            aggregate_lane_outcomes(&[], 0, 0),
            AggregateDecision::Advance { ok_count: 0 }
        );
    }
}
