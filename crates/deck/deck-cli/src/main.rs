//! `deck` — the deck framework CLI (the `next` / `vite` analog).
//!
//! deck is the framework; an app (the hub `apps/shell`, a space scaffold) is
//! built *with* deck via `deck build`, just as `next build` builds a Next app.
//! This is why the command lives here and not in the `ship` app CLI. Backed by
//! `deck-bundler` (Rolldown). `.sui` sources must be compiled first
//! by the deck bundler; `deck dev` / `deck serve` are future commands.

mod scaffold;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "deck", about = "The deck framework CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Regenerate the typed dynamic-route table (routes.gen.ts) from the app's
    /// studio view tree. Also run inside `build`; standalone for dev iteration.
    Routes {
        /// App root holding src/studio (default: cwd).
        #[arg(long, default_value = ".")]
        shell_root: PathBuf,
    },

    /// Generate boilerplate for a new view or primitive component.
    #[command(subcommand)]
    Scaffold(Scaffold),

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

#[derive(Subcommand)]
enum Scaffold {
    /// New SUI view under `views/<name>/` (page.sui + controller.ts +
    /// style.css + manifest.jsonc; optional view-private widget).
    View {
        /// View dir name + manifest `name` field. Lowercase ASCII + hyphens.
        name: String,
        /// URL pattern; defaults to `/<name>`.
        #[arg(long)]
        path: Option<String>,
        /// Optional view-private widget; tag must start with `<name>-`.
        #[arg(long)]
        component: Option<String>,
    },

    /// New SUI primitive under `packages/components/sui/primitives/src/<tag>/`.
    Component {
        /// Tag name (must start with `sui-`).
        tag: String,
        /// Also write a paired `<tag>.controller.ts` stub.
        #[arg(long)]
        controller: bool,
    },

    /// New first-party SUI component pack under `packages/components/<name>/`.
    /// Generates package.json, sui.config.ts, tsconfig.json, the umbrella
    /// `src/index.ts`, and a starter `<name>-example` component.
    Pack {
        /// Pack name (lowercase + hyphens). Becomes `@ship/<name>`.
        name: String,
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
        Command::Routes { shell_root } => {
            let cwd = std::env::current_dir().context("cwd")?;
            let root = if shell_root.is_absolute() { shell_root } else { cwd.join(shell_root) };
            let n = deck_bundler::routes::generate_routes(&root)?;
            println!("deck routes → {n} dynamic routes");
            Ok(())
        }
        Command::Scaffold(Scaffold::View { name, path, component }) => {
            scaffold::run_view(name, path, component)
        }
        Command::Scaffold(Scaffold::Component { tag, controller }) => {
            scaffold::run_component(tag, controller)
        }
        Command::Scaffold(Scaffold::Pack { name }) => scaffold::run_pack(name),
        Command::Build { shell_root, out, bundle_framework, no_minify } => {
            build(shell_root, out, bundle_framework, !no_minify)
        }
    }
}

/// Resolve the Cargo workspace root from `start`. It's the cwd for module
/// resolution and the source of the framework bundles. Walking the topmost
/// `Cargo.toml` is the one canonical fact; any monorepo-specific marker
/// surfaces later when bundle_app reads the framework assets dir.
fn find_repo_root(start: &Path) -> Result<PathBuf> {
    ship_substrate::locate::workspace_root_from(start).ok_or_else(|| {
        anyhow!("deck build: no Cargo workspace root above {}", start.display())
    })
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

    // The app graph compiles its own `.sui` in-process (Slice 4a) — locate the
    // sui-compiler the same way the view/component build does.
    let home = ship_substrate::ShipHome::from_env();
    let compiler = deck_bundler::compile::locate_sui_compiler(&home)
        .ok_or_else(|| anyhow!("deck build: sui-compiler not found (build it: cargo build -p sui-compiler)"))?;

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
            &compiler,
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
