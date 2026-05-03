//! `sui-compiler` CLI. Compiles a single `.sui` file or a
//! directory of them, writing the emitted JS (and CEM JSON for
//! components) to `--out`. Errors render with file path, line, and
//! column.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

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
        /// Output directory.
        #[arg(long)]
        out: PathBuf,
        /// Override mode (otherwise auto-detected from --component-name).
        #[arg(long)]
        mode: Option<ModeArg>,
        /// Suppress emitting the `.sui.d.ts` declaration file.
        #[arg(long)]
        no_dts: bool,
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
    let cli = Cli::parse();
    match cli.cmd {
        Command::Build { input, out, mode, no_dts, styles, component_name } => {
            build(input, out, mode, no_dts, styles, component_name)
        }
    }
}

fn build(
    input: PathBuf,
    out: PathBuf,
    mode: Option<ModeArg>,
    no_dts: bool,
    styles: Option<PathBuf>,
    component_name: Option<String>,
) -> Result<()> {
    fs::create_dir_all(&out).with_context(|| format!("create out dir {}", out.display()))?;
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
        compile_one(&path, &out, mode.clone(), no_dts, styles_text.as_deref(), component_meta)?;
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
    out_dir: &Path,
    mode: Option<ModeArg>,
    no_dts: bool,
    styles: Option<&str>,
    component_meta: Option<ComponentMeta>,
) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    let mode = mode.map(|m| m.into()).unwrap_or(CompileMode::Auto);
    // Paired controller is auto-detected: if `<stem>.ts` exists next to
    // the source, the compiler emits `import setup from "./<stem>.ts"`.
    // Authors who want template-only just don't write the .ts file.
    let setup_url = paired_setup(path);
    let opts = CompileOpts {
        setup_url,
        mode,
        source_path: Some(path.to_path_buf()),
        styles: styles.map(|s| s.to_string()),
        component_meta,
        emit_dts: !no_dts,
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
    let js_path = out_dir.join(format!("{stem_no_ext}.js"));
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
    // Write the .d.ts next to the source file so `import type { Scope } from
    // './view.sui'` resolves correctly regardless of --out placement.
    if let Some(dts) = result.dts {
        let src_dir = path.parent().unwrap_or(Path::new("."));
        let dts_path = src_dir.join(format!("{stem}.d.ts"));
        fs::write(&dts_path, dts)
            .with_context(|| format!("write {}", dts_path.display()))?;
    }
    Ok(())
}

fn paired_setup(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    let stem = file_name
        .strip_suffix(".html.set")
        .or_else(|| file_name.strip_suffix(".sui"))?;
    let parent = path.parent()?;
    let candidate = parent.join(format!("{stem}.ts"));
    if candidate.exists() {
        Some(format!("./{stem}.ts"))
    } else {
        None
    }
}
