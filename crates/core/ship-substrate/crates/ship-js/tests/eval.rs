use ship_js::{eval, eval_global, JsError};

#[test]
fn eval_returns_assigned_json() {
    let bundle = "globalThis.__ship_result = {a: 1, b: [2, 3]};";
    let val = eval(bundle).unwrap();
    assert_eq!(val["a"], 1);
    assert_eq!(val["b"], serde_json::json!([2, 3]));
}

#[test]
fn eval_handles_string_number_bool_null() {
    let cases: &[(&str, serde_json::Value)] = &[
        (
            "globalThis.__ship_result = 'hello';",
            serde_json::json!("hello"),
        ),
        (
            "globalThis.__ship_result = 42;",
            serde_json::json!(42),
        ),
        (
            "globalThis.__ship_result = true;",
            serde_json::json!(true),
        ),
        (
            "globalThis.__ship_result = null;",
            serde_json::json!(null),
        ),
    ];
    for (bundle, expected) in cases {
        let val = eval(bundle)
            .unwrap_or_else(|e| panic!("failed for bundle '{bundle}': {e}"));
        assert_eq!(&val, expected);
    }
}

#[test]
fn eval_global_reads_named_global() {
    let bundle = "globalThis.__custom = {found: true};";
    let val = eval_global(bundle, "__custom").unwrap();
    assert_eq!(val["found"], true);
}

#[test]
fn eval_missing_result_is_error() {
    let bundle = "var x = 1;";
    let err = eval(bundle).unwrap_err();
    assert!(matches!(err, JsError::MissingResult(_)));
}

#[test]
fn eval_syntax_error_is_error() {
    let bundle = "this is { not valid js !!!";
    let err = eval(bundle).unwrap_err();
    assert!(matches!(err, JsError::Eval(_)));
}

#[test]
fn eval_runs_pure_functional_js() {
    let bundle =
        "globalThis.__ship_result = [1,2,3].map(x => x * 2).reduce((a, b) => a + b);";
    let val = eval(bundle).unwrap();
    assert_eq!(val, serde_json::json!(12));
}
