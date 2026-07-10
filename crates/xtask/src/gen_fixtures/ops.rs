//! Op RESPONSE fixtures from the themed corpus.
//!
//! Reads `proto/fixtures/fixture-seed.json`, validates each op response value
//! against its typed proto type, and emits `proto/fixtures/<service>.json` keyed
//! `{ "<Method>": { "response_ok": <value> } }`.
//!
//! This is the original gen_fixtures.rs logic, moved into a submodule during
//! the R1 directory split. No logic has changed.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::{json, Value};

/// Validate a corpus value against the proto response type `T`.
fn check<T: serde::de::DeserializeOwned>(v: &Value) -> Result<()> {
    serde_json::from_value::<T>(v.clone()).map(|_| ()).map_err(|e| anyhow::anyhow!(e))
}

/// Decode-validator per proto RESPONSE type. Keyed by the proto message's
/// simple name (`MethodPolicy.output_type`). A new method reusing an existing
/// type is covered automatically; only a new RESPONSE TYPE needs an entry here.
fn type_validators() -> BTreeMap<&'static str, fn(&Value) -> Result<()>> {
    use ship_ops_rpc::*;
    macro_rules! v {
        ($t:ty) => {
            (stringify!($t), check::<$t> as fn(&Value) -> Result<()>)
        };
    }
    [
        v!(ScopedMutation), v!(Deletion),
        v!(CollectionList), v!(CollectionDetail), v!(ItemPage), v!(Item), v!(WatchEvent),
        v!(SearchResponse),
        v!(AgentIndexResponse), v!(AgentsSummariesResponse), v!(AgentDetail),
        v!(AgentTemplatesResponse), v!(PermissionPresetsResponse), v!(HarnessesResponse),
        // S4: the four structured reads + the opaque-doc read.
        v!(AgentCatalogsResponse), v!(AgentWireResponse), v!(AgentSettingsResponse), v!(AgentDoc),
        // S4: the four typed control results.
        v!(DispatchResult), v!(StopResult), v!(SteerResult), v!(GrantResult),
        v!(AnalyticsSessionsResponse), v!(AnalyticsSessionResponse),
        v!(AnalyticsToolUsageResponse), v!(AnalyticsSessionTurnsResponse),
        v!(ArtifactsListResponse), v!(ArtifactRecord),
        v!(CollectionItem),
        v!(EventsListResponse),
        v!(StatusBuckets), v!(GitLogResponse), v!(GitBranchesResponse),
        v!(GitOverviewResponse), v!(GitDiffResponse),
        v!(McpIndexResponse), v!(MetaResponse),
        v!(PermissionsIndexResponse), v!(GetPermissionResponse), v!(ToolPatternsResponse),
        v!(PipelinesIndexResponse),
        // engine cut 2026-07-10: pipelines.* typed run-level reads retired
        // (Summaries / Run / History / ListRuns / Cancel / ResolveStep);
        // the proto's PipelinesService now exposes List/Get/Put/Patch/Delete
        // only. PipelinesIndexResponse (List) remains.
        v!(WorkspaceActiveResponse), v!(ProjectRecord),
        v!(RulesIndexResponse), v!(RuleGetResponse),
        v!(ListProcsResponse), v!(SessionsListResponse), v!(SettingValue),
        v!(SkillsIndexResponse), v!(SkillFileResponse),
        // space.* (clean SpaceService — ship-space crate): typed identity +
        v!(Space), v!(SpaceList), v!(SpaceInfo),
        // settings.* (unified, scope-discriminated SettingsService — ship-config
        // crate): the settings keystone. Replaces the retired ConfigService.
        v!(Setting), v!(SettingList),
        // account.* (typed AccountService — ship-account crate): Ship Cloud
        // identity. State snapshot + the PKCE login/logout lifecycle results.
        v!(AccountState), v!(LoginStarted), v!(LoginCompleted), v!(LogoutResult),
        // isolation.* (typed IsolationService — ship-isolation crate): capability
        // probe + recommender results + wizard previews. The data-bearing
        // `IsolationResolutionReason` is a proto `oneof`; the conformance gate
        // decodes the oneof arm's payload via buffa's generated serde impl.
        v!(IsolationStatusResponse), v!(IsolationRecommendationsResponse),
        v!(IsolationPreviewResponse), v!(IsolationVerifyResponse),
        // Scaffold + feedback/annotation typed results (proto-added; fixtures backfilled).
        v!(SkillScaffoldResponse), v!(ViewScaffoldResponse),
        v!(PostFeedbackResponse), v!(PostAnnotationResponse),
        // UI discovery reads + terminal session/stop typed results
        // (proto-added; fixtures backfilled on this branch).
        v!(UiQueryResponse), v!(UiReadResponse),
        v!(TerminalSession), v!(StopTerminalResponse),
        v!(ListTerminalsResponse), v!(SendInputResponse), v!(ResizeTerminalResponse),
        v!(BytesChunk),
     ]
     .into_iter()
     .collect()
}

use ship_ops_rpc::policy::{op_name, snake};

/// The generated umbrella + streaming services — NOT per-domain fixture targets.
fn is_non_fixture_service(service: &str) -> bool {
    // DeviceService is unfinished (#130) — typed rpcs, fixtures/validators land
    // with the service. Skip until then.
    matches!(service, "ReadOpsService" | "CommandOpsService" | "StreamService" | "DeviceService")
}

/// PascalCase an op name (`rules.create` → `RulesCreate`).
fn pascal(op: &str) -> String {
    op.split(['.', '_'])
        .map(|seg| {
            let mut c = seg.chars();
            c.next().map(|f| f.to_uppercase().chain(c).collect::<String>()).unwrap_or_default()
        })
        .collect()
}

/// Count ops exposed on BOTH the generated umbrella and a per-domain service.
pub fn scaffold_dup_count() -> usize {
    use std::collections::BTreeSet;
    let policies = ship_ops_rpc::policy::read_policies();
    let umbrella: BTreeSet<String> = policies
        .iter()
        .filter(|p| matches!(p.service.as_str(), "ReadOpsService" | "CommandOpsService"))
        .map(|p| p.method.clone())
        .collect();
    policies
        .iter()
        .filter(|p| !is_non_fixture_service(&p.service) && p.service != "OpsService")
        .map(|p| op_name(&p.service, &p.method))
        .filter(|op| umbrella.contains(&pascal(op)))
        .collect::<BTreeSet<_>>()
        .len()
}

/// Run response-fixture generation. Returns count of fixtures written.
pub fn run(fixtures_dir: &Path, check_mode: bool) -> Result<usize> {
    let theme_path = fixtures_dir.join("fixture-seed.json");
    let theme: Value = serde_json::from_slice(
        &std::fs::read(&theme_path).with_context(|| format!("read {}", theme_path.display()))?,
    )
    .with_context(|| format!("parse {}", theme_path.display()))?;
    let responses = theme
        .get("responses")
        .and_then(Value::as_object)
        .context("fixture-seed.json: missing `responses` object")?;

    let validators = type_validators();
    let mut by_service: BTreeMap<String, serde_json::Map<String, Value>> = BTreeMap::new();
    let mut count = 0usize;

    for p in ship_ops_rpc::policy::read_policies() {
        if is_non_fixture_service(&p.service) {
            continue;
        }
        if p.output_type.is_empty() || p.output_type == "JsonResponse" {
            continue;
        }
        let validate = validators.get(p.output_type.as_str()).with_context(|| {
            format!(
                "{}.{} returns typed `{}` but no fixture validator exists — \
                 add `v!({})` to gen_fixtures::ops::type_validators",
                p.service, p.method, p.output_type, p.output_type
            )
        })?;
        let op = op_name(&p.service, &p.method);
        let value = responses.get(&op).with_context(|| {
            format!("fixture-seed.json: no `responses.{op}` for {}.{}", p.service, p.method)
        })?;
        validate(value).with_context(|| {
            format!("fixture `{op}` does not match proto {}.{}", p.service, p.method)
        })?;
        by_service
            .entry(snake(&p.service))
            .or_default()
            .insert(p.method.clone(), json!({ "response_ok": value }));
        count += 1;
    }

    for (service, methods) in &by_service {
        let path = fixtures_dir.join(format!("{service}.json"));
        // Read existing file (if any) so we PRESERVE the `request` field that
        // op_requests::run owns. ops::run only owns `response_ok` per method.
        let existing_methods: serde_json::Map<String, Value> = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        let mut merged = serde_json::Map::new();
        for (method, response_entry) in methods {
            let mut entry = existing_methods
                .get(method)
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();
            // Replace response_ok with our authoritative version; preserve other keys (request).
            entry.insert(
                "response_ok".to_string(),
                response_entry["response_ok"].clone(),
            );
            merged.insert(method.clone(), Value::Object(entry));
        }
        let body = serde_json::to_string_pretty(&Value::Object(merged))?;
        let content = format!("{body}\n");
        if check_mode {
            let existing_raw = std::fs::read_to_string(&path).unwrap_or_default();
            if existing_raw != content {
                anyhow::bail!("op fixture drift: {} is out of date", path.display());
            }
        } else {
            std::fs::write(&path, &content)
                .with_context(|| format!("write {}", path.display()))?;
        }
    }

    Ok(count)
}

/// List all typed op methods and whether they have a fixture in fixture-seed.json.
pub fn missing_fixtures(fixtures_dir: &Path) -> Result<Vec<String>> {
    let theme_path = fixtures_dir.join("fixture-seed.json");
    let theme: Value = serde_json::from_slice(&std::fs::read(&theme_path)?)?;
    let responses = theme
        .get("responses")
        .and_then(Value::as_object)
        .context("fixture-seed.json: missing `responses` object")?;
    let validators = type_validators();
    let mut missing = Vec::new();
    for p in ship_ops_rpc::policy::read_policies() {
        if is_non_fixture_service(&p.service) { continue; }
        if p.output_type.is_empty() || p.output_type == "JsonResponse" { continue; }
        let op = op_name(&p.service, &p.method);
        if !validators.contains_key(p.output_type.as_str()) {
            missing.push(format!("{op}: no decode-validator for `{}`", p.output_type));
        } else if !responses.contains_key(&op) {
            missing.push(format!("{op}: no corpus fixture in fixture-seed.json"));
        }
    }
    Ok(missing)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme_responses() -> serde_json::Map<String, Value> {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let theme: Value = serde_json::from_slice(
            &std::fs::read(root.join("proto/fixtures/fixture-seed.json")).unwrap(),
        )
        .unwrap();
        theme.get("responses").and_then(Value::as_object).unwrap().clone()
    }

    /// THE CONTRACT-COMPLETENESS GATE (fixtures axis): every typed per-domain
    /// proto method must have a decode-validator AND a corpus fixture.
    #[test]
    fn every_typed_method_has_a_validator_and_fixture() {
        let validators = type_validators();
        let responses = theme_responses();
        let mut gaps = Vec::new();
        for p in ship_ops_rpc::policy::read_policies() {
            if is_non_fixture_service(&p.service) { continue; }
            if p.output_type.is_empty() || p.output_type == "JsonResponse" { continue; }
            let op = op_name(&p.service, &p.method);
            let Some(validate) = validators.get(p.output_type.as_str()) else {
                gaps.push(format!("{op}: response `{}` has no validator", p.output_type));
                continue;
            };
            match responses.get(&op) {
                Some(v) => {
                    if let Err(e) = validate(v) {
                        gaps.push(format!("{op}: corpus does not match `{}`: {e}", p.output_type));
                    }
                }
                None => gaps.push(format!("{op}: no corpus fixture in fixture-seed.json")),
            }
        }
        assert!(gaps.is_empty(), "contract-completeness gaps (fixtures):\n  {}", gaps.join("\n  "));
    }
}
