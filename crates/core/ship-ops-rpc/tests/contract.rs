//! Connect-RPC contract tests for the `ship.ops.v1` services — Rust side.
//!
//! Asserts the generated proto types parse the committed fixtures in
//! `proto/fixtures/<service>.json` (emitted by `cargo xtask gen-fixtures` from
//! the themed corpus). The TS Connect client asserts the inverse against the
//! same files. Both reading one fixture catches wire drift — field renames,
//! int64 encoding, default handling, a method added on one side only — the same
//! guard the gateway's `auth_service.json` contract test provides.
//!
//! These fixtures are also the realistic seed corpus for fuzzing and the
//! design-handoff reference (see proto/fixtures/theme.json).

use serde_json::Value;
use ship_ops_rpc::{
    GitBranchesResponse, GitLogResponse, GitOverviewResponse, RuleGetResponse, RulesIndexResponse,
    StatusBuckets,
};

const RULES: &str = include_str!("../../../../proto/fixtures/rules_service.json");
const GIT: &str = include_str!("../../../../proto/fixtures/git_service.json");

/// The `response_ok` body for a method, as a JSON string ready to decode.
fn response_ok(fixtures_json: &str, method: &str) -> String {
    let v: Value = serde_json::from_str(fixtures_json).expect("fixtures parse");
    serde_json::to_string(&v[method]["response_ok"]).expect("response_ok serialize")
}

#[test]
fn rules_list_parses_and_carries_the_theme() {
    let r: RulesIndexResponse =
        serde_json::from_str(&response_ok(RULES, "List")).expect("parse RulesIndexResponse");
    assert_eq!(r.items.len(), 3);
    assert_eq!(r.items[0].id, "no-placeholders");
    assert_eq!(r.items[0].scope, "project");
}

#[test]
fn rules_get_parses() {
    let r: RuleGetResponse =
        serde_json::from_str(&response_ok(RULES, "Get")).expect("parse RuleGetResponse");
    assert_eq!(r.id, "no-placeholders");
    assert!(r.body.starts_with("# No placeholders"));
}

#[test]
fn git_status_parses_nested_files() {
    let s: StatusBuckets =
        serde_json::from_str(&response_ok(GIT, "Status")).expect("parse StatusBuckets");
    assert!(!s.clean);
    assert_eq!(s.total, 3);
    assert_eq!(s.staged.len(), 1);
    assert_eq!(s.staged[0].basename, "checkout.rs");
}

#[test]
fn git_log_parses_commits() {
    let l: GitLogResponse =
        serde_json::from_str(&response_ok(GIT, "Log")).expect("parse GitLogResponse");
    assert_eq!(l.commits.len(), 2);
    assert_eq!(l.commits[0].author, "rust-lane");
    assert_eq!(l.commits[0].additions, 142);
}

#[test]
fn git_branches_parses_upstream_and_counts() {
    let b: GitBranchesResponse =
        serde_json::from_str(&response_ok(GIT, "Branches")).expect("parse GitBranchesResponse");
    assert_eq!(b.branches.len(), 2);
    assert!(b.branches[0].active);
    assert_eq!(b.branches[0].upstream.as_deref(), Some("origin/feat/checkout-flow"));
    assert_eq!(b.ahead, 2);
}

#[test]
fn git_overview_parses_the_aggregate() {
    let o: GitOverviewResponse =
        serde_json::from_str(&response_ok(GIT, "Overview")).expect("parse GitOverviewResponse");
    assert_eq!(o.branch.as_deref(), Some("feat/checkout-flow"));
    assert_eq!(o.total_changed, 3);
    assert_eq!(o.commits.len(), 1);
}
