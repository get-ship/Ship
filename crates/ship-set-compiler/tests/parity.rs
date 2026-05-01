//! Parity tests. For each fixture, compile via the AOT compiler and
//! confirm the emitted JS is parseable by Node and structurally
//! consistent with re-running the same compile.
//!
//! The full DOM-level parity check (compile + interpret side-by-side
//! in jsdom and compare serialized HTML) lives in
//! `tests/parity_runner.mjs`. It is gated on `RUN_PARITY=1` because
//! it requires the workspace to be installed (`pnpm install`) so the
//! TypeScript sources of `@ship-set/dom` and `@ship-set/runtime`
//! resolve at runtime.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use ship_set_compiler::{compile, CompileMode, CompileOpts};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn compile_fixture(name: &str, mode: CompileMode, setup: Option<&str>) -> String {
    let path = fixtures_dir().join(format!("{name}.html.set"));
    let source = fs::read_to_string(&path).unwrap();
    let out = compile(
        &source,
        CompileOpts {
            setup_url: setup.map(String::from),
            mode,
            source_path: Some(path),
            styles: None,
        },
    )
    .expect("compile");
    out.js
}

fn write_mjs(name: &str, js: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ship-set-compiler-parity-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let p = dir.join(format!("{name}.mjs"));
    fs::write(&p, js).unwrap();
    p
}

fn node_check(path: &PathBuf) {
    let status = Command::new("node")
        .arg("--check")
        .arg(path)
        .status();
    let status = match status {
        Ok(s) => s,
        Err(e) => {
            eprintln!("`node` not found ({e}); skipping parity syntactic check");
            return;
        }
    };
    assert!(
        status.success(),
        "compiled JS at {} did not parse with `node --check`",
        path.display()
    );
}

#[test]
fn hello_compiles_to_valid_js() {
    let js = compile_fixture("hello", CompileMode::View, None);
    let p = write_mjs("hello", &js);
    node_check(&p);
}

#[test]
fn counter_compiles_to_valid_js() {
    let js = compile_fixture("counter", CompileMode::View, Some("./counter.ts"));
    let p = write_mjs("counter", &js);
    node_check(&p);
}

#[test]
fn agents_compiles_to_valid_js() {
    let js = compile_fixture("agents", CompileMode::View, Some("./agents.ts"));
    let p = write_mjs("agents", &js);
    node_check(&p);
}

#[test]
fn ship_button_compiles_to_valid_js() {
    let js = compile_fixture("ship-button", CompileMode::Component, None);
    let p = write_mjs("ship-button", &js);
    node_check(&p);
}

/// Compiling the same source twice must produce byte-identical JS.
/// This is the structural parity gate: the parse + emit pipeline is
/// a pure function.
#[test]
fn compile_is_deterministic() {
    for (name, mode, setup) in [
        ("hello", CompileMode::View, None),
        ("counter", CompileMode::View, Some("./counter.ts")),
        ("agents", CompileMode::View, Some("./agents.ts")),
        ("ship-button", CompileMode::Component, None),
    ] {
        let a = compile_fixture(name, mode, setup);
        let b = compile_fixture(name, mode, setup);
        assert_eq!(a, b, "compiler not deterministic for {name}");
    }
}

/// Full hydrate-and-compare parity. Opt-in: requires the workspace's
/// JS dependencies to be installed (`pnpm install`) so that
/// `@ship-set/dom` and `@ship-set/runtime` resolve.
#[test]
fn full_parity_with_interpreter() {
    if std::env::var("RUN_PARITY").is_err() {
        eprintln!(
            "skipping full parity (set RUN_PARITY=1 with workspace deps installed to enable)"
        );
        return;
    }
    let runner = fixtures_dir()
        .parent()
        .unwrap()
        .join("parity_runner.mjs");
    assert!(
        runner.exists(),
        "parity_runner.mjs missing at {}",
        runner.display()
    );
    let status = Command::new("node")
        .arg(&runner)
        .status()
        .expect("invoke node");
    assert!(status.success(), "parity_runner failed");
}
