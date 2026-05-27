//! sui-compiler integration for the Rust bundler (#163).
//!
//! Compiles a `.sui` source to `.js` on disk via the sui-compiler binary. This
//! is invoked inline from the Rolldown plugin's `load` hook
//! (`rolldown_bundle.rs`) — Rolldown resolves the module graph and calls back
//! here to materialize each `.sui` as JS, so there is no separate bundle phase.
//! Runs `sui-compiler build <file> --out <dir> [--component-name <stem>]` and
//! writes to a caller-owned dir.

use std::path::{Path, PathBuf};
use std::process::Command;

use ship_substrate::ShipHome;

/// A `.sui` compiles as a custom-element (`component`) or a view. Component
/// mode passes `--component-name` so the tag is `customElements.define()`d;
/// view mode emits a route module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    Component,
    View,
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("sui-compiler spawn failed: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("sui-compiler exit {code} for {file}:\n{stderr}")]
    Failed {
        code: i32,
        file: PathBuf,
        stderr: String,
        line: Option<u32>,
        col: Option<u32>,
    },
    #[error("sui-compiler reported success but emitted no output at {0}")]
    NoOutput(PathBuf),
}

#[derive(Debug, Clone)]
pub struct CompileResult {
    /// The compiled `<stem>.js` inside `out_dir`.
    pub js_path: PathBuf,
    /// `<stem>.custom-elements.json` when the compiler emitted one (component
    /// mode does). Feeds catalog refresh.
    pub cem_path: Option<PathBuf>,
}

/// Locate the sui-compiler binary: installed runtime copy first, then a dev
/// fallback to the freshest `target/{release,debug}/sui-compiler` — one
/// discovery path, shared with toolchain install.
pub fn locate_sui_compiler(home: &ShipHome) -> Option<PathBuf> {
    let installed = home.runtime_sui_compiler();
    if installed.exists() {
        return Some(installed);
    }
    crate::toolchain::install::walk_for_target_binary("sui-compiler").ok()
}

/// Compile one `.sui` to `<out_dir>/<stem>.js`. The emitted module imports its
/// controller by relative path + the `@ship/*` singletons, both resolvable by
/// the esbuild phase.
pub fn compile_sui(
    compiler_bin: &Path,
    sui_file: &Path,
    out_dir: &Path,
    mode: CompileMode,
    stem: &str,
) -> Result<CompileResult, CompileError> {
    let mut cmd = Command::new(compiler_bin);
    cmd.arg("build")
        .arg(sui_file)
        .arg("--out")
        .arg(out_dir)
        .arg("--no-dts");
    if mode == CompileMode::Component {
        cmd.arg("--component-name").arg(stem);
    }

    let out = cmd.output().map_err(CompileError::Spawn)?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        let (line, col) = parse_loc(&stderr);
        return Err(CompileError::Failed {
            code: out.status.code().unwrap_or(-1),
            file: sui_file.to_path_buf(),
            stderr,
            line,
            col,
        });
    }

    let js_path = out_dir.join(format!("{stem}.js"));
    if !js_path.exists() {
        return Err(CompileError::NoOutput(js_path));
    }
    let cem = out_dir.join(format!("{stem}.custom-elements.json"));
    Ok(CompileResult {
        js_path,
        cem_path: cem.exists().then_some(cem),
    })
}

/// Extract the first `line:col` from compiler stderr (matches the worker's
/// `/(\d+):(\d+)/` heuristic) for `compile_error` event reporting.
fn parse_loc(stderr: &str) -> (Option<u32>, Option<u32>) {
    let bytes = stderr.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let line_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b':' {
            let col_start = i + 1;
            let mut j = col_start;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > col_start {
                let line = stderr[line_start..i].parse().ok();
                let col = stderr[col_start..j].parse().ok();
                if line.is_some() && col.is_some() {
                    return (line, col);
                }
            }
        }
    }
    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn compiler_bin() -> PathBuf {
        crate::toolchain::install::walk_for_target_binary("sui-compiler")
            .expect("sui-compiler must be built for tests — `cargo build -p sui-compiler`")
    }

    #[test]
    fn compiles_view_to_js() {
        let bin = compiler_bin();
        let src = tempdir().unwrap();
        let out = tempdir().unwrap();
        let sui = src.path().join("page.sui");
        std::fs::write(&sui, "<div class=\"hello\">hi {1 + 1}</div>\n").unwrap();

        let res = compile_sui(&bin, &sui, out.path(), CompileMode::View, "page")
            .expect("view should compile");

        assert_eq!(res.js_path, out.path().join("page.js"));
        assert!(res.js_path.exists());
        assert!(std::fs::read_to_string(&res.js_path).unwrap().len() > 0);
    }

    #[test]
    fn surfaces_compile_errors_with_location() {
        let bin = compiler_bin();
        let src = tempdir().unwrap();
        let out = tempdir().unwrap();
        let sui = src.path().join("broken.sui");
        // Unclosed control-flow block — a grammar error the compiler rejects.
        std::fs::write(&sui, "<div>{#if cond()}oops</div>\n").unwrap();

        let err = compile_sui(&bin, &sui, out.path(), CompileMode::View, "broken")
            .unwrap_err();
        assert!(matches!(err, CompileError::Failed { .. }));
    }
}
