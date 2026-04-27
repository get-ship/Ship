//! `sail` — command-line entry to the Sail compiler.
//!
//! See `docs/sail/10-dev-tooling.md § CLI`.
//!
//! Verbs (v1):
//! - `sail build [path]` — compile every `.sail` under `path` to `dist/`
//! - `sail check [path]` — parse + report diagnostics; no emit
//! - `sail new <name>`   — scaffold a starter project
//!
//! `dev` (watch + recompile loop) lives in the daemon for the Ship use
//! case; standalone OSS users get it next.

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use sail_compiler::{CompileOpts, Severity, compile_source, compile_source_with, tag_for};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "sail", version, about = "Sail framework CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Compile every `.sail` under <path> into <out>/.
    Build {
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output directory for compiled artifacts.
        #[arg(short, long, default_value = "dist")]
        out: PathBuf,
    },
    /// Parse every `.sail` under <path> and report any errors.
    Check {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Scaffold a new Sail project directory.
    New {
        /// Project directory to create.
        name: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.cmd {
        Cmd::Build { path, out } => run_build(&path, &out),
        Cmd::Check { path } => run_check(&path),
        Cmd::New { name } => run_new(&name),
    };
    if let Err(e) = result {
        eprintln!("sail: error: {e:#}");
        std::process::exit(1);
    }
}

fn run_build(root: &Path, out: &Path) -> Result<()> {
    let files = collect_sail_files(root)?;
    if files.is_empty() {
        eprintln!("sail: no .sail files found under {}", root.display());
        return Ok(());
    }
    std::fs::create_dir_all(out).with_context(|| format!("create {}", out.display()))?;
    let mut count = 0usize;
    for path in &files {
        let tag = tag_for(path);
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?;
        let result =
            compile_source(&source, &tag).with_context(|| format!("compile {}", path.display()))?;
        write_artifact(out, &tag, "js", &result.js)?;
        write_artifact(out, &tag, "html", &result.ssr)?;
        write_artifact(out, &tag, "d.ts", &result.dts)?;
        write_artifact(out, &tag, "schema.json", &result.schema)?;
        count += 1;
    }
    eprintln!("sail: built {count} component(s) → {}/", out.display());
    Ok(())
}

fn run_check(root: &Path) -> Result<()> {
    let files = collect_sail_files(root)?;
    if files.is_empty() {
        eprintln!("sail: no .sail files found under {}", root.display());
        return Ok(());
    }
    let mut errors = 0usize;
    let opts = CompileOpts::default();
    for path in &files {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?;
        let tag = tag_for(path);
        match compile_source_with(&source, &tag, &opts) {
            Ok(result) => {
                for d in &result.diagnostics {
                    let label = match d.severity {
                        Severity::Error => { errors += 1; "error" }
                        Severity::Warning => "warning",
                        Severity::Info => "info",
                    };
                    eprintln!("{label} [{}]: {}: {}", d.code, path.display(), d.message);
                    if let Some(h) = &d.help {
                        eprintln!("    help: {h}");
                    }
                }
            }
            Err(e) => {
                eprintln!("error: {}: {e:#}", path.display());
                errors += 1;
            }
        }
    }
    if errors > 0 {
        bail!("{errors} file(s) failed to compile");
    }
    eprintln!("sail: {} file(s) ok", files.len());
    Ok(())
}

fn run_new(dir: &Path) -> Result<()> {
    if dir.exists() {
        bail!("{} already exists", dir.display());
    }
    std::fs::create_dir_all(dir.join("sail"))?;
    std::fs::write(dir.join("sail/app-counter.sail"), STARTER_COUNTER)?;
    std::fs::write(dir.join("README.md"), STARTER_README)?;
    eprintln!("sail: created {}", dir.display());
    eprintln!("    → cd {} && sail build", dir.display());
    Ok(())
}

fn collect_sail_files(root: &Path) -> Result<Vec<PathBuf>> {
    if root.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }
    let mut out: Vec<PathBuf> = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries =
            std::fs::read_dir(&dir).with_context(|| format!("read_dir {}", dir.display()))?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                // Skip dist/, node_modules/, .git/ and similar.
                let skip = p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|n| matches!(n, "dist" | "node_modules" | ".git" | "target"))
                    .unwrap_or(false);
                if !skip {
                    stack.push(p);
                }
            } else if p.extension().and_then(|s| s.to_str()) == Some("sail") {
                out.push(p);
            }
        }
    }
    out.sort();
    Ok(out)
}

fn write_artifact(out: &Path, tag: &str, ext: &str, body: &str) -> Result<()> {
    let path = out.join(format!("{tag}.{ext}"));
    std::fs::write(&path, body).with_context(|| format!("write {}", path.display()))
}

const STARTER_COUNTER: &str = r#"<script>
state count = 0
derived doubled = count() * 2
function inc() { count.set(count() + 1) }
</script>

<template>
<div class="counter">
  <p>Count: {count}</p>
  <p>Doubled: {doubled}</p>
  <button on:click={inc}>+1</button>
</div>
</template>

<style>
.counter { padding: 1rem; font-family: system-ui; }
button { padding: 0.5rem 1rem; }
</style>
"#;

const STARTER_README: &str = r#"# New Sail project

Run `sail build` to compile every `.sail` under this directory into `dist/`.

The starter component lives at `sail/app-counter.sail`. Each `.sail` file
becomes four artifacts in `dist/`: `<tag>.js`, `<tag>.html` (SSR),
`<tag>.d.ts`, and `<tag>.schema.json`.
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn collect_finds_only_sail_files() {
        let dir = tmp_dir();
        std::fs::write(dir.path().join("a.sail"), "<template><div/></template>").unwrap();
        std::fs::write(dir.path().join("b.txt"), "ignore").unwrap();
        std::fs::create_dir_all(dir.path().join("nested")).unwrap();
        std::fs::write(
            dir.path().join("nested/c.sail"),
            "<template><div/></template>",
        )
        .unwrap();
        let mut found = collect_sail_files(dir.path()).unwrap();
        found.sort();
        assert_eq!(found.len(), 2);
        assert!(found[0].ends_with("a.sail"));
        assert!(found[1].ends_with("c.sail"));
    }

    #[test]
    fn collect_skips_dist_and_node_modules() {
        let dir = tmp_dir();
        for skip in ["dist", "node_modules", ".git", "target"] {
            std::fs::create_dir_all(dir.path().join(skip)).unwrap();
            std::fs::write(
                dir.path().join(skip).join("x.sail"),
                "<template><div/></template>",
            )
            .unwrap();
        }
        std::fs::write(dir.path().join("ok.sail"), "<template><div/></template>").unwrap();
        let found = collect_sail_files(dir.path()).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("ok.sail"));
    }

    #[test]
    fn build_emits_four_artifacts_per_file() {
        let dir = tmp_dir();
        let src = dir.path().join("counter.sail");
        let mut f = std::fs::File::create(&src).unwrap();
        f.write_all(b"<script>state c = 0</script><template><span>{c}</span></template>")
            .unwrap();
        let out = dir.path().join("dist");
        run_build(dir.path(), &out).unwrap();
        assert!(out.join("sail-counter.js").exists());
        assert!(out.join("sail-counter.html").exists());
        assert!(out.join("sail-counter.d.ts").exists());
        assert!(out.join("sail-counter.schema.json").exists());
    }

    #[test]
    fn check_passes_clean_files_and_fails_broken() {
        let dir = tmp_dir();
        std::fs::write(dir.path().join("ok.sail"), "<template><span>ok</span></template>").unwrap();
        run_check(dir.path()).unwrap();

        std::fs::write(
            dir.path().join("bad.sail"),
            "<template><div></span></template>",
        )
        .unwrap();
        let err = run_check(dir.path()).unwrap_err();
        assert!(err.to_string().contains("failed to compile"));
    }

    #[test]
    fn new_scaffolds_starter() {
        let dir = tmp_dir();
        let proj = dir.path().join("myapp");
        run_new(&proj).unwrap();
        assert!(proj.join("sail/app-counter.sail").exists());
        assert!(proj.join("README.md").exists());
        // Re-running fails because it already exists.
        assert!(run_new(&proj).is_err());
    }
}
