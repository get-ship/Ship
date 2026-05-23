//! `sui-compiler` CLI. Compiles a single `.sui` file or a directory of
//! them, writing the emitted JS (and CEM/schema/meta/dts/ssr.html sidecars)
//! into a project-level mirror tree at `<repo-root>/.sui-out/<source-
//! relative-path>/`. Pass `--out <dir>` to override and emit every file to
//! a single directory (legacy behaviour). Errors render with file path,
//! line, and column.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

use sui_compiler::out_layout::resolve_out_dir;
use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

/// Hard caps on virtual memory and wall-clock time. A bug in the
/// compiler (or a pathological input) must never be able to OOM the
/// host: a single uncaught `Vec::with_capacity` once allocated 19+GB
/// before the kernel killed the process and took down the surrounding
/// VM. These bounds guarantee a clean death long before that.
///
/// Generous for any real input: a single `.sui` file compiles in
/// milliseconds with <100MB RSS. A directory of hundreds compiles in
/// seconds. Hitting either cap is a bug to file, not a knob to tune.
/// To override (e.g. unusually large project, deliberate stress run),
/// set `SUI_COMPILER_NO_LIMITS=1`.
const MEM_CAP_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GB virtual memory
const WALL_CAP_SECS: u64 = 60;

fn install_resource_bounds() {
    if std::env::var_os("SUI_COMPILER_NO_LIMITS").is_some() {
        return;
    }
    // SAFETY: setrlimit with valid resource id and rlimit struct is sound.
    unsafe {
        let lim = libc::rlimit { rlim_cur: MEM_CAP_BYTES as libc::rlim_t, rlim_max: MEM_CAP_BYTES as libc::rlim_t };
        libc::setrlimit(libc::RLIMIT_AS, &lim);
    }
    thread::Builder::new()
        .name("sui-compiler-watchdog".into())
        .spawn(|| {
            thread::sleep(Duration::from_secs(WALL_CAP_SECS));
            eprintln!("sui-compiler: aborted after {WALL_CAP_SECS}s wall-clock cap");
            process::abort();
        })
        .expect("spawn watchdog thread");
}

#[derive(Parser)]
#[command(name = "sui-compiler", about = "AOT compiler for .sui files")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile one file or every `.sui` under a directory.
    Build {
        /// Path to a `.sui` file or a directory.
        input: PathBuf,
        /// Output directory. When omitted, each file's outputs land in the
        /// project mirror tree at `<repo-root>/.sui-out/<source-rel>/`
        /// and the JS module is named `<stem>.compiled.js`. When supplied,
        /// every file is emitted into this single directory and the JS
        /// module is named `<stem>.js` (legacy behaviour).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Remove each source file's mirror directory (or `--out` directory)
        /// before emitting. Idempotent rebuilds.
        #[arg(long)]
        clean: bool,
        /// Override mode (otherwise auto-detected from --component-name).
        #[arg(long)]
        mode: Option<ModeArg>,
        /// Suppress emitting the `.sui.d.ts` declaration file.
        #[arg(long)]
        no_dts: bool,
        /// Suppress emitting the `.ssr.html` static SSR file (e.g. for island bundles).
        #[arg(long)]
        no_ssr: bool,
        /// Path to a CSS file whose contents are prepended as a `<style>`
        /// block. Only valid for component sources (requires --component-name).
        /// Requires `input` to be a single file (not a directory).
        #[arg(long)]
        styles: Option<PathBuf>,
        /// Custom element tag name for this component file. Setting this flag
        /// implies component mode — no `--mode component` needed. When omitted
        /// for a file in component mode, defaults to the filename stem.
        #[arg(long)]
        component_name: Option<String>,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum ModeArg {
    View,
    Component,
    Auto,
}

impl From<ModeArg> for CompileMode {
    fn from(m: ModeArg) -> Self {
        match m {
            ModeArg::View => CompileMode::View,
            ModeArg::Component => CompileMode::Component,
            ModeArg::Auto => CompileMode::Auto,
        }
    }
}

fn main() -> Result<()> {
    install_resource_bounds();
    let cli = Cli::parse();
    match cli.cmd {
        Command::Build { input, out, clean, mode, no_dts, styles, component_name, no_ssr } => {
            build(input, out, clean, mode, no_dts, styles, component_name, no_ssr)
        }
    }
}

fn build(
    input: PathBuf,
    out: Option<PathBuf>,
    clean: bool,
    mode: Option<ModeArg>,
    no_dts: bool,
    styles: Option<PathBuf>,
    component_name: Option<String>,
    no_ssr: bool,
) -> Result<()> {
    let mut targets = Vec::new();
    if input.is_dir() {
        if styles.is_some() {
            return Err(anyhow!(
                "--styles requires a single .sui input, not a directory"
            ));
        }
        collect_targets(&input, &mut targets)?;
    } else if input
        .file_name()
        .and_then(|f| f.to_str())
        .is_some_and(is_sui_source)
    {
        targets.push(input.clone());
    } else {
        return Err(anyhow!("not a .sui file or directory: {}", input.display()));
    }
    if targets.is_empty() {
        return Err(anyhow!("no .sui files found under {}", input.display()));
    }

    if clean {
        if let Some(out_dir) = out.as_deref() {
            // Single --out dir mode: clean it once before emit.
            if out_dir.exists() {
                fs::remove_dir_all(out_dir)
                    .with_context(|| format!("clean {}", out_dir.display()))?;
            }
        } else {
            // Mirror-tree mode: clean each unique target mirror directory.
            let mut cleaned: HashSet<PathBuf> = HashSet::new();
            for path in &targets {
                let mirror = resolve_out_dir(path);
                if cleaned.insert(mirror.clone()) && mirror.exists() {
                    fs::remove_dir_all(&mirror)
                        .with_context(|| format!("clean {}", mirror.display()))?;
                }
            }
        }
    }

    if let Some(out_dir) = out.as_deref() {
        fs::create_dir_all(out_dir)
            .with_context(|| format!("create out dir {}", out_dir.display()))?;
    }

    let styles_text = match styles.as_deref() {
        Some(p) => Some(
            fs::read_to_string(p)
                .with_context(|| format!("read --styles {}", p.display()))?,
        ),
        None => None,
    };
    // Component mode: tag name comes from --component-name when supplied,
    // otherwise defaults to the filename stem at compile-one time. Shadow
    // is always on for components — no opt-out flag.
    let count = targets.len();
    for path in targets {
        let component_meta = match (mode.clone(), component_name.clone()) {
            (Some(ModeArg::Component), name) => Some(ComponentMeta {
                name: name.unwrap_or_else(|| stem_of(&path)),
                shadow: true,
            }),
            (_, Some(name)) => Some(ComponentMeta { name, shadow: true }),
            _ => None,
        };
        compile_one(&path, out.as_deref(), mode.clone(), no_dts, styles_text.as_deref(), component_meta, no_ssr)?;
    }
    println!("compiled {count} file(s)");
    Ok(())
}

fn stem_of(path: &Path) -> String {
    path.file_name()
        .and_then(|f| f.to_str())
        .map(|s| {
            s.strip_suffix(".sui")
                .or_else(|| s.strip_suffix(".html.set"))
                .unwrap_or(s)
                .to_string()
        })
        .unwrap_or_default()
}

fn is_sui_source(name: &str) -> bool {
    name.ends_with(".sui") || name.ends_with(".html.set")
}

fn collect_targets(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            collect_targets(&p, out)?;
        } else if p
            .file_name()
            .and_then(|f| f.to_str())
            .is_some_and(is_sui_source)
        {
            out.push(p);
        }
    }
    Ok(())
}

fn compile_one(
    path: &Path,
    out_override: Option<&Path>,
    mode: Option<ModeArg>,
    no_dts: bool,
    styles: Option<&str>,
    component_meta: Option<ComponentMeta>,
    no_ssr: bool,
) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    let mode = mode.map(|m| m.into()).unwrap_or(CompileMode::Auto);
    // Resolve the emit dir for this file. Without --out, outputs land in
    // the project mirror tree at `<repo-root>/.sui-out/<src-rel>/`;
    // with --out, all files share that directory (legacy single-dir mode).
    let mirror_mode = out_override.is_none();
    let out_dir: PathBuf = match out_override {
        Some(p) => p.to_path_buf(),
        None => resolve_out_dir(path),
    };
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("create out dir {}", out_dir.display()))?;
    // Paired controller is auto-detected: if `<stem>.controller.ts`
    // exists next to the source, the compiler emits
    // `import setup from "<rel>"` where `<rel>` is the path from the
    // emit dir back to the controller. In mirror mode that's an absolute
    // path back to the source dir; in legacy --out=source-dir mode it's
    // `./<stem>.controller.ts`. Authors who want template-only just
    // don't write the .controller.ts file.
    let setup_url = paired_setup(path, &out_dir, mirror_mode);
    let opts = CompileOpts {
        setup_url,
        mode,
        source_path: Some(path.to_path_buf()),
        styles: styles.map(|s| s.to_string()),
        component_meta,
        emit_dts: !no_dts,
        emit_ssr: !no_ssr,
    };
    let result = compile(&source, opts).map_err(|e| {
        let rendered = e.render(&source);
        anyhow!("{rendered}")
    })?;

    let stem = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow!("bad path"))?;
    let stem_no_ext = stem
        .strip_suffix(".html.set")
        .or_else(|| stem.strip_suffix(".sui"))
        .unwrap_or(stem);
    // In mirror mode the emitted JS is named `<stem>.compiled.js` so
    // imports can name it directly without a separate rename step in
    // the build pipeline. With --out we keep the legacy `<stem>.js`
    // for callers that expect that name.
    let js_name = if mirror_mode {
        format!("{stem_no_ext}.compiled.js")
    } else {
        format!("{stem_no_ext}.js")
    };
    let js_path = out_dir.join(&js_name);
    fs::write(&js_path, &result.js)
        .with_context(|| format!("write {}", js_path.display()))?;
    if let Some(cem) = result.cem {
        let cem_path = out_dir.join(format!("{stem_no_ext}.custom-elements.json"));
        let pretty = serde_json::to_string_pretty(&cem)?;
        fs::write(&cem_path, pretty)
            .with_context(|| format!("write {}", cem_path.display()))?;
    }
    if let Some(schema) = result.schema {
        let schema_path = out_dir.join(format!("{stem_no_ext}.schema.json"));
        let pretty = serde_json::to_string_pretty(&schema)?;
        fs::write(&schema_path, pretty)
            .with_context(|| format!("write {}", schema_path.display()))?;
    }
    // `.sui.meta.json` sidecar — binding info for LSP/IDE tooling. The
    // runtime never loads this; it's purely developer-facing fuel.
    if let Some(meta) = result.meta {
        let meta_path = out_dir.join(format!("{stem_no_ext}.sui.meta.json"));
        let pretty = serde_json::to_string_pretty(&meta)?;
        fs::write(&meta_path, pretty)
            .with_context(|| format!("write {}", meta_path.display()))?;
    }
    // `.sui.d.ts` lands in the mirror tree alongside the other emit
    // artifacts. Controllers' `import type { Scope } from './<name>.sui'`
    // resolves to it via the workspace tsconfig `paths` mapping.
    if let Some(dts) = result.dts {
        let dts_path = out_dir.join(format!("{stem}.d.ts"));
        fs::write(&dts_path, dts)
            .with_context(|| format!("write {}", dts_path.display()))?;
    }
    // Write the static SSR HTML skeleton so the server can splice it into
    // the first-paint response without re-rendering at request time.
    if let Some(ssr_html) = result.ssr_html {
        let ssr_path = out_dir.join(format!("{stem_no_ext}.ssr.html"));
        fs::write(&ssr_path, ssr_html)
            .with_context(|| format!("write {}", ssr_path.display()))?;
    }
    Ok(())
}

fn paired_setup(path: &Path, out_dir: &Path, mirror_mode: bool) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    let stem = file_name
        .strip_suffix(".html.set")
        .or_else(|| file_name.strip_suffix(".sui"))?;
    let parent = path.parent()?;
    // page.sui pairs with controller.ts (no stem prefix); every other
    // stem pairs with <stem>.controller.ts.
    let filename = if stem == "page" {
        "controller.ts".to_string()
    } else {
        format!("{stem}.controller.ts")
    };
    let candidate = parent.join(&filename);
    if !candidate.exists() {
        return None;
    }
    if mirror_mode {
        // Mirror tree may be far from the source. Walk up from out_dir
        // until we hit the source dir, then reuse that as the import path.
        let rel = pathdiff(&candidate, out_dir);
        Some(rel)
    } else {
        Some(format!("./{filename}"))
    }
}

/// Compute a relative path from `from_dir` to `to_file` using `..`
/// segments. Falls back to the absolute `to_file` path when the two have
/// no common ancestor.
fn pathdiff(to_file: &Path, from_dir: &Path) -> String {
    use std::path::Component;
    let to = match to_file.canonicalize() {
        Ok(p) => p,
        Err(_) => to_file.to_path_buf(),
    };
    let from = match from_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => from_dir.to_path_buf(),
    };
    let to_parts: Vec<Component> = to.components().collect();
    let from_parts: Vec<Component> = from.components().collect();
    let common = to_parts
        .iter()
        .zip(from_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();
    if common == 0 {
        return to.to_string_lossy().into_owned();
    }
    let ups = from_parts.len() - common;
    let mut out = String::new();
    for _ in 0..ups {
        out.push_str("../");
    }
    if ups == 0 {
        out.push_str("./");
    }
    let remainder: PathBuf = to_parts[common..].iter().collect();
    out.push_str(&remainder.to_string_lossy());
    out
}
