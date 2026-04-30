//! Build the initial render scope from a parsed `Program`.
//!
//! Two outputs:
//! * `Scope` — `HashMap<name, Value>` consumed by the evaluator while
//!   walking the template.
//! * `init: Value` — the JSON object emitted into the
//!   `<script type="application/json" id="__set_init__">` island, with
//!   each `<state>` and `<data>` declaration's first-paint value.
//!
//! `<state>` initials come from parsing the `value=` source as JSON
//! first (covers `0`, `''`, `false`, `null`, `[]`, `{}`); on JSON
//! parse failure, the evaluator runs against an empty scope (covers
//! literal expressions like `[1, 2]` written with bare keys). On
//! evaluator failure the slot is `null` so the JSON island has a
//! consistent shape and the walker still has a binding to mark as
//! un-evaluable.
//!
//! `<data>` initials are always `{ "kind": "loading" }` envelopes —
//! D-2 is the static path; the daemon-cache integration is Phase F's
//! concern. `live` data and non-live data both render the same
//! envelope.
//!
//! A `route` placeholder with empty `params`/`query` is added so
//! expressions like `route.params.id` evaluate to `undefined` cleanly
//! instead of unbound errors.

use serde_json::{Map, Value, json};

use crate::ir::Program;

use super::eval::{Scope, eval};

/// Result of scope construction. The walker reads `scope`; the
/// renderer emits `init` into the JSON island.
pub struct InitialScope {
    pub scope: Scope,
    pub init: Value,
}

pub fn build(program: &Program) -> InitialScope {
    let mut scope: Scope = Scope::new();
    let mut init: Map<String, Value> = Map::new();

    scope.insert(
        "route".into(),
        json!({ "params": {}, "query": {} }),
    );

    for state in &program.state {
        let value = resolve_state_value(&state.value.source, &scope);
        scope.insert(state.name.clone(), value.clone());
        init.insert(state.name.clone(), value);
    }

    for data in &program.data {
        let envelope = json!({ "kind": "loading" });
        scope.insert(data.name.clone(), envelope.clone());
        init.insert(data.name.clone(), envelope);
    }

    InitialScope { scope, init: Value::Object(init) }
}

/// Two-stage resolution: try JSON first (the common cheap case), then
/// fall back to the renderable-subset evaluator. Bare expressions like
/// `[1, 2, 3]` aren't valid JSON (no quotes around keys / unquoted
/// values) only when objects, but `{}` is valid JSON; `[]` is too;
/// quoted strings via `'...'` are not — let the evaluator handle those.
fn resolve_state_value(source: &str, scope: &Scope) -> Value {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Value::Null;
    }
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        return v;
    }
    eval(trimmed, scope).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    fn parse_src(src: &str) -> Program {
        parse(src).expect("parse")
    }

    #[test]
    fn state_zero_becomes_integer() {
        let p = parse_src("<state name=\"n\" value=\"0\" />");
        let s = build(&p);
        assert_eq!(s.scope.get("n").unwrap(), &json!(0));
        assert_eq!(s.init.get("n").unwrap(), &json!(0));
    }

    #[test]
    fn state_string_literal_unwraps() {
        // `'hi'` isn't valid JSON; evaluator handles single-quoted strings.
        let p = parse_src("<state name=\"s\" value=\"'hi'\" />");
        let s = build(&p);
        assert_eq!(s.scope.get("s").unwrap(), &json!("hi"));
    }

    #[test]
    fn state_array_literal_evaluates() {
        // `value="[]"` → expression source is `[]`. The JSON path catches
        // this without going through the evaluator.
        let p = parse_src("<state name=\"xs\" value=\"[1, 2, 3]\" />");
        let s = build(&p);
        assert_eq!(s.scope.get("xs").unwrap(), &json!([1, 2, 3]));
    }

    #[test]
    fn data_emits_loading_envelope() {
        let p = parse_src("<data name=\"agents\" source=\"ship.agents.summaries()\" live />");
        let s = build(&p);
        assert_eq!(s.init.get("agents").unwrap(), &json!({"kind": "loading"}));
        assert_eq!(s.scope.get("agents").unwrap(), &json!({"kind": "loading"}));
    }

    #[test]
    fn route_placeholder_is_present() {
        let p = parse_src("<state name=\"x\" value=\"0\" />");
        let s = build(&p);
        let route = s.scope.get("route").expect("route in scope");
        assert_eq!(route, &json!({"params": {}, "query": {}}));
    }
}
