//! Consume the daemon's OpenAPI spec and re-key it as `{METHOD} {path}` with a
//! small `purpose` classifier derived from the verb + path segments. Request
//! and response `$ref`s are preserved so downstream consumers can follow them.

use anyhow::Result;
use serde_json::{json, Map, Value};
use utoipa::OpenApi;

pub fn extract() -> Result<Value> {
    let spec = shipd::openapi::ApiDoc::openapi();
    let json_text = spec.to_json()?;
    let root: Value = serde_json::from_str(&json_text)?;
    let mut out = Map::new();
    let Some(paths) = root.get("paths").and_then(|v| v.as_object()) else {
        return Ok(Value::Object(out));
    };

    for (path, methods) in paths {
        let Some(ops) = methods.as_object() else { continue };
        for (method, op) in ops {
            let key = format!("{} {}", method.to_uppercase(), path);
            let params: Vec<String> = extract_path_params(path);
            let request_ref = op
                .get("requestBody")
                .and_then(|b| b.get("content"))
                .and_then(|c| c.get("application/json"))
                .and_then(|j| j.get("schema"))
                .cloned()
                .unwrap_or(Value::Null);
            let response_ref = op
                .get("responses")
                .and_then(|r| r.as_object())
                .and_then(|r| {
                    // Prefer 200, then first 2xx.
                    r.get("200").or_else(|| {
                        r.iter()
                            .find(|(k, _)| k.starts_with('2'))
                            .map(|(_, v)| v)
                    })
                })
                .and_then(|r| r.get("content"))
                .and_then(|c| c.get("application/json"))
                .and_then(|j| j.get("schema"))
                .cloned()
                .unwrap_or(Value::Null);

            out.insert(
                key,
                json!({
                    "method": method.to_uppercase(),
                    "path": path,
                    "purpose": infer_purpose(method, path),
                    "params": params,
                    "summary": op.get("summary").cloned().unwrap_or(Value::Null),
                    "description": op.get("description").cloned().unwrap_or(Value::Null),
                    "request": request_ref,
                    "response": response_ref,
                }),
            );
        }
    }
    Ok(Value::Object(out))
}

fn extract_path_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|seg| {
            let seg = seg.trim();
            if seg.starts_with('{') && seg.ends_with('}') {
                Some(seg.trim_matches(|c| c == '{' || c == '}').to_string())
            } else {
                None
            }
        })
        .collect()
}

fn infer_purpose(method: &str, path: &str) -> &'static str {
    let m = method.to_uppercase();
    if path.ends_with("/sse") || path.contains("/stream") {
        return "sse";
    }
    match m.as_str() {
        "GET" if path.contains("/view/") => "view",
        "GET" => "read",
        "POST" => "create",
        "PUT" | "PATCH" => "update",
        "DELETE" => "delete",
        _ => "action",
    }
}
