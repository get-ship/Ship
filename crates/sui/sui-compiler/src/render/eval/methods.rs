//! Allowlisted method-call dispatch. Anything outside this set returns
//! `EvalError::NonEvaluable` and the walker falls back to D-1's
//! marker-only output.

use serde_json::Value;

use super::EvalError;
use super::coerce::{num_to_value, strict_eq, to_number, to_string};
use super::exec::execute;
use super::parser::Node;

type Scope = std::collections::HashMap<String, Value>;

pub(super) fn dispatch(
    obj: &Value,
    prop: &str,
    args: &[Node],
    scope: &Scope,
) -> Result<Value, EvalError> {
    match prop {
        "toString" if args.is_empty() => Ok(Value::String(to_string(obj))),
        "slice" => slice(obj, args, scope),
        "includes" if args.len() == 1 => {
            let needle = execute(&args[0], scope)?;
            includes(obj, &needle)
        }
        "keys" if args.is_empty() => keys(obj),
        "values" if args.is_empty() => values(obj),
        "entries" if args.is_empty() => entries(obj),
        "map" | "filter" => Err(EvalError::NonEvaluable(format!(
            ".{prop}() callback is not evaluable"
        ))),
        _ => Err(EvalError::NonEvaluable(format!(
            "method .{prop}() is not in the allowlist"
        ))),
    }
}

fn slice(obj: &Value, args: &[Node], scope: &Scope) -> Result<Value, EvalError> {
    if args.is_empty() || args.len() > 2 {
        return Err(EvalError::NonEvaluable(
            "slice expects 1 or 2 arguments".into(),
        ));
    }
    let start = to_number(&execute(&args[0], scope)?) as isize;
    match obj {
        Value::Array(arr) => {
            let len = arr.len() as isize;
            let end = if args.len() == 2 {
                to_number(&execute(&args[1], scope)?) as isize
            } else {
                len
            };
            let (s, e) = clamp_slice(start, end, len);
            Ok(Value::Array(arr[s..e].to_vec()))
        }
        Value::String(s) => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len() as isize;
            let end = if args.len() == 2 {
                to_number(&execute(&args[1], scope)?) as isize
            } else {
                len
            };
            let (a, b) = clamp_slice(start, end, len);
            Ok(Value::String(chars[a..b].iter().collect()))
        }
        _ => Err(EvalError::NonEvaluable("slice on non-array/string".into())),
    }
}

fn clamp_slice(start: isize, end: isize, len: isize) -> (usize, usize) {
    let norm = |i: isize| -> isize {
        if i < 0 { (len + i).max(0) } else { i.min(len) }
    };
    let s = norm(start);
    let e = norm(end);
    (s as usize, e.max(s) as usize)
}

fn includes(obj: &Value, needle: &Value) -> Result<Value, EvalError> {
    match (obj, needle) {
        (Value::Array(arr), n) => Ok(Value::Bool(arr.iter().any(|v| strict_eq(v, n)))),
        (Value::String(s), Value::String(n)) => Ok(Value::Bool(s.contains(n.as_str()))),
        _ => Err(EvalError::NonEvaluable(
            "includes on unsupported type".into(),
        )),
    }
}

fn keys(obj: &Value) -> Result<Value, EvalError> {
    match obj {
        Value::Object(map) => Ok(Value::Array(
            map.keys().map(|k| Value::String(k.clone())).collect(),
        )),
        Value::Array(arr) => Ok(Value::Array(
            (0..arr.len()).map(|i| num_to_value(i as f64)).collect(),
        )),
        _ => Err(EvalError::NonEvaluable(".keys() on unsupported type".into())),
    }
}

fn values(obj: &Value) -> Result<Value, EvalError> {
    match obj {
        Value::Object(map) => Ok(Value::Array(map.values().cloned().collect())),
        Value::Array(arr) => Ok(Value::Array(arr.clone())),
        _ => Err(EvalError::NonEvaluable(
            ".values() on unsupported type".into(),
        )),
    }
}

fn entries(obj: &Value) -> Result<Value, EvalError> {
    match obj {
        Value::Object(map) => Ok(Value::Array(
            map.iter()
                .map(|(k, v)| Value::Array(vec![Value::String(k.clone()), v.clone()]))
                .collect(),
        )),
        Value::Array(arr) => Ok(Value::Array(
            arr.iter()
                .enumerate()
                .map(|(i, v)| Value::Array(vec![num_to_value(i as f64), v.clone()]))
                .collect(),
        )),
        _ => Err(EvalError::NonEvaluable(
            ".entries() on unsupported type".into(),
        )),
    }
}
