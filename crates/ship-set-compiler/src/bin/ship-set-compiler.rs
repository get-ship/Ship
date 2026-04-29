//! `ship-set-compiler` CLI. Compiles a single `.html.set` file or a
//! directory of them, writing the emitted JS (and CEM JSON for
//! components) to `--out`. Errors render with file path, line, and
//! column.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

use ship_set_compiler::{compile, CompileMode, CompileOpts};

#[derive(Parser)]
#[command(name = "ship-set-compiler", about = "AOT compiler for .html.set files")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile one file or every `.html.set` under a directory.
    Build {
        /// Path to a `.html.set` file or a directory.
        input: PathBuf,
        /// Output directory.
        #[arg(long)]
        out: PathBuf,
        /// Override mode (otherwise auto-detected).
        #[arg(long)]
        mode: Option<ModeArg>,
        /// Skip writing the paired `.ts` setup import even if a
        /// sibling file exists.
        #[arg(long)]
        no_setup: bool,
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
        Command::Build { input, out, mode, no_setup } => build(input, out, mode, no_setup),
    }
}

fn build(input: PathBuf, out: PathBuf, mode: Option<ModeArg>, no_setup: bool) -> Result<()> {
    fs::create_dir_all(&out).with_context(|| format!("create out dir {}", out.display()))?;
    let mut targets = Vec::new();
    if input.is_dir() {
        collect_targets(&input, &mut targets)?;
    } else if input
        .file_name()
        .and_then(|f| f.to_str())
        .map_or(false, |n| n.ends_with(".html.set"))
    {
        targets.push(input.clone());
    } else {
        return Err(anyhow!("not a .html.set file or directory: {}", input.display()));
    }
    if targets.is_empty() {
        return Err(anyhow!("no .html.set files found under {}", input.display()));
    }
    let count = targets.len();
    for path in targets {
        compile_one(&path, &out, mode.clone(), no_setup)?;
    }
    println!("compiled {count} file(s)");
    Ok(())
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
            .map_or(false, |n| n.ends_with(".html.set"))
        {
            out.push(p);
        }
    }
    Ok(())
}

fn compile_one(path: &Path, out_dir: &Path, mode: Option<ModeArg>, no_setup: bool) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    let mode = mode.map(|m| m.into()).unwrap_or(CompileMode::Auto);
    let setup_url = if no_setup {
        None
    } else {
        paired_setup(path)
    };
    let opts = CompileOpts {
        setup_url,
        mode,
        source_path: Some(path.to_path_buf()),
    };
    let result = compile(&source, opts).map_err(|e| {
        let rendered = e.render(&source);
        anyhow!("{rendered}")
    })?;

    let stem = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow!("bad path"))?;
    let stem_no_ext = stem.trim_end_matches(".html.set");
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
    Ok(())
}

fn paired_setup(path: &Path) -> Option<String> {
    // For `foo/view.html.set` look for `foo/view.ts`.
    let file_name = path.file_name()?.to_str()?;
    let stem = file_name.trim_end_matches(".html.set");
    let parent = path.parent()?;
    let candidate = parent.join(format!("{stem}.ts"));
    if candidate.exists() {
        Some(format!("./{stem}.ts"))
    } else {
        None
    }
}
