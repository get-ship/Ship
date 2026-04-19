//! Minijinja rendering for hook action values.

use anyhow::{Context, Result, anyhow};
use minijinja::{Environment, Value as JinjaValue};
use serde_json::Value;

/// Render every string leaf of `input` through minijinja with `context`.
/// Non-string leaves pass through unchanged.
pub fn render_value(input: &Value, context: &Value) -> Result<Value> {
    let env = Environment::new();
    let jinja_ctx = JinjaValue::from_serialize(context);
    render_with(&env, input, &jinja_ctx)
}

/// Render a single template string.
pub fn render_string(template: &str, context: &Value) -> Result<String> {
    let env = Environment::new();
    let jinja_ctx = JinjaValue::from_serialize(context);
    render_string_with(&env, template, &jinja_ctx)
}

/// Evaluate a minijinja expression as a boolean. Empty / missing = false.
pub fn evaluate_when(expr: &str, context: &Value) -> Result<bool> {
    let rendered = render_string(&format!("{{% if {expr} %}}1{{% else %}}0{{% endif %}}"), context)
        .with_context(|| format!("evaluating when: {expr}"))?;
    Ok(rendered.trim() == "1")
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
}
