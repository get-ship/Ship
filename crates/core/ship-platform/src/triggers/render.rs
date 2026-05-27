//! Minijinja rendering for hook action values.

use std::collections::BTreeMap;

use anyhow::{Context, Result, anyhow};
use minijinja::{Environment, Value as JinjaValue};
use serde_json::Value;

/// Convert a `serde_json::Value` into a minijinja value directly.
///
/// We can't use `JinjaValue::from_serialize`: the workspace enables
/// serde_json's `arbitrary_precision`, so a `Number` serializes as the magic
/// `{"$serde_json::private::Number": "..."}` struct. minijinja's serializer has
/// no knowledge of that token and produces a *map* instead of a number, which
/// silently broke every numeric comparison and interpolation (`n > 1` compared
/// a map, not 0). Mapping the value tree by hand keeps numbers as numbers.
fn json_to_jinja(value: &Value) -> JinjaValue {
    match value {
        Value::Null => JinjaValue::from(()),
        Value::Bool(b) => JinjaValue::from(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                JinjaValue::from(i)
            } else if let Some(u) = n.as_u64() {
                JinjaValue::from(u)
            } else if let Some(f) = n.as_f64() {
                JinjaValue::from(f)
            } else {
                JinjaValue::from(n.to_string())
            }
        }
        Value::String(s) => JinjaValue::from(s.clone()),
        Value::Array(arr) => arr.iter().map(json_to_jinja).collect::<JinjaValue>(),
        Value::Object(map) => {
            let entries: BTreeMap<String, JinjaValue> =
                map.iter().map(|(k, v)| (k.clone(), json_to_jinja(v))).collect();
            JinjaValue::from(entries)
        }
    }
}

/// Render every string leaf of `input` through minijinja with `context`.
/// Non-string leaves pass through unchanged.
pub fn render_value(input: &Value, context: &Value) -> Result<Value> {
    let env = Environment::new();
    let jinja_ctx = json_to_jinja(context);
    render_with(&env, input, &jinja_ctx)
}

/// Render a single template string.
pub fn render_string(template: &str, context: &Value) -> Result<String> {
    let env = Environment::new();
    let jinja_ctx = json_to_jinja(context);
    render_string_with(&env, template, &jinja_ctx)
}

/// Evaluate a minijinja expression as a boolean. Empty / missing = false.
///
/// Compiles `expr` as an expression and evaluates its truthiness directly,
/// rather than round-tripping through a `{% if %}` string template — the
/// template path stringifies operands before comparing, so `n > 1` with
/// `n = 0` wrongly took the truthy branch.
pub fn evaluate_when(expr: &str, context: &Value) -> Result<bool> {
    let env = Environment::new();
    let compiled = env
        .compile_expression(expr)
        .with_context(|| format!("compiling when: {expr}"))?;
    let jinja_ctx = json_to_jinja(context);
    let result = compiled
        .eval(jinja_ctx)
        .with_context(|| format!("evaluating when: {expr}"))?;
    Ok(result.is_true())
}

fn render_with(env: &Environment<'_>, input: &Value, ctx: &JinjaValue) -> Result<Value> {
    match input {
        Value::String(s) => Ok(Value::String(render_string_with(env, s, ctx)?)),
        Value::Array(arr) => {
            let rendered: Result<Vec<_>> =
                arr.iter().map(|v| render_with(env, v, ctx)).collect();
            Ok(Value::Array(rendered?))
        }
        Value::Object(map) => {
            let mut out = serde_json::Map::with_capacity(map.len());
            for (k, v) in map {
                out.insert(k.clone(), render_with(env, v, ctx)?);
            }
            Ok(Value::Object(out))
        }
        other => Ok(other.clone()),
    }
}

fn render_string_with(env: &Environment<'_>, template: &str, ctx: &JinjaValue) -> Result<String> {
    if !template.contains("{{") && !template.contains("{%") {
        return Ok(template.to_string());
    }
    let tmpl = env
        .template_from_str(template)
        .map_err(|e| anyhow!("template compile failed: {e}"))?;
    tmpl.render(ctx)
        .map_err(|e| anyhow!("template render failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn strings_are_rendered() {
        let ctx = json!({ "event": { "entity_id": "job-1" } });
        let input = json!({ "msg": "hi {{ event.entity_id }}", "n": 42 });
        let out = render_value(&input, &ctx).unwrap();
        assert_eq!(out["msg"], "hi job-1");
        assert_eq!(out["n"], 42);
    }

    #[test]
    fn non_template_strings_pass_through() {
        let ctx = json!({});
        let out = render_value(&json!("plain text"), &ctx).unwrap();
        assert_eq!(out, json!("plain text"));
    }

    #[test]
    fn arrays_render_recursively() {
        let ctx = json!({ "x": "A" });
        let out = render_value(&json!(["{{ x }}", ["{{ x }}"]]), &ctx).unwrap();
        assert_eq!(out, json!(["A", ["A"]]));
    }

    #[test]
    fn when_truthy() {
        let ctx = json!({ "n": 3 });
        assert!(evaluate_when("n > 1", &ctx).unwrap());
    }

    #[test]
    fn when_falsy() {
        let ctx = json!({ "n": 0 });
        assert!(!evaluate_when("n > 1", &ctx).unwrap());
    }

    #[test]
    fn numeric_context_is_a_number_not_a_map() {
        // serde_json's `arbitrary_precision` makes `Value::from_serialize`
        // emit numbers as `{"$serde_json::private::Number": "..."}` maps, which
        // broke every numeric comparison. Guard the conversion directly.
        let ctx = json!({ "n": 0, "m": 5 });
        assert!(evaluate_when("m > n", &ctx).unwrap());
        assert!(!evaluate_when("n > m", &ctx).unwrap());
        assert_eq!(render_string("{{ n }}-{{ m }}", &ctx).unwrap(), "0-5");
    }
}
