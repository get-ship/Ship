//! Integration test for `sail check` exit-code behavior.
//!
//! Spawns the compiled `sail` binary against tempdir fixtures so the
//! end-to-end exit code (the part the set-port agent's verification
//! step reads) is exercised, not just the in-process logic.

use std::path::PathBuf;
use std::process::Command;

fn sail_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sail"))
}

fn write(dir: &std::path::Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn check_exits_zero_on_clean_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = write(
        dir.path(),
        "ok.set",
        "<template><span>ok</span></template>",
    );
    let out = Command::new(sail_bin())
        .arg("check")
        .arg(&path)
        .output()
        .expect("spawn sail check");
    assert!(
        out.status.success(),
        "expected exit 0, got {:?}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn check_exits_nonzero_on_listener_leak() {
    let dir = tempfile::tempdir().unwrap();
    let path = write(
        dir.path(),
        "leaky.set",
        "<script>mount { el.addEventListener(\"click\", h) }</script><template>x</template>",
    );
    let out = Command::new(sail_bin())
        .arg("check")
        .arg(&path)
        .output()
        .expect("spawn sail check");
    assert!(!out.status.success(), "expected non-zero exit");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("SAIL/E0501"),
        "expected SAIL/E0501 in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("ERROR"),
        "expected ERROR label in stderr, got:\n{stderr}"
    );
}

#[test]
fn check_severity_warning_still_gates_on_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = write(
        dir.path(),
        "leaky.set",
        "<script>mount { el.addEventListener(\"click\", h) }</script><template>x</template>",
    );
    let out = Command::new(sail_bin())
        .arg("check")
        .arg(&path)
        .arg("--severity")
        .arg("warning")
        .output()
        .expect("spawn sail check");
    assert!(!out.status.success(), "Error still gates under --severity warning");
}

#[test]
fn check_help_lists_severity_flag() {
    let out = Command::new(sail_bin())
        .arg("check")
        .arg("--help")
        .output()
        .expect("spawn sail check --help");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("--severity"), "expected --severity in help: {stdout}");
}
