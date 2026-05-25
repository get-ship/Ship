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

/// One op's fixture binding: dotted op name → (service, rpc method, validator).
/// The service/method name the wire path uses; the validator pins the proto type.
struct Binding {
    op: &'static str,
    service: &'static str,
    method: &'static str,
    validate: fn(&Value) -> Result<()>,
}

/// The typed-response ops covered so far. Grows as more ops gain typed proto
/// messages (JsonResponse ops have no shape to validate, so they're absent).
fn bindings() -> Vec<Binding> {
    use ship_ops_rpc::*;
    macro_rules! b {
        ($op:literal, $svc:literal, $m:literal, $t:ty) => {
            Binding { op: $op, service: $svc, method: $m, validate: check::<$t> }
        };
    }
    vec![
        b!("rules.list", "rules_service", "List", RulesIndexResponse),
        b!("rules.get", "rules_service", "Get", RuleGetResponse),
        b!("git.status", "git_service", "Status", StatusBuckets),
        b!("git.log", "git_service", "Log", GitLogResponse),
        b!("git.branches", "git_service", "Branches", GitBranchesResponse),
        b!("git.overview", "git_service", "Overview", GitOverviewResponse),
    ]
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

    // service file → { Method: { response_ok: value } }
    let mut by_service: BTreeMap<&str, serde_json::Map<String, Value>> = BTreeMap::new();
    let mut count = 0usize;
    for b in bindings() {
        let value = responses
            .get(b.op)
            .with_context(|| format!("theme.json: no `responses.{}` for {}.{}", b.op, b.service, b.method))?;
        (b.validate)(value).with_context(|| {
            format!("fixture `{}` does not match proto {}.{}", b.op, b.service, b.method)
        })?;
        by_service
            .entry(b.service)
            .or_default()
            .insert(b.method.to_string(), json!({ "response_ok": value }));
        count += 1;
    }

    for (service, methods) in &by_service {
        let path = fixtures_dir.join(format!("{service}.json"));
        let body = serde_json::to_string_pretty(&Value::Object(methods.clone()))?;
        std::fs::write(&path, format!("{body}\n"))
            .with_context(|| format!("write {}", path.display()))?;
    }
    eprintln!(
        "gen-fixtures: {count} op fixtures → {} service file(s) in {}",
        by_service.len(),
        fixtures_dir.display()
    );
    Ok(())
}
