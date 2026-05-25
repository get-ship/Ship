//! Generate proto contract fixtures from the themed corpus (`cargo xtask gen-fixtures`).
//!
//! Source of VALUES: `proto/fixtures/theme.json` — one coherent demo Ship
//! workspace (see its `_theme`). Source of SHAPE: the `ship.ops.v1` proto
//! messages. For each op we decode the corpus value into its generated
//! connectrpc response type (the drift guard — a renamed/retyped field fails
//! here), then emit `proto/fixtures/<service>.json` keyed
//! `{ "<Method>": { "response_ok": <value> } }` — the same shape as the
//! hand-authored gateway fixture (`auth_service.json`).
//!
//! Three consumers: cross-language CONTRACT tests (Rust parses + TS Connect
//! client decodes the same file → wire drift caught), FUZZ seed corpus (the
//! FDS is the schema, these are the seeds), and DESIGN HANDOFF (realistic
//! coherent data, not placeholders).
//!
//! Only typed responses get a fixture; ops still returning the opaque
//! `JsonResponse` have no shape to validate and are skipped until they gain a
//! typed proto message (which the proto-SSOT migration is doing).

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{json, Value};

/// Validate a corpus value against the proto response type `T` (decode-only;
/// `T: Deserialize` is all the generated types guarantee). Non-capturing, so it
/// coerces to a `fn` pointer in the table below.
fn check<T: serde::de::DeserializeOwned>(v: &Value) -> Result<()> {
    serde_json::from_value::<T>(v.clone()).map(|_| ()).map_err(|e| anyhow::anyhow!(e))
}

/// Decode-validator per proto RESPONSE type. The one hand-maintained input —
/// decode-validation needs the concrete generated Rust type, which a descriptor
/// alone can't supply. Keyed by the proto message's simple name (the
/// `MethodPolicy.output_type`). A NEW method reusing an existing type is covered
/// automatically; only a new RESPONSE TYPE needs an entry here. The contract
/// gate fails if a typed method's response type is missing from this map.
fn type_validators() -> BTreeMap<&'static str, fn(&Value) -> Result<()>> {
    use ship_ops_rpc::*;
    macro_rules! v {
        ($t:ty) => {
            (stringify!($t), check::<$t> as fn(&Value) -> Result<()>)
        };
    }
    [
        v!(ScopedMutation), v!(Deletion),
        v!(AgentIndexResponse), v!(AgentsSummariesResponse), v!(AgentDetail),
        v!(AgentTemplatesResponse), v!(PermissionPresetsResponse), v!(HarnessesResponse),
        v!(AnalyticsSessionsResponse), v!(AnalyticsSessionResponse),
        v!(AnalyticsToolUsageResponse), v!(AnalyticsSessionTurnsResponse),
        v!(ArtifactsListResponse), v!(ArtifactRecord),
        v!(ChatStateResponse), v!(CollectionItem),
        v!(EventsListResponse),
        v!(StatusBuckets), v!(GitLogResponse), v!(GitBranchesResponse),
        v!(GitOverviewResponse), v!(GitDiffResponse),
        v!(McpIndexResponse), v!(MetaResponse),
        v!(PermissionsIndexResponse), v!(ToolPatternsResponse),
        v!(PipelinesIndexResponse),
        v!(WorkspaceActiveResponse), v!(ProjectRecord),
        v!(RulesIndexResponse), v!(RuleGetResponse),
        v!(ListProcsResponse), v!(SessionsListResponse), v!(SettingValue),
        v!(SkillsIndexResponse), v!(SkillFileResponse),
        v!(SpecsListResponse), v!(SpecGetResponse), v!(SpecDetail),
        v!(PostViewErrorResponse), v!(ListViewErrorsResponse), v!(ClearViewErrorsResponse),
    ]
    .into_iter()
    .collect()
}

/// snake_case a PascalCase proto identifier: `WriteFrontmatter` → `write_frontmatter`.
fn snake(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.char_indices() {
        if ch.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

/// The generated umbrella + streaming services — NOT per-domain fixture targets.
/// `ReadOpsService`/`CommandOpsService` are the bulk-port scaffold (every op is
/// ALSO on its per-domain service → fixtures there, not here); `StreamService`
/// is server-streaming (no unary response fixture).
fn is_non_fixture_service(service: &str) -> bool {
    matches!(service, "ReadOpsService" | "CommandOpsService" | "StreamService")
}

/// Reconstruct the dotted op name from a proto (service, method): the service
/// carries the domain (`RulesService` → `rules`), the method the rest
/// (`Create` → `create`) ⇒ `rules.create`. Clean inverse — the service boundary
/// removes the `.`-vs-`_` ambiguity that pascal() alone has.
fn op_name(service: &str, method: &str) -> String {
    let svc = snake(service);
    let domain = svc.strip_suffix("_service").unwrap_or(&svc);
    format!("{domain}.{}", snake(method))
}

/// PascalCase an op name (`rules.create` → `RulesCreate`) — mirrors the umbrella
/// method namer, so a per-domain op can be checked for an umbrella twin.
fn pascal(op: &str) -> String {
    op.split(['.', '_'])
        .map(|seg| {
            let mut c = seg.chars();
            c.next().map(|f| f.to_uppercase().chain(c).collect::<String>()).unwrap_or_default()
        })
        .collect()
}

/// Count ops exposed on BOTH the generated umbrella (ReadOps/CommandOps) AND a
/// per-domain service — the transitional scaffold double-exposure. Reported (not
/// gated on a magic number): it shrinks to 0 as the umbrella is retired (#78).
/// Derived live from the descriptor, so it's never stale.
fn scaffold_dup_count() -> usize {
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

pub fn run() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures_dir = root.join("proto/fixtures");
    let theme_path = fixtures_dir.join("theme.json");
    let theme: Value = serde_json::from_slice(
        &std::fs::read(&theme_path).with_context(|| format!("read {}", theme_path.display()))?,
    )
    .with_context(|| format!("parse {}", theme_path.display()))?;
    let responses = theme
        .get("responses")
        .and_then(Value::as_object)
        .context("theme.json: missing `responses` object")?;

    let validators = type_validators();
    // service file → { Method: { response_ok: value } }
    let mut by_service: BTreeMap<String, serde_json::Map<String, Value>> = BTreeMap::new();
    let mut count = 0usize;

    // ENUMERATE FROM THE PROTO (the SSOT) — every authored rpc method. Untyped
    // (JsonResponse) methods have no shape to validate and are skipped; the
    // generated umbrella + streaming services are not per-domain fixture targets.
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
                 add `v!({})` to gen_fixtures::type_validators",
                p.service, p.method, p.output_type, p.output_type
            )
        })?;
        let op = op_name(&p.service, &p.method);
        let value = responses.get(&op).with_context(|| {
            format!("theme.json: no `responses.{op}` for {}.{}", p.service, p.method)
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
        let body = serde_json::to_string_pretty(&Value::Object(methods.clone()))?;
        std::fs::write(&path, format!("{body}\n"))
            .with_context(|| format!("write {}", path.display()))?;
    }
    eprintln!(
        "gen-fixtures: {count} op fixtures (proto-enumerated) → {} service file(s) in {}",
        by_service.len(),
        fixtures_dir.display()
    );
    eprintln!(
        "gen-fixtures: scaffold dup — {} ops still double-exposed on the generated \
         umbrella + their per-domain service (retires to 0 with the umbrella, #78)",
        scaffold_dup_count()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme_responses() -> serde_json::Map<String, Value> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let theme: Value =
            serde_json::from_slice(&std::fs::read(root.join("proto/fixtures/theme.json")).unwrap())
                .unwrap();
        theme.get("responses").and_then(Value::as_object).unwrap().clone()
    }

    /// THE CONTRACT-COMPLETENESS GATE (fixtures axis): every typed per-domain
    /// proto method must have a decode-validator AND a corpus fixture. A newly
    /// authored service method with a typed response but no fixture fails here —
    /// the seam becomes a hard CI failure, not silent drift.
    #[test]
    fn every_typed_method_has_a_validator_and_fixture() {
        let validators = type_validators();
        let responses = theme_responses();
        let mut gaps = Vec::new();
        for p in ship_ops_rpc::policy::read_policies() {
            if is_non_fixture_service(&p.service) {
                continue;
            }
            if p.output_type.is_empty() || p.output_type == "JsonResponse" {
                continue;
            }
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
                None => gaps.push(format!("{op}: no corpus fixture in theme.json")),
            }
        }
        assert!(gaps.is_empty(), "contract-completeness gaps (fixtures):\n  {}", gaps.join("\n  "));
    }
}
