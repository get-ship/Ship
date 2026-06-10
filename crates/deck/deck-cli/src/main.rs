//! `deck` — the deck framework CLI (the `next` / `vite` analog).
//!
//! deck is the framework; an app (the hub `apps/shell`, a space scaffold) is
//! built *with* deck via `deck build`, just as `next build` builds a Next app.
//! This is why the command lives here and not in the `ship` app CLI. Backed by
//! `deck-bundler` (Rolldown). `.sui` sources must be compiled first
//! by the deck bundler; `deck dev` / `deck serve` are future commands.

mod scaffold;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use deck_bundler::TargetKind;

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
        /// Materialize the /_deck/* framework surface into the output (self-contained).
        #[arg(long)]
        bundle_framework: bool,
        /// Skip minification.
        #[arg(long)]
        no_minify: bool,
    },

    /// Bake all framework + consumer bundles via the in-process Rolldown
    /// bundler. Reads deck.jsonc for consumer targets; framework singletons
    /// are hardcoded in deck-bundler. Honesty guards verify every framework
    /// bundle is non-empty after the bake — a silently-failed target cannot
    /// ship empty bytes.
    BuildAssets,
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
    // Surface bundler diagnostics (e.g. a view referencing a component whose
    // pack isn't declared in `deck.jsonc packs`) to stderr. Honors RUST_LOG;
    // floors at `warn` so the diagnostic always shows even when unset.
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .without_time()
        .try_init();
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
        Command::BuildAssets => build_assets(),
    }
}

/// The framework bundles deck-core/build.rs bakes via `include_str!`. Verified
/// non-empty after the bake so a silently-failed target cannot ship empty bytes
/// (the failure mode build.rs guards — top nav vanishing with no warning).
const FRAMEWORK_BUNDLES: &[&str] = &[
    "sui-runtime.js",
    "sui-dom.js",
    "sui-sdk-client.js",
];

fn build_assets() -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let repo_root = find_repo_root(&cwd)?;

    let cfg = deck_bundler::DeckConfig::load(&repo_root);
    let bundler = deck_bundler::Bundler::start(repo_root.clone(), None, cfg.clone());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    rt.block_on(bundler.build_all())
        .map_err(|e| anyhow::anyhow!("build_all failed: {e}"))?;

    // Build the layout once — used for both the framework verify and the
    // consumer guard below. assets_dir_for("sui-runtime") is now
    // .deck/framework/ (the canonical bake target per Recommendation B1).
    let layout = deck_bundler::targets::BundleLayout::with_config(
        repo_root.clone(),
        None,
        cfg,
    );

    // Verify framework bundles are non-empty after the bake. build_all returns
    // Err on per-target failure, but a swallowed empty-write wouldn't surface
    // there — keep this as defense in depth so the daemon can never ship empty
    // framework bytes. Use assets_dir_for("sui-runtime") so the verify path
    // stays in sync with the bundler's output path (.deck/framework/).
    let fw_dir = layout.assets_dir_for("sui-runtime");
    for name in FRAMEWORK_BUNDLES {
        let path = fw_dir.join(name);
        let len = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if len == 0 {
            bail!("framework bundle missing or empty after bake: {}", path.display());
        }
        println!("  {name}: {len} bytes");
    }

    // Consumer guard: the .deck/ bundles declared in deck.jsonc have no
    // build.rs guard — verify each is non-empty as defense in depth.
    // Use BundleLayout to derive the exact output path per target (same routing
    // logic the bundler uses) so the guard is always in sync with reality.
    //
    // Dev-loop skip: when DECK_DEV_SKIP_LAZY=1, the same lazy-pack targets
    // the bundler skipped above are also skipped by the guard (otherwise
    // a successful dev build would always fail the guard for the skipped
    // targets). The bundler's skip log is the ground truth for what was
    // skipped.
    let skip_lazy = std::env::var("DECK_DEV_SKIP_LAZY")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    // consumer_targets() returns BundleTarget (no kind field); the kind
    // lives on cfg.targets. Look it up by name for the skip check.
    for t in layout.consumer_targets() {
        let kind_is_lazy = layout
            .cfg
            .targets
            .iter()
            .any(|c| c.name == t.name && matches!(c.kind, TargetKind::LazyPack));
        if skip_lazy && kind_is_lazy {
            println!(
                "  {}: skipped (DECK_DEV_SKIP_LAZY=1, kind=lazy-pack)",
                t.name
            );
            continue;
        }
        let out_dir = layout.assets_dir_for(&t.name);
        let out_path = out_dir.join(&t.out_file);
        let len = std::fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
        if len == 0 {
            bail!(
                "consumer bundle missing or empty after bake: {} \
                 (target `{}` failed — check bundler logs)",
                out_path.display(),
                t.name
            );
        }
        println!("  {}: {len} bytes", t.name);
    }

    println!("deck build-assets: bundles baked via in-process Rolldown");
    println!("note: lucide-icons.json is committed source; regen via `cargo xtask gen-icons`");
    Ok(())
}

/// Resolve the Cargo workspace root from `start`. It's the cwd for module
/// resolution and the source of the framework bundles. Walking the topmost
/// `Cargo.toml` is the one canonical fact; any monorepo-specific marker
/// surfaces later when bundle_app reads the framework assets dir.
fn find_repo_root(start: &Path) -> Result<PathBuf> {
    deck_bundler::paths::topmost_cargo_root(start).ok_or_else(|| {
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

    // Load deck.jsonc to pick up `firstPartyApp` (sensible defaults when absent).
    // The first-party-app config drives the per-tag component resolver: chrome
    // `.sui` emit `@sui-tag/<tag>` imports that resolve to workspace source.
    let cfg = deck_bundler::DeckConfig::load(&repo_root);

    // The app graph compiles its own `.sui` in-process via the linked
    // sui-compiler crate (#255) — no binary to locate.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("tokio runtime")?;
    let b = rt
        .block_on(deck_bundler::app::bundle_app(
            &repo_root,
            cfg.first_party_app.as_ref(),
            &cfg.packs,
            &cfg.component_source_roots,
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
        println!("  _deck  {} files (self-contained)", b.framework_assets);
    }
    Ok(())
}
