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
//! design-handoff reference (see proto/fixtures/fixture-seed.json).

use serde_json::Value;
use ship_ops_rpc::{
    AgentCatalogsResponse, AgentDetail, AgentDoc, AgentSettingsResponse, AgentWireResponse,
    AgentsSummariesResponse, AnalyticsSessionsResponse, AnalyticsSessionTurnsResponse,
    AnalyticsToolUsageResponse, RuleGetResponse, RulesIndexResponse,
};

const AGENTS: &str = include_str!("../../../../proto/fixtures/agents_service.json");
const ANALYTICS: &str = include_str!("../../../../proto/fixtures/analytics_service.json");
const RULES: &str = include_str!("../../../../proto/fixtures/rules_service.json");

/// The `response_ok` body for a method, as a `Value` ready to decode via
/// `from_value`. Returns the parsed node directly rather than re-serializing to
/// a string: under workspace feature-unification `serde_json/arbitrary_precision`
/// is enabled (pulled in via `schemars`), and round-tripping a `Value` back
/// through `to_string` emits the internal Number magic-map token, which then
/// fails to decode into `f64` fields ("invalid type: map, expected a double").
/// `from_value` converts arbitrary-precision numbers to `f64` correctly.
fn response_ok(fixtures_json: &str, method: &str) -> Value {
    let v: Value = serde_json::from_str(fixtures_json).expect("fixtures parse");
    v[method]["response_ok"].clone()
}

#[test]
fn rules_list_parses_and_carries_the_theme() {
    let r: RulesIndexResponse =
        serde_json::from_value(response_ok(RULES, "List")).expect("parse RulesIndexResponse");
    assert_eq!(r.items.len(), 3);
    assert_eq!(r.items[0].id, "no-placeholders");
    assert_eq!(r.items[0].scope, "project");
}

#[test]
fn rules_get_parses() {
    let r: RuleGetResponse =
        serde_json::from_value(response_ok(RULES, "Get")).expect("parse RuleGetResponse");
    assert_eq!(r.id, "no-placeholders");
    assert!(r.body.starts_with("# No placeholders"));
}

#[test]
fn agents_list_parses_index() {
    let r: ship_ops_rpc::AgentIndexResponse =
        serde_json::from_value(response_ok(AGENTS, "List")).expect("parse AgentIndexResponse");
    assert_eq!(r.items.len(), 2);
    assert_eq!(r.items[0].id, "rust-lane");
}

#[test]
fn agents_summaries_parses_active_flag() {
    let r: AgentsSummariesResponse =
        serde_json::from_value(response_ok(AGENTS, "Summaries")).expect("parse AgentsSummariesResponse");
    assert_eq!(r.items.len(), 2);
    assert!(r.items[0].active);
    assert_eq!(r.items[0].skills_count, 2);
}

#[test]
fn agents_detail_parses_permissions() {
    let d: AgentDetail =
        serde_json::from_value(response_ok(AGENTS, "Detail")).expect("parse AgentDetail");
    assert_eq!(d.id, "rust-lane");
    assert_eq!(d.skills.len(), 2);
    assert_eq!(d.permissions.preset, "standard");
    assert!(!d.permissions.tools_allow.is_empty());
}

// ── S4: the four reads that were JsonResponse become typed ────────────────

#[test]
fn agents_catalogs_parses_structured_rows() {
    // Catalogs is now field-by-field structured (skills/mcp/rules/presets +
    // tool_patterns), not an opaque JSON blob.
    let c: AgentCatalogsResponse =
        serde_json::from_value(response_ok(AGENTS, "Catalogs")).expect("parse AgentCatalogsResponse");
    assert!(!c.skills.is_empty());
    assert_eq!(c.skills[0].id, "run");
    assert!(!c.tool_patterns.is_empty());
    assert_eq!(c.tool_patterns[0].source, "harness");
}

#[test]
fn agents_wire_parses_nodes_and_edges() {
    // Wire is now structured: the typed agent detail + positioned nodes +
    // bezier edges + the picker catalogs.
    let w: AgentWireResponse =
        serde_json::from_value(response_ok(AGENTS, "Wire")).expect("parse AgentWireResponse");
    assert_eq!(w.agent.id, "rust-lane");
    assert!(!w.nodes.is_empty());
    assert_eq!(w.nodes[0].kind, "agent");
    assert!(!w.edges.is_empty());
    // Edges count is nodes - 1 (every peripheral wires to the center).
    assert_eq!(w.edges.len(), w.nodes.len() - 1);
}

#[test]
fn agents_settings_parses_tiers() {
    // Settings is now structured: typed agent detail + the tier triad +
    // catalogs.
    let s: AgentSettingsResponse =
        serde_json::from_value(response_ok(AGENTS, "Settings")).expect("parse AgentSettingsResponse");
    assert_eq!(s.agent.id, "rust-lane");
    assert_eq!(s.tiers.len(), 3);
    assert!(s.tiers.iter().any(|t| t.selected));
}

#[test]
fn agents_get_carries_opaque_doc_json() {
    // Get is opaque-by-design: id + resolved scope are typed; the def body
    // rides as a serialized JSON string (`doc_json`), NOT field-by-field.
    let d: AgentDoc =
        serde_json::from_value(response_ok(AGENTS, "Get")).expect("parse AgentDoc");
    assert_eq!(d.id, "rust-lane");
    assert_eq!(d.scope, "project");
    assert!(!d.doc_json.is_empty());
    // The string round-trips back to the def object it wrapped.
    let parsed: Value = serde_json::from_str(&d.doc_json).expect("doc_json is valid JSON");
    assert!(parsed.get("agent").is_some(), "doc_json carries the raw def under `agent`");
}

#[test]
fn analytics_list_sessions_parses_tool_counts() {
    let r: AnalyticsSessionsResponse =
        serde_json::from_value(response_ok(ANALYTICS, "ListSessions")).expect("parse AnalyticsSessionsResponse");
    assert_eq!(r.sessions.len(), 2);
    assert_eq!(r.sessions[0].session_key, "sess-rust-lane-1");
    assert_eq!(r.sessions[0].tool_use_count, 42);
}

#[test]
fn analytics_tool_usage_parses_per_tool() {
    let r: AnalyticsToolUsageResponse =
        serde_json::from_value(response_ok(ANALYTICS, "ToolUsage")).expect("parse AnalyticsToolUsageResponse");
    assert_eq!(r.tools[0].tool, "Bash");
    assert_eq!(r.tools[0].calls, 28);
}

#[test]
fn analytics_session_turns_parses_cache_tokens() {
    let r: AnalyticsSessionTurnsResponse =
        serde_json::from_value(response_ok(ANALYTICS, "SessionTurns")).expect("parse AnalyticsSessionTurnsResponse");
    assert_eq!(r.turns.len(), 2);
    assert_eq!(r.turns[0].cache_creation_tokens, 3800);
    assert_eq!(r.turns[1].cache_read_tokens, 3800);
}
