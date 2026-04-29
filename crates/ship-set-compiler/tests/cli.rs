//! CLI smoke tests. The binary must compile a fixture file (or
//! directory) into the requested output directory and surface the
//! diagnostic on bad input.

use std::path::PathBuf;
use std::process::Command;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn binary_path() -> PathBuf {
    let mut p = manifest_dir();
    while !p.join("target").exists() {
        let parent = p.parent().expect("workspace root");
        p = parent.to_path_buf();
    }
    let exe = if cfg!(windows) {
        "ship-set-compiler.exe"
    } else {
        "ship-set-compiler"
    };
    p.join("target").join("debug").join(exe)
}

#[test]
fn cli_builds_a_single_fixture() {
    let bin = binary_path();
    if !bin.exists() {
        eprintln!("binary not built; run `cargo build -p ship-set-compiler`");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let input = manifest_dir()
        .join("tests/fixtures/hello.html.set");
    let out = tmp.path().to_path_buf();
    let status = Command::new(&bin)
        .args(["build"])
        .arg(&input)
        .args(["--out"])
        .arg(&out)
        .status()
        .expect("invoke ship-set-compiler");
    assert!(status.success(), "CLI failed");
    let js = out.join("hello.js");
    assert!(js.exists(), "expected {} to exist", js.display());
    let body = std::fs::read_to_string(js).unwrap();
    assert!(body.contains("export default view"));
}

#[test]
fn cli_renders_diagnostic_with_path() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let bad = tmp.path().join("bad.html.set");
    std::fs::write(&bad, r#"<state name="x" value={a = 1} />"#).unwrap();
    let out = tmp.path().join("out");
    let output = Command::new(&bin)
        .args(["build"])
        .arg(&bad)
        .args(["--out"])
        .arg(&out)
        .output()
        .expect("invoke");
    assert!(!output.status.success(), "expected failure");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("bad.html.set"),
        "stderr should mention the source path: {stderr}"
    );
    assert!(
        stderr.contains("line 1"),
        "stderr should mention a line number: {stderr}"
    );
    assert!(
        stderr.contains("assignments"),
        "stderr should describe the error: {stderr}"
    );
}
