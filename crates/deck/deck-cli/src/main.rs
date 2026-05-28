//! `deck` — the deck framework CLI (the `next` / `vite` analog).
//!
//! deck is the framework; an app (the hub `apps/shell`, a space scaffold) is
//! built *with* deck via `deck build`, just as `next build` builds a Next app.
//! This is why the command lives here and not in the `ship` app CLI. Backed by
//! `deck-bundler` (Rolldown). `.sui` sources must be compiled first
//! (`sui-build`); `deck dev` / `deck serve` are future commands.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "deck", about = "The deck framework CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build a SUI SPA app (hub, space scaffold) to a static output dir.
    Build {
        /// App root holding index.html, src/, public/ (default: cwd).
        #[arg(long, default_value = ".")]
        shell_root: PathBuf,
        /// Output dir (default: <shell-root>/dist).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Materialize the /_sui/* framework surface into the output (self-contained).
        #[arg(long)]
        bundle_framework: bool,
        /// Skip minification.
        #[arg(long)]
        no_minify: bool,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("deck: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Build { shell_root, out, bundle_framework, no_minify } => {
            build(shell_root, out, bundle_framework, !no_minify)
        }
    }
}

/// Walk up for the workspace root — the `crates/deck/deck-core/assets` marker.
/// It's the cwd for module resolution and the source of the framework bundles.
fn find_repo_root(start: &Path) -> Result<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("crates/deck/deck-core/assets").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            bail!(
                "deck build: no workspace root (crates/deck/deck-core/assets) above {}",
                start.display()
            );
        }
    }
}

fn build(shell_root: PathBuf, out: Option<PathBuf>, bundle_framework: bool, minify: bool) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let abs = |p: &Path| if p.is_absolute() { p.to_path_buf() } else { cwd.join(p) };

    let shell_root = abs(&shell_root)
        .canonicalize()
        .with_context(|| format!("shell-root {}", shell_root.display()))?;
    let out_dir = out.map(|o| abs(&o)).unwrap_or_else(|| shell_root.join("dist"));
    let repo_root = find_repo_root(&shell_root)?;
    let framework_assets_dir = repo_root.join("crates/deck/deck-core/assets");
    let deck_out_dir = repo_root.join(".deck");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("tokio runtime")?;
    let b = rt
        .block_on(deck_bundler::app::bundle_app(
            &repo_root,
            &shell_root,
            &out_dir,
            minify,
            bundle_framework,
            &framework_assets_dir,
            &deck_out_dir,
        ))
        .map_err(|e| anyhow::anyhow!("{e:#}"))?;

    println!("deck build → {}", b.out_dir.display());
    println!("  js  {}", b.js_href);
    println!("  css {}", b.css_href);
    if b.framework_assets > 0 {
        println!("  _sui  {} files (self-contained)", b.framework_assets);
    }
    Ok(())
}
