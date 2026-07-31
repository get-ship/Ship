//! Scenario 2 — typed-item validation, adversarial.
//!
//! Master property 2: `validate_item` accepts exactly the items the schema
//! permits and rejects everything else (wrong type, missing required, extra
//! field) — no coercion surprises. Pure unit-level: no event bus, no DB.

use serde_json::json;
use ship_collection::schema::{parse_item_data, parse_schema_doc, validate_item};
use ship_collection::types::SchemaDoc;

fn schema(fields: serde_json::Value) -> SchemaDoc {
    parse_schema_doc(
        &serde_json::to_vec(&json!({ "id": "adversarial", "fields": fields })).unwrap(),
    )
    .unwrap()
}

fn all_fields_schema() -> SchemaDoc {
    schema(json!({
        "s": { "type": "string", "required": true },
        "n": { "type": "number", "required": true },
        "b": { "type": "boolean" },
        "e": { "type": "enum", "values": ["a", "b", "c"] },
        "arr": { "type": "array" },
        "obj": { "type": "object" },
    }))
}

// ── wrong types ────────────────────────────────────────────────────────────

#[test]
fn number_field_rejects_string() {
    let s = all_fields_schema();
    let item = json!({ "s": "x", "n": "not a number", "b": true });
    assert!(validate_item(&s, &item).is_err());
}

#[test]
fn string_field_rejects_number() {
    let s = all_fields_schema();
    let item = json!({ "s": 42, "n": 1.0, "b": true });
    assert!(validate_item(&s, &item).is_err());
}

#[test]
fn boolean_field_rejects_string_true() {
    let s = all_fields_schema();
    let item = json!({ "s": "x", "n": 1.0, "b": "true" });
    assert!(validate_item(&s, &item).is_err());
}

#[test]
fn boolean_field_rejects_number_zero_or_one() {
    let s = all_fields_schema();
    for v in [json!(0), json!(1)] {
        let item = json!({ "s": "x", "n": 1.0, "b": v });
        assert!(
            validate_item(&s, &item).is_err(),
            "b={v:?} must be rejected, not coerced"
        );
    }
}

#[test]
fn array_field_rejects_object() {
    let s = all_fields_schema();
    let item = json!({ "s": "x", "n": 1.0, "arr": { "not": "an array" } });
    assert!(validate_item(&s, &item).is_err());
}

#[test]
fn object_field_rejects_array() {
    let s = all_fields_schema();
    let item = json!({ "s": "x", "n": 1.0, "obj": [1, 2, 3] });
    assert!(validate_item(&s, &item).is_err());
}

#[test]
fn enum_field_rejects_value_outside_set() {
    let s = all_fields_schema();
    let item = json!({ "s": "x", "n": 1.0, "e": "not-in-set" });
    assert!(validate_item(&s, &item).is_err());
}

#[test]
fn enum_field_rejects_non_string() {
    let s = all_fields_schema();
    let item = json!({ "s": "x", "n": 1.0, "e": 1 });
    assert!(validate_item(&s, &item).is_err());
}

#[test]
fn enum_field_is_case_sensitive() {
    // "A" is not in ["a","b","c"] — an accidental case-insensitive match
    // would be a coercion surprise.
    let s = all_fields_schema();
    let item = json!({ "s": "x", "n": 1.0, "e": "A" });
    assert!(
        validate_item(&s, &item).is_err(),
        "enum match must be case-sensitive"
    );
}

// ── nulls in required fields ──────────────────────────────────────────────

#[test]
fn required_field_rejects_explicit_null() {
    let s = all_fields_schema();
    let item = json!({ "s": null, "n": 1.0 });
    assert!(validate_item(&s, &item).is_err());
}

#[test]
fn required_field_rejects_missing() {
    let s = all_fields_schema();
    let item = json!({ "n": 1.0 });
    assert!(validate_item(&s, &item).is_err());
}

#[test]
fn optional_field_accepts_null() {
    let s = all_fields_schema();
    let item = json!({ "s": "x", "n": 1.0, "b": null });
    assert!(
        validate_item(&s, &item).is_ok(),
        "optional field explicit null should be accepted"
    );
}

#[test]
fn optional_field_accepts_absence() {
    let s = all_fields_schema();
    let item = json!({ "s": "x", "n": 1.0 });
    assert!(validate_item(&s, &item).is_ok());
}

// ── extra / unknown fields ────────────────────────────────────────────────

#[test]
fn unknown_field_is_rejected() {
    let s = all_fields_schema();
    let item = json!({ "s": "x", "n": 1.0, "not_a_field": "surprise" });
    assert!(
        validate_item(&s, &item).is_err(),
        "unknown fields must be rejected, not silently dropped"
    );
}

#[test]
fn unknown_field_name_matching_reserved_sql_keyword_is_rejected_as_unknown() {
    let s = all_fields_schema();
    let item = json!({ "s": "x", "n": 1.0, "SELECT": "1" });
    assert!(validate_item(&s, &item).is_err());
}

// ── boundary values ────────────────────────────────────────────────────────

#[test]
fn number_field_accepts_extremes() {
    let s = all_fields_schema();
    for v in [
        json!(f64::MAX),
        json!(-f64::MAX),
        json!(0),
        json!(-0.0),
        json!(1e308),
    ] {
        let item = json!({ "s": "x", "n": v });
        assert!(
            validate_item(&s, &item).is_ok(),
            "n={v:?} should validate as a number"
        );
    }
}

#[test]
fn number_field_rejects_nan_and_infinity_as_unrepresentable_json() {
    // serde_json cannot represent NaN/Infinity at all — confirms the JSON
    // boundary itself refuses them before validate_item ever sees them.
    let nan_json = "{\"s\":\"x\",\"n\":NaN}";
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(nan_json);
    assert!(
        parsed.is_err(),
        "NaN is not valid JSON — must fail at parse, not at validate_item"
    );
}

#[test]
fn string_field_accepts_empty_string() {
    let s = all_fields_schema();
    let item = json!({ "s": "", "n": 1.0 });
    assert!(
        validate_item(&s, &item).is_ok(),
        "empty string is a valid string"
    );
}

#[test]
fn string_field_accepts_very_long_string() {
    let s = all_fields_schema();
    let huge = "x".repeat(10_000_000);
    let item = json!({ "s": huge, "n": 1.0 });
    assert!(
        validate_item(&s, &item).is_ok(),
        "no documented max-length bound on string fields"
    );
}

/// FINDING (fixed): a deeply nested JSON value (5000 levels) reaching item
/// data used to stack-overflow and SIGABRT the whole process — not a
/// catchable `Err`. `validate_item`'s Object/Array checks are O(1)
/// (`value.is_object()` / `value.is_array()`, no recursive walk), so the
/// crash was never in `validate_item` itself; it was `serde_json::Value`'s
/// recursive `Drop` (confirmed by removing the `validate_item` call — the
/// overflow still happened when the nested `Value` merely went out of
/// scope), and `serde_json::from_slice`'s own recursive-descent parse is
/// equally unbounded. The real ingress is `CollectionService::create_item` /
/// `update_item`, which call `serde_json::from_slice::<Value>(data_json)` on
/// caller-controlled bytes — a single malicious `data_json` payload could
/// crash the daemon before `validate_item` ever ran. The fix is
/// `schema::parse_item_data`, which bounds nesting depth on the raw bytes
/// (iteratively, no recursion) BEFORE `serde_json::from_slice` is called, so
/// an over-depth payload is rejected before a dangerous `Value` tree is ever
/// constructed. `create_item`/`update_item` now route through it.
///
/// This test builds the malicious payload as raw bytes (never materializing
/// an in-memory deeply-nested `Value`, which is itself a separate hazard
/// unrelated to this fix — see `deeply_nested_and_chain_does_not_panic` in
/// `query_contract_torture.rs`) and drives it through the real ingress
/// (`parse_item_data`), matching how deep item data actually reaches the
/// daemon over the wire.
#[test]
fn deeply_nested_object_field_does_not_stack_overflow() {
    // Build a 5000-deep nested JSON object as raw bytes for the `obj` field.
    let mut nested = String::new();
    for _ in 0..5000 {
        nested.push_str("{\"nested\":");
    }
    nested.push('1');
    for _ in 0..5000 {
        nested.push('}');
    }
    let data_json = format!(r#"{{"s":"x","n":1.0,"obj":{nested}}}"#);

    // Must be a clean rejection, not a panic/crash — matches the real
    // create_item/update_item ingress path.
    let result = parse_item_data(data_json.as_bytes());
    assert!(
        result.is_err(),
        "expected rejection for over-depth item data, got {result:?}"
    );
}

#[test]
fn huge_array_field_validates_without_panicking() {
    let s = all_fields_schema();
    let big_array: Vec<i64> = (0..200_000).collect();
    let item = json!({ "s": "x", "n": 1.0, "arr": big_array });
    assert!(validate_item(&s, &item).is_ok());
}

// ── data root shape ─────────────────────────────────────────────────────

#[test]
fn non_object_data_root_is_rejected() {
    let s = all_fields_schema();
    for bad_root in [
        json!([1, 2, 3]),
        json!("a string"),
        json!(42),
        json!(null),
        json!(true),
    ] {
        assert!(
            validate_item(&s, &bad_root).is_err(),
            "root={bad_root:?} must be rejected — item data must be an object"
        );
    }
}

// ── multiple simultaneous violations ──────────────────────────────────────

#[test]
fn multiple_violations_all_reported_and_none_silently_coerced() {
    let s = all_fields_schema();
    // Missing required `s`, wrong type for `n`, unknown field `zzz`.
    let item = json!({ "n": "wrong-type", "zzz": 1 });
    let err = validate_item(&s, &item).unwrap_err().to_string();
    assert!(
        err.contains('s'),
        "must mention missing required field: {err}"
    );
    assert!(err.contains("zzz"), "must mention unknown field: {err}");
}

// ── boundary of schema definition itself (validate_schema_doc) ───────────

#[test]
fn enum_field_without_values_is_rejected_at_schema_definition_time() {
    let bad = serde_json::to_vec(&json!({
        "id": "badenum",
        "fields": { "e": { "type": "enum" } }
    }))
    .unwrap();
    assert!(
        parse_schema_doc(&bad).is_err(),
        "enum field with no 'values' must be rejected"
    );
}

#[test]
fn schema_with_no_fields_is_rejected() {
    let bad = serde_json::to_vec(&json!({ "id": "empty", "fields": {} })).unwrap();
    assert!(parse_schema_doc(&bad).is_err());
}

#[test]
fn schema_missing_id_is_rejected() {
    let bad =
        serde_json::to_vec(&json!({ "id": "", "fields": { "a": { "type": "string" } } })).unwrap();
    assert!(parse_schema_doc(&bad).is_err());
}
