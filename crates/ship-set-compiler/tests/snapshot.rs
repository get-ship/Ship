//! Snapshot tests. For each fixture, compile and compare against a
//! checked-in `<fixture>.expected.js`. Set `UPDATE_SNAPSHOTS=1` in
//! the environment to overwrite the expectations.

use std::fs;
use std::path::{Path, PathBuf};

use ship_set_compiler::{compile, CompileMode, CompileOpts};

const FIXTURES: &[(&str, Option<&str>, CompileMode)] = &[
    ("hello", None, CompileMode::View),
    ("counter", Some("./counter.ts"), CompileMode::View),
    ("agents", Some("./agents.ts"), CompileMode::View),
    ("ship-button", None, CompileMode::Component),
];

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn compile_fixture(name: &str, setup_url: Option<&str>, mode: CompileMode) -> String {
    let path = fixtures_dir().join(format!("{name}.html.set"));
    let source = fs::read_to_string(&path).expect("read fixture");
    let opts = CompileOpts {
        setup_url: setup_url.map(|s| s.to_string()),
        mode,
        source_path: Some(path),
    };
    let out = compile(&source, opts).expect("compile");
    out.js
}

fn assert_snapshot(name: &str, actual: &str, expected_path: &Path) {
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        fs::write(expected_path, actual).expect("write snapshot");
        return;
    }
    let expected = match fs::read_to_string(expected_path) {
        Ok(s) => s,
        Err(_) => {
            // First run — write the file but fail so the user notices.
            fs::write(expected_path, actual).expect("write snapshot");
            panic!(
                "snapshot {} did not exist; wrote it now — re-run to confirm",
                expected_path.display()
            );
        }
    };
    assert_eq!(
        expected, actual,
        "snapshot mismatch for {name}. Set UPDATE_SNAPSHOTS=1 to refresh."
    );
}

#[test]
fn hello_snapshot() {
    let (name, setup, mode) = FIXTURES[0];
    let js = compile_fixture(name, setup, mode);
    let path = fixtures_dir().join(format!("{name}.expected.js"));
    assert_snapshot(name, &js, &path);
}

#[test]
fn counter_snapshot() {
    let (name, setup, mode) = FIXTURES[1];
    let js = compile_fixture(name, setup, mode);
    let path = fixtures_dir().join(format!("{name}.expected.js"));
    assert_snapshot(name, &js, &path);
}

#[test]
fn agents_snapshot() {
    let (name, setup, mode) = FIXTURES[2];
    let js = compile_fixture(name, setup, mode);
    let path = fixtures_dir().join(format!("{name}.expected.js"));
    assert_snapshot(name, &js, &path);
}

#[test]
fn ship_button_snapshot() {
    let (name, setup, mode) = FIXTURES[3];
    let js = compile_fixture(name, setup, mode);
    let path = fixtures_dir().join(format!("{name}.expected.js"));
    assert_snapshot(name, &js, &path);
}

#[test]
fn ship_button_emits_cem() {
    let path = fixtures_dir().join("ship-button.html.set");
    let source = fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
    };
    let out = compile(&source, opts).expect("compile");
    let cem = out.cem.expect("CEM emitted for component-mode");
    let cem_str = serde_json::to_string_pretty(&cem).unwrap();
    let expected_path = fixtures_dir().join("ship-button.expected.cem.json");
    assert_snapshot("ship-button.cem", &cem_str, &expected_path);
}

#[test]
fn schema_emitted_for_each_fixture() {
    for (name, _, mode) in FIXTURES {
        let path = fixtures_dir().join(format!("{name}.html.set"));
        let source = fs::read_to_string(&path).unwrap();
        let opts = CompileOpts {
            setup_url: None,
            mode: *mode,
            source_path: Some(path),
        };
        let out = compile(&source, opts).expect("compile");
        let schema = out.schema.expect("schema emitted");
        let s = serde_json::to_string_pretty(&schema).unwrap();
        let expected = fixtures_dir().join(format!("{name}.expected.schema.json"));
        assert_snapshot(&format!("{name}.schema"), &s, &expected);
    }
}
