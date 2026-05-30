//! Per-op REQUEST fixtures, extending each service JSON with a `"request"` key.
//!
//! Reads the policy args from `ship_ops_rpc::policy::read_policies()` and
//! generates deterministic request stubs for each typed op. These extend the
//! `proto/fixtures/<service>.json` entries that `ops.rs` emits for responses:
//! each method entry gains `"request": { ... }` alongside `"response_ok"`.
//!
//! Generation strategy: for each `MethodArg`, produce a sensible default value
//! based on its `kind`. String args get `"test-<field>"`, number args get 0,
//! bool args get false, repeated args get `[]`, map args get `{}`.
//! Required context args (like `space_id`, `session_id`) get more realistic values.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{json, Value};
use ship_ops_rpc::policy::{MethodArg, MethodPolicy, op_name, read_policies, snake};

/// The generated umbrella + streaming services — not fixture targets.
fn is_non_fixture_service(service: &str) -> bool {
    matches!(service, "ReadOpsService" | "CommandOpsService" | "StreamService")
}

/// Generate a deterministic stub value for a single request arg.
fn stub_arg(arg: &MethodArg) -> Value {
    let name = arg.name.as_str();
    // High-signal arg names get realistic stubs so the fixture reads well.
    if matches!(name, "space_id" | "spaceId") {
        return json!("test-space-id");
    }
    if matches!(name, "session_id" | "sessionId") {
        return json!("test-session-id");
    }
    if matches!(name, "agent_id" | "agentId" | "agent") {
        return json!("test-agent");
    }
    if matches!(name, "slug") {
        return json!("test-slug");
    }
    if matches!(name, "branch") {
        return json!("main");
    }
    if matches!(name, "path") {
        return json!("test/path.md");
    }
    if matches!(name, "name") {
        return json!("test-name");
    }
    if matches!(name, "content" | "body" | "text") {
        return json!("test content");
    }
    if matches!(name, "id") {
        return json!("test-id");
    }

    match arg.kind.as_str() {
        "string" => json!(format!("test-{name}")),
        "number" => json!(0),
        "bool" => json!(false),
        k if k.ends_with("[]") => json!([]),
        "map" => json!({}),
        _ => json!(format!("test-{name}")),
    }
}

/// Build a request stub object for one op.
fn make_request_stub(policy: &MethodPolicy) -> Value {
    if policy.args.is_empty() {
        return json!({});
    }
    let mut obj = serde_json::Map::new();
    for arg in &policy.args {
        obj.insert(arg.name.clone(), stub_arg(arg));
    }
    Value::Object(obj)
}

/// Load existing service fixture file (returns empty map if not found).
fn load_service_file(path: &std::path::Path) -> Result<serde_json::Map<String, Value>> {
    if !path.exists() {
        return Ok(serde_json::Map::new());
    }
    let data = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let v: Value = serde_json::from_slice(&data)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(v.as_object().cloned().unwrap_or_default())
}

/// Run request-fixture generation — extends each service JSON with `"request"` keys.
/// Returns count of request stubs generated.
pub fn run(fixtures_dir: &PathBuf, check_mode: bool) -> Result<usize> {
    let policies: Vec<MethodPolicy> = read_policies();

    // Group by service file (same as ops.rs groups response_ok entries).
    let mut by_service: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
    for p in &policies {
        if is_non_fixture_service(&p.service) { continue; }
        if p.output_type.is_empty() || p.output_type == "JsonResponse" { continue; }
        let svc = snake(&p.service);
        by_service.entry(svc).or_default().insert(p.method.clone(), make_request_stub(p));
    }

    let mut count = 0usize;
    for (service, requests) in &by_service {
        let path = fixtures_dir.join(format!("{service}.json"));
        let mut file_map = load_service_file(&path)?;

        for (method, request_stub) in requests {
            let entry = file_map.entry(method.clone()).or_insert_with(|| json!({}));
            if let Some(obj) = entry.as_object_mut() {
                obj.insert("request".to_string(), request_stub.clone());
                count += 1;
            }
        }

        let body = serde_json::to_string_pretty(&Value::Object(file_map))?;
        let content = format!("{body}\n");
        if check_mode {
            let existing = std::fs::read_to_string(&path).unwrap_or_default();
            if existing != content {
                anyhow::bail!("op-request fixture drift: {} is out of date", path.display());
            }
        } else {
            std::fs::write(&path, &content)
                .with_context(|| format!("write {}", path.display()))?;
        }
    }

    Ok(count)
}

/// Return ops that have request args but currently only have a response fixture.
pub fn missing_requests(fixtures_dir: &PathBuf) -> Result<Vec<String>> {
    let policies = read_policies();
    let mut missing = Vec::new();
    for p in &policies {
        if is_non_fixture_service(&p.service) { continue; }
        if p.output_type.is_empty() || p.output_type == "JsonResponse" { continue; }
        if p.args.is_empty() { continue; } // no-arg ops don't need a request fixture
        let svc = snake(&p.service);
        let path = fixtures_dir.join(format!("{svc}.json"));
        if !path.exists() {
            missing.push(format!("{}: service file missing", op_name(&p.service, &p.method)));
            continue;
        }
        let data = std::fs::read(&path)?;
        let v: Value = serde_json::from_slice(&data)?;
        let has_request = v
            .get(&p.method)
            .and_then(|m| m.get("request"))
            .is_some();
        if !has_request {
            missing.push(format!("{}: missing request stub", op_name(&p.service, &p.method)));
        }
    }
    Ok(missing)
}
