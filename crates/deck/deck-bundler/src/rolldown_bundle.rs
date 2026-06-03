//! Rolldown bundler — in-process Rust bundler with a SUI resolve/load plugin
//! (#163). Replaces the bun worker + esbuild path. Rolldown handles TS transpile
//! + node resolution natively (oxc); the plugin only:
//!   - `resolve_id`: redirect `./X.compiled.js` siblings to a virtual id,
//!     externalize the framework singletons, re-anchor `@ship/<pkg>` to
//!     workspace `src/`.
//!   - `load`: compile `.sui` (and `.compiled.js`→sibling `.sui`) via
//!     sui-compiler, returning JS.
//! Output goes to `assets_dir_for` (sui-out/dist) — no source-tree writes.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use rolldown::plugin::{
    HookLoadArgs, HookLoadOutput, HookLoadReturn, HookResolveIdArgs, HookResolveIdOutput,
    HookResolveIdReturn, HookUsage, Plugin, PluginContext, SharedLoadPluginContext,
};
use rolldown::{BundlerBuilder, BundlerOptions, InputItem, OutputFormat};
use rolldown_common::{ModuleType, ResolvedExternal};

use super::compile::{compile_sui, CompileMode};
use super::proto::BundleManifest;
use super::targets::{BundleLayout, BundleTarget, TargetEntry};

const SINGLETONS: [&str; 3] = ["@ship/sui-runtime", "@ship/sui-dom", "@ship/sdk-client"];

/// Component source roots — a `.sui` under one is a custom element; else view.
fn detect_mode(abs: &Path, repo_root: &Path) -> (CompileMode, String) {
    let stem = abs.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
    for rel in ["packages/components/ui-components/src", "apps/shell/src/components"] {
        if abs.starts_with(repo_root.join(rel)) {
            return (CompileMode::Component, stem);
        }
    }
    let parent = abs
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let mode = if !stem.is_empty() && stem == parent {
        CompileMode::Component
    } else {
        CompileMode::View
    };
    (mode, stem)
}

fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            o => out.push(o.as_os_str()),
        }
    }
    out
}

/// SUI compile plugin — `resolve_id` redirects `*.compiled.js` siblings to a
/// virtual id and re-anchors `@ship/<pkg>` to workspace `src/`; `load` compiles
/// the sibling `.sui` via sui-compiler. Reused by the app build (`app.rs`) so
/// the SPA graph compiles its own `.sui` in-process — one compile path, no
/// on-disk `.compiled.js` pre-step.
#[derive(Debug)]
pub(crate) struct SuiPlugin {
    pub(crate) repo_root: PathBuf,
    pub(crate) compiler: PathBuf,
}

impl SuiPlugin {
    /// Compile a `.sui` to JS via sui-compiler (component or view per dir).
    ///
    /// The output dir must be UNIQUE per invocation. Every view entry is
    /// `page.sui` (stem `page`), so a process-shared `shipbundle-page-{pid}`
    /// dir let concurrent cold-builds of DIFFERENT views read/write/serve one
    /// another's `page.js` — a 200-OK wrong bundle. A RAII `TempDir` (removed
    /// when this fn returns) gives every compile its own directory and also
    /// fixes the temp-dir leak the per-pid scheme accumulated.
    fn compile(&self, sui: &Path) -> Result<String> {
        let (mode, stem) = detect_mode(sui, &self.repo_root);
        let out = tempfile::Builder::new()
            .prefix(&format!("shipbundle-{}-", std::process::id()))
            .tempdir()?;
        let res = compile_sui(&self.compiler, sui, out.path(), mode, &stem)
            .map_err(|e| anyhow!("compile {}: {e}", sui.display()))?;
        Ok(std::fs::read_to_string(&res.js_path)?)
    }
}

impl Plugin for SuiPlugin {
    fn name(&self) -> Cow<'static, str> {
        "ship-sui".into()
    }

    async fn resolve_id(
        &self,
        _ctx: &PluginContext,
        args: &HookResolveIdArgs<'_>,
    ) -> HookResolveIdReturn {
        let spec = args.specifier;
        // Framework singletons stay external (one runtime instance).
        if SINGLETONS.contains(&spec) {
            return Ok(Some(HookResolveIdOutput {
                id: spec.into(),
                external: Some(ResolvedExternal::Bool(true)),
                ..Default::default()
            }));
        }
        // `*.compiled.js` → virtual id; `load` compiles the sibling `.sui`. The
        // `.compiled.js` is synthetic (never on disk), so we must redirect here
        // rather than fall through to Rolldown's native resolver, which would
        // fail to resolve a nonexistent file. Two shapes reach here: relative
        // `./X.compiled.js` siblings (umbrella `index.ts` imports), and the
        // ABSOLUTE paths the virtual-aggregate entry emits (sui-marketing &
        // separate-bundle islands) — both must become a virtual id.
        if spec.ends_with(".compiled.js") {
            if Path::new(spec).is_absolute() {
                return Ok(Some(HookResolveIdOutput::from_id(
                    normalize(Path::new(spec)).to_string_lossy().to_string(),
                )));
            }
            if (spec.starts_with("./") || spec.starts_with("../"))
                && let Some(importer) = args.importer
            {
                let dir = Path::new(importer).parent().unwrap_or(&self.repo_root);
                let abs = normalize(&dir.join(spec));
                return Ok(Some(HookResolveIdOutput::from_id(abs.to_string_lossy().to_string())));
            }
        }
        // `@ship/<non-singleton>` → workspace package source (not prebuilt
        // dist) so its components compile from source too. A bare specifier
        // (`@ship/ui-components`) resolves to `src/index.ts`; a subpath
        // (`@ship/ui-components/tokens/themes`) resolves to the matching file
        // under `src/` — honoring the package's `exports` subpath map. Without
        // this the subpath collapsed to `src/index.ts`, surfacing as a
        // `MissingExport` for the named imports that live in the subpath file.
        if let Some(rest) = spec.strip_prefix("@ship/") {
            let name = rest.split('/').next().unwrap_or(rest);
            let subpath = rest.strip_prefix(name).map(|s| s.trim_start_matches('/'));
            let cands = if let Some(s) = name.strip_prefix("sui-") {
                vec![s.to_string(), name.to_string()]
            } else {
                vec![name.to_string()]
            };
            for d in &cands {
                for umbrella in ["", "sui", "deck", "components"] {
                    let pkg = if umbrella.is_empty() {
                        self.repo_root.join("packages").join(d)
                    } else {
                        self.repo_root.join("packages").join(umbrella).join(d)
                    };
                    if !pkg.join("src/index.ts").exists() {
                        continue
                    }
                    let src = pkg.join("src");
                    let target = match subpath {
                        None | Some("") => src.join("index.ts"),
                        Some(sub) => {
                            // `tokens/themes` → tokens/themes.ts, or its index.ts.
                            let direct = src.join(sub);
                            if direct.extension().is_some() && direct.exists() {
                                direct
                            } else {
                                let with_ext = src.join(format!("{sub}.ts"));
                                if with_ext.exists() {
                                    with_ext
                                } else {
                                    src.join(sub).join("index.ts")
                                }
                            }
                        }
                    };
                    if target.exists() {
                        return Ok(Some(HookResolveIdOutput::from_id(
                            target.to_string_lossy().to_string(),
                        )));
                    }
                }
            }
        }
        // Everything else: let Rolldown's native resolver handle it (relative
        // .ts/.js, node_modules, extension resolution).
        Ok(None)
    }

    async fn load(&self, _ctx: SharedLoadPluginContext, args: &HookLoadArgs<'_>) -> HookLoadReturn {
        let id = args.id;
        let sui: Option<PathBuf> = if let Some(base) = id.strip_suffix(".compiled.js") {
            Some(PathBuf::from(format!("{base}.sui")))
        } else if id.ends_with(".sui") {
            Some(PathBuf::from(id))
        } else {
            None
        };
        if let Some(sui) = sui {
            let code = self
                .compile(&sui)
                .map_err(|e| anyhow!("{e}"))?;
            return Ok(Some(HookLoadOutput {
                code: code.into(),
                module_type: Some(ModuleType::Js),
                ..Default::default()
            }));
        }
        // Real .ts/.js — Rolldown loads + transpiles natively.
        Ok(None)
    }

    fn register_hook_usage(&self) -> HookUsage {
        HookUsage::ResolveId | HookUsage::Load
    }
}

/// Build one target with Rolldown. Output written under `assets_dir_for`;
/// returns the manifest. `staging` holds generated virtual-aggregate entries.
pub async fn bundle_target(
    layout: &BundleLayout,
    target: &BundleTarget,
    compiler: &Path,
    staging: &Path,
    production: bool,
) -> Result<BundleManifest> {
    // Entry: real `.ts`/`.sui` directly, or a generated `.ts` for virtual
    // aggregates importing each source's `.compiled.js` (so `load` compiles it).
    let entry: PathBuf = match &target.entry {
        TargetEntry::Real(p) => p.clone(),
        TargetEntry::Virtual { sources } => {
            std::fs::create_dir_all(staging)?;
            let mut imports = String::new();
            for s in sources {
                let compiled = s.with_extension("compiled.js");
                imports.push_str(&format!("import {:?};\n", compiled.to_string_lossy()));
            }
            let entry = staging.join(format!("_entry_{}.ts", target.name.replace('/', "_")));
            std::fs::write(&entry, imports)?;
            entry
        }
    };

    let out_dir = layout.assets_dir_for(&target.name);
    std::fs::create_dir_all(&out_dir)?;
    let entry_name = target.out_file.strip_suffix(".js").unwrap_or(&target.out_file);

    let options = BundlerOptions {
        input: Some(vec![InputItem {
            name: Some(entry_name.to_string()),
            import: entry.to_string_lossy().to_string(),
        }]),
        cwd: Some(layout.repo_root.clone()),
        dir: Some(out_dir.to_string_lossy().to_string()),
        format: Some(OutputFormat::Esm),
        minify: Some(production.into()),
        ..Default::default()
    };

    let plugin = SuiPlugin {
        repo_root: layout.repo_root.clone(),
        compiler: compiler.to_path_buf(),
    };
    // The bundler + its oxc module graph live in `bundler` and are dropped at
    // the end of this scope. Wrap the build/write in an inner block so the
    // transient arenas are released BEFORE the `malloc_trim` below — otherwise
    // the graph is still alive when we trim and nothing is reclaimed.
    {
        let mut bundler = BundlerBuilder::default()
            .with_options(options)
            .with_plugins(vec![Arc::new(plugin)])
            .build()
            .map_err(|e| anyhow!("rolldown build({}): {e:?}", target.name))?;

        bundler
            .write()
            .await
            .map_err(|e| anyhow!("rolldown write({}): {e:?}", target.name))?;
    }
    // Return the freed module-graph arenas to the OS so repeated builds don't
    // ratchet RSS up under glibc (#256). No-op off glibc.
    crate::arena::trim_arenas();

    let out_path = out_dir.join(&target.out_file);
    let bytes = std::fs::metadata(&out_path).map(|m| m.len() as usize).unwrap_or(0);
    Ok(BundleManifest {
        target: target.name.clone(),
        out_path: out_path.to_string_lossy().to_string(),
        bytes,
        inputs: match &target.entry {
            TargetEntry::Real(p) => vec![p.to_string_lossy().to_string()],
            TargetEntry::Virtual { sources } => {
                sources.iter().map(|p| p.to_string_lossy().to_string()).collect()
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> PathBuf {
        let mut d = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
        loop {
            if d.join("package.json").exists() && d.join("packages").is_dir() {
                return d;
            }
            if !d.pop() {
                panic!("repo root not found");
            }
        }
    }

    fn compiler() -> PathBuf {
        ship_substrate::locate::walk_for_target_binary("sui-compiler")
            .expect("sui-compiler built")
    }

    /// Bundle the REAL sui-components umbrella + a view through Rolldown into a
    /// scratch out_root; assert full-size, runtime external, valid.
    #[tokio::test(flavor = "multi_thread")]
    async fn rolldown_bundles_real_umbrella() {
        let repo = repo_root();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout =
            BundleLayout::new(repo, None).with_out_root(scratch.path().to_path_buf());
        let comp = compiler();

        let targets = layout.all_targets();
        let umbrella = targets.iter().find(|t| t.name == "sui-components").unwrap();
        let m = bundle_target(&layout, umbrella, &comp, staging.path(), true)
            .await
            .expect("umbrella should bundle");
        assert!(m.bytes > 200_000, "umbrella full-size, got {}", m.bytes);
        let body = std::fs::read_to_string(&m.out_path).unwrap();
        assert!(body.contains("@ship/sui-runtime"), "runtime kept external");
    }

    /// The `sui-marketing` bundle is now TRACED from the site-owned entry
    /// `apps/site/src/marketing/marketing-bundle.ts` (no hardcoded source list).
    /// Rolldown must follow that import graph and register exactly the marketing
    /// tag set — no tag dropped, and the full umbrella must NOT leak in.
    #[tokio::test(flavor = "multi_thread")]
    async fn rolldown_bundles_marketing_traced() {
        let repo = repo_root();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout =
            BundleLayout::new(repo, None).with_out_root(scratch.path().to_path_buf());
        let comp = compiler();

        let targets = layout.all_targets();
        let marketing = targets.iter().find(|t| t.name == "sui-marketing").unwrap();
        // Traced entry: a real on-disk site module, not a synthetic aggregate.
        assert!(
            matches!(&marketing.entry, TargetEntry::Real(p)
                if p.ends_with("apps/site/src/marketing/marketing-bundle.ts")),
            "sui-marketing must be a Real (traced) entry, got {:?}",
            marketing.entry
        );

        let m = bundle_target(&layout, marketing, &comp, staging.path(), true)
            .await
            .expect("marketing should bundle");
        let body = std::fs::read_to_string(&m.out_path).unwrap();
        assert!(body.contains("@ship/sui-runtime"), "runtime kept external");

        // The 10 traceable marketing components (sui-stat / sui-toolbar /
        // sui-progress from the retired const have no `.sui` source and are not
        // members — see marketing-bundle.ts). Each registers via its
        // `component(\`<tag>\`, …)` call; the tag literal survives minification.
        for tag in [
            "sui-icon", "sui-badge", "sui-button", "sui-separator",
            "sui-status-dot", "sui-table", "sui-bar-chart",
            "sui-sparkline", "sui-brand-mark", "sui-theme-picker",
        ] {
            assert!(
                body.contains(&format!("`{tag}`")),
                "marketing bundle must register {tag}"
            );
        }
        // Other-pack tags must NOT leak in (entry imports specific components,
        // not a package index). `ship-git-diff` lives in the @ship/code pack, not
        // anything the focused marketing entry imports.
        assert!(
            !body.contains("`ship-git-diff`"),
            "out-of-entry packs must not leak into the focused marketing bundle"
        );
    }

    /// The `sui-arcade` bundle is the heavy lazy chunk for the marketing site's
    /// games. Its `index.ts` registers every arcade tag and pulls the
    /// pixi + arcade-engine graph. pixi + arcade-engine must be BUNDLED (it's the
    /// big chunk), while the runtime singleton stays EXTERNAL — exactly one
    /// `@ship/sui-runtime` instance across the page.
    #[tokio::test(flavor = "multi_thread")]
    async fn rolldown_bundles_arcade_pixi_inline_runtime_external() {
        let repo = repo_root();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout =
            BundleLayout::new(repo, None).with_out_root(scratch.path().to_path_buf());
        let comp = compiler();

        let targets = layout.all_targets();
        let arcade = targets.iter().find(|t| t.name == "sui-arcade").unwrap();
        let m = bundle_target(&layout, arcade, &comp, staging.path(), true)
            .await
            .expect("arcade should bundle");
        // The heavy lazy chunk: pixi (~800 KB) is bundled in.
        assert!(m.bytes > 200_000, "arcade bundles pixi, got {} bytes", m.bytes);
        let body = std::fs::read_to_string(&m.out_path).unwrap();
        // Runtime stays external (one instance) — an import of the bare
        // specifier survives, not an inlined second copy.
        assert!(
            body.contains("@ship/sui-runtime"),
            "arcade keeps the runtime singleton external"
        );
        // Each arcade tag registers via its `component(\`<tag>\`, …)` call.
        for tag in ["sui-stage", "sui-sprite", "sui-emitter", "sui-input", "sui-hud"] {
            assert!(
                body.contains(&format!("`{tag}`")),
                "arcade bundle must register {tag}"
            );
        }
    }

    /// The `sui-pdf` bundle wraps the pdf.js (`pdfjs-dist`) engine. The pack's
    /// `index.ts` side-effect-imports the worker module
    /// (`pdfjs-dist/build/pdf.worker.mjs`) instead of a `?url` asset — the deck
    /// bundler has no `?url` support, so a `?url` import ABORTS the build (the bug
    /// this guards). Bundling the worker module straight in (it registers
    /// `globalThis.pdfjsWorker`) keeps the worker same-origin + offline with no
    /// served-asset URL. This test asserts the pack BUILDS through Rolldown, the
    /// engine + worker are inlined (heavy chunk), the runtime stays external, and
    /// `<ship-pdf>` registers.
    #[tokio::test(flavor = "multi_thread")]
    async fn rolldown_bundles_pdf_with_worker_inlined() {
        let repo = repo_root();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout =
            BundleLayout::new(repo, None).with_out_root(scratch.path().to_path_buf());
        let comp = compiler();

        let targets = layout.all_targets();
        let pdf = targets.iter().find(|t| t.name == "sui-pdf").unwrap();
        let m = bundle_target(&layout, pdf, &comp, staging.path(), true)
            .await
            .expect("pdf pack must bundle through Rolldown (no `?url` abort)");
        // The heavy lazy chunk: pdf.js engine + worker are inlined.
        assert!(m.bytes > 200_000, "pdf bundles the engine, got {} bytes", m.bytes);
        let body = std::fs::read_to_string(&m.out_path).unwrap();
        // Runtime stays external (one instance).
        assert!(
            body.contains("@ship/sui-runtime"),
            "pdf keeps the runtime singleton external"
        );
        // The worker module is bundled, not fetched — it side-effect-registers the
        // main-thread handler on `globalThis.pdfjsWorker`. Its presence proves the
        // worker resolves SAME-ORIGIN with no served-asset URL and no CDN.
        assert!(
            body.contains("pdfjsWorker"),
            "pdf must bundle the worker module's main-thread handler registration"
        );
        // No CDN / `?url` leaked into the bundle.
        assert!(
            !body.contains("pdf.worker.mjs?url"),
            "the `?url` asset suffix must be gone (it aborts the deck bundler)"
        );
        for needle in ["cdnjs", "unpkg", "jsdelivr"] {
            assert!(!body.contains(needle), "worker must not load from a CDN ({needle})");
        }
        // `<ship-pdf>` registers via its `component(\`ship-pdf\`, …)` call.
        assert!(
            body.contains("`ship-pdf`"),
            "pdf bundle must register the ship-pdf tag"
        );
    }
}
