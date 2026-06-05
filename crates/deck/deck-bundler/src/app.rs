//! Deck `app` build target (#170) — bundles a SUI SPA app (the hub,
//! `apps/shell`, and the space scaffold, `apps/spaces`) to a static dir.
//!
//! This is the deck-owned replacement for the retired `shell-app.ts`
//! (`Bun.build`) — one bundler engine (Rolldown) for framework + views +
//! apps. Driven by `deck build` (`crates/deck/deck-cli`). The app graph
//! compiles its own `.sui` in-process via the shared `SuiPlugin`
//! (`rolldown_bundle.rs`) — the exact resolve/load hooks the view/component
//! build uses — so no on-disk `.compiled.js` pre-step is needed.
//! `AppPlugin` runs alongside it to externalize the framework singletons
//! (which resolve to `/_sui/*` via the document's importmap); it runs FIRST so
//! importmap specifiers (incl. `@ship/sui-runtime/*` subpaths) stay external
//! rather than being re-anchored to source by `SuiPlugin`.
//!
//! Steps mirror `buildShellApp`: bundle `src/main.ts` (split, hashed),
//! copy + hash the chrome stylesheet (plain CSS — no bundler needed), copy
//! `public/` verbatim, rewrite the two `/src/*` refs in `index.html`, and
//! (optionally) materialize the `/_sui/*` framework surface for a
//! self-contained build.

use std::borrow::Cow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use rolldown::plugin::{
    HookResolveIdArgs, HookResolveIdOutput, HookResolveIdReturn, HookUsage, Plugin, PluginContext,
};
use rolldown::{BundlerBuilder, BundlerOptions, InputItem, OutputFormat};
use rolldown_common::{ChunkFilenamesOutputOption, ResolvedExternal};

/// deck's baked framework bundles → their `/_sui/*` filename in the output.
/// Lucide icons are NOT here — the daemon serves them per-icon from its embedded
/// set at `/_sui/icons/{name}.svg`. Static-only deploys without a daemon use
/// `<sui-icon src=…>` (CDN escape hatch).
const FRAMEWORK_ASSETS: [(&str, &str); 4] = [
    ("sui-runtime.js", "_sui/runtime.js"),
    ("sui-dom.js", "_sui/dom.js"),
    ("sui-sdk-client.js", "_sui/sdk-client.js"),
    ("sui-components.js", "_sui/components.js"),
];

pub struct AppBuild {
    pub out_dir: PathBuf,
    pub js_href: String,
    pub css_href: String,
    pub framework_assets: usize,
}

/// Externalizes exactly the document's importmap specifiers, so the bundle
/// leaves `@ship/sui-runtime` (etc.) as bare imports the browser resolves to
/// `/_sui/*` — the single-runtime guarantee. `.sui` compilation is handled by
/// the sibling `SuiPlugin` in the same bundle pass.
#[derive(Debug)]
struct AppPlugin {
    externals: HashSet<String>,
}

impl Plugin for AppPlugin {
    fn name(&self) -> Cow<'static, str> {
        "deck-app-externals".into()
    }

    async fn resolve_id(
        &self,
        _ctx: &PluginContext,
        args: &HookResolveIdArgs<'_>,
    ) -> HookResolveIdReturn {
        if self.externals.contains(args.specifier) {
            return Ok(Some(HookResolveIdOutput {
                id: args.specifier.into(),
                external: Some(ResolvedExternal::Bool(true)),
                ..Default::default()
            }));
        }
        Ok(None)
    }

    fn register_hook_usage(&self) -> HookUsage {
        HookUsage::ResolveId
    }
}

/// Parse the bare specifiers from `<script type="importmap">` — the SSOT for
/// which imports resolve to daemon `/_sui/*` URLs at runtime, so the build
/// externalizes exactly those (never a hand-kept second list, #157).
fn importmap_externals(html: &str) -> Result<HashSet<String>> {
    let start = html
        .find("type=\"importmap\"")
        .and_then(|i| html[i..].find('>').map(|j| i + j + 1))
        .context("no <script type=\"importmap\"> in index.html")?;
    let end = html[start..]
        .find("</script>")
        .map(|j| start + j)
        .context("unterminated importmap script")?;
    let json: serde_json::Value =
        serde_json::from_str(html[start..end].trim()).context("importmap is not valid JSON")?;
    let imports = json
        .get("imports")
        .and_then(|v| v.as_object())
        .context("importmap has no `imports` object")?;
    let keys: HashSet<String> = imports.keys().cloned().collect();
    if keys.is_empty() {
        bail!("importmap has no imports");
    }
    Ok(keys)
}

/// Short content hash for cache-busting an asset filename (non-crypto).
fn content_hash(bytes: &[u8]) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut h);
    format!("{:016x}", h.finish())
}

/// Build a SUI SPA app to `out_dir`. `repo_root` is the workspace root (cwd for
/// module resolution); `shell_root` holds `index.html`, `src/`, `public/`.
/// `first_party_app` comes from `deck.jsonc` (via `DeckConfig`) and enables the
/// per-tag component resolver: chrome components resolve their primitive/domain
/// tags from source via `@sui-tag/<tag>`, exactly as views do. The umbrella
/// (`/_sui/components.js`) becomes redundant once this traced build is canonical.
pub async fn bundle_app(
    repo_root: &Path,
    first_party_app: Option<&deck_config::FirstPartyApp>,
    packs: &[String],
    shell_root: &Path,
    out_dir: &Path,
    minify: bool,
    bundle_framework: bool,
    framework_assets_dir: &Path,
    _deck_out_dir: &Path,
) -> Result<AppBuild> {
    // Typed route table first (#168): main.ts imports routes.gen.ts, so it must
    // be current before Rolldown reads the graph. No-op for apps with no studio.
    crate::routes::generate_routes(shell_root).context("generate routes.gen.ts")?;

    let index_path = shell_root.join("index.html");
    let html = std::fs::read_to_string(&index_path)
        .with_context(|| format!("read {}", index_path.display()))?;
    let externals = importmap_externals(&html)?;

    let main_ts = shell_root.join("src/main.ts");
    let css_entry = shell_root.join("src/chrome/layout.css");
    for p in [&main_ts, &css_entry] {
        if !p.exists() {
            bail!("app build: expected {}", p.display());
        }
    }

    // Wipe a conventional output dir so stale hashed chunks don't accumulate.
    let base = out_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if out_dir.exists() && (base == "dist" || out_dir.to_string_lossy().contains("/.deck")) {
        std::fs::remove_dir_all(out_dir).ok();
    }
    let assets_dir = out_dir.join("assets");
    std::fs::create_dir_all(&assets_dir)?;

    // 1. Bundle main.ts (split, hashed); framework left external via the plugin.
    let options = BundlerOptions {
        input: Some(vec![InputItem {
            name: Some("main".to_string()),
            import: main_ts.to_string_lossy().to_string(),
        }]),
        cwd: Some(repo_root.to_path_buf()),
        dir: Some(out_dir.to_string_lossy().to_string()),
        format: Some(OutputFormat::Esm),
        minify: Some(minify.into()),
        entry_filenames: Some(ChunkFilenamesOutputOption::String(
            "assets/[name]-[hash].js".to_string(),
        )),
        chunk_filenames: Some(ChunkFilenamesOutputOption::String(
            "assets/[name]-[hash].js".to_string(),
        )),
        ..Default::default()
    };
    // AppPlugin FIRST so importmap specifiers (incl. `@ship/sui-runtime/*`
    // subpaths) win as externals; SuiPlugin then compiles `.sui` siblings and
    // re-anchors the remaining `@ship/<pkg>` (e.g. sui-charts, pipelines) to
    // workspace source so the app graph compiles them in-process.
    let app_plugin = AppPlugin { externals };
    // Chrome components resolve their primitive/domain tags from source via
    // `@sui-tag/<tag>`, the same per-tag resolver the view build uses.
    // `emit_component_imports: true` causes each compiled `.sui` to emit
    // `@sui-tag/<tag>` registration imports; `component_index` maps every known
    // workspace tag to its source `.sui` so those imports resolve correctly.
    let resolver = crate::sui_plugin::component_index(repo_root, first_party_app, packs);
    let sui_plugin = crate::sui_plugin::SuiPlugin {
        repo_root: repo_root.to_path_buf(),
        emit_component_imports: true,
        component_index: resolver.index,
        shadowed_tags: resolver.shadowed,
    };
    let mut bundler = BundlerBuilder::default()
        .with_options(options)
        .with_plugins(vec![Arc::new(app_plugin), Arc::new(sui_plugin)])
        .build()
        .map_err(|e| anyhow!("rolldown build(app): {e:?}"))?;
    bundler
        .write()
        .await
        .map_err(|e| anyhow!("rolldown write(app): {e:?}"))?;

    // The entry chunk is `assets/main-<hash>.js` (input name "main").
    let js_name = std::fs::read_dir(&assets_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|n| n.starts_with("main-") && n.ends_with(".js"))
        .context("app build: no entry chunk assets/main-*.js emitted")?;
    let js_href = format!("/assets/{js_name}");

    // 2. Copy + hash the chrome stylesheet (plain CSS — no bundler needed).
    let css_bytes = std::fs::read(&css_entry)?;
    let css_name = format!("layout-{}.css", &content_hash(&css_bytes)[..8]);
    std::fs::write(assets_dir.join(&css_name), &css_bytes)?;
    let css_href = format!("/assets/{css_name}");

    // 3. Copy public/ verbatim (favicons, logos referenced by root-absolute href).
    let public_dir = shell_root.join("public");
    if public_dir.exists() {
        for entry in std::fs::read_dir(&public_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                std::fs::copy(entry.path(), out_dir.join(entry.file_name()))?;
            }
        }
    }

    // 4. Rewrite the two /src refs; preserve importmap, /_sui IIFEs, public hrefs.
    let rewritten = html
        .replace("/src/main.ts", &js_href)
        .replace("/src/chrome/layout.css", &css_href);
    std::fs::write(out_dir.join("index.html"), rewritten)?;

    // 5. Self-contained build: materialize the /_sui/* code surface locally.
    let mut framework_assets = 0;
    if bundle_framework {
        let sui_dir = out_dir.join("_sui");
        std::fs::create_dir_all(&sui_dir)?;
        for (src, dist_rel) in FRAMEWORK_ASSETS {
            let from = framework_assets_dir.join(src);
            if !from.exists() {
                bail!("app build: bundle_framework: missing framework asset {}", from.display());
            }
            std::fs::copy(&from, out_dir.join(dist_rel))?;
            framework_assets += 1;
        }
    }

    Ok(AppBuild {
        out_dir: out_dir.to_path_buf(),
        js_href,
        css_href,
        framework_assets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(p: &Path, s: &str) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, s).unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn bundles_app_externalizes_importmap_and_rewrites_html() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("index.html"),
            "<!doctype html><html><head>\
             <script type=\"importmap\">{\"imports\":{\"@ext/lib\":\"/x.js\"}}</script>\
             <link rel=\"stylesheet\" href=\"/src/chrome/layout.css\">\
             <script type=\"module\" src=\"/src/main.ts\"></script>\
             </head><body></body></html>",
        );
        // main imports a local module AND the importmap external.
        write(&r.join("src/main.ts"), "import './local'\nimport '@ext/lib'\nexport const x = 1\n");
        write(&r.join("src/local.ts"), "export const y = 2\n");
        write(&r.join("src/chrome/layout.css"), ".a{color:red}\n");

        let out = r.join("dist");
        let build = bundle_app(r, None, &[], r, &out, false, false, r, r)
            .await
            .expect("bundle_app");

        // 1. Entry chunk emitted + HTML rewritten to its hashed href.
        assert!(build.js_href.starts_with("/assets/main-") && build.js_href.ends_with(".js"));
        let html = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains(&build.js_href), "html keeps hashed js href");
        assert!(!html.contains("/src/main.ts"), "raw entry ref rewritten");
        assert!(html.contains(&build.css_href) && !html.contains("/src/chrome/layout.css"));
        assert!(html.contains("type=\"importmap\""), "importmap preserved");

        // 2. The importmap specifier is left external (a bare import survives);
        //    the local module is bundled in (no bare './local').
        let entry = out.join(build.js_href.trim_start_matches('/'));
        let js = std::fs::read_to_string(&entry).unwrap();
        assert!(js.contains("@ext/lib"), "importmap specifier externalized, not resolved");
        assert!(!js.contains("'./local'") && !js.contains("\"./local\""), "local module inlined");

        // 3. CSS copied verbatim under a hashed name.
        let css = std::fs::read_to_string(out.join(build.css_href.trim_start_matches('/'))).unwrap();
        assert_eq!(css, ".a{color:red}\n");
    }

    /// The app build compiles a `.sui` referenced by the entry IN-PROCESS (via
    /// the shared SuiPlugin) — no on-disk `.compiled.js` pre-step — while the
    /// runtime stays external (single-runtime guarantee). This is the core of
    /// Slice 4a: the shell no longer needs an on-disk pre-compile of its packs.
    #[tokio::test(flavor = "multi_thread")]
    async fn app_build_compiles_sui_entry_in_process() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("index.html"),
            "<!doctype html><html><head>\
             <script type=\"importmap\">{\"imports\":{\"@ship/sui-runtime\":\"/_sui/runtime.js\"}}</script>\
             <link rel=\"stylesheet\" href=\"/src/chrome/layout.css\">\
             <script type=\"module\" src=\"/src/main.ts\"></script>\
             </head><body></body></html>",
        );
        // main.ts side-effect-imports a `.compiled.js` sibling — exactly the
        // shape a pack's `src/index.ts` uses. No `.compiled.js` exists on disk;
        // SuiPlugin must synthesize + compile the sibling `widget.sui`.
        write(&r.join("src/main.ts"), "import './widget.compiled.js'\nexport const x = 1\n");
        write(&r.join("src/widget.sui"), "<div class=\"w\">hi {1 + 1}</div>\n");
        write(&r.join("src/chrome/layout.css"), ".a{color:red}\n");

        let out = r.join("dist");
        let build = bundle_app(r, None, &[], r, &out, false, false, r, r)
            .await
            .expect("bundle_app should compile the .sui entry in-process");

        let entry = out.join(build.js_href.trim_start_matches('/'));
        let js = std::fs::read_to_string(&entry).unwrap();
        // The synthetic `.compiled.js` specifier was resolved + inlined, not
        // left as a live import (no on-disk module backs it). Rolldown's dev
        // banners mention the source path in comments, so check for a real
        // import/from statement rather than any substring occurrence.
        assert!(
            !js.contains("import './widget.compiled.js'")
                && !js.contains("from './widget.compiled.js'"),
            "synthetic .compiled.js inlined, not left as a bare import"
        );
        // The compiled view's body is present (proves the .sui was compiled in,
        // not stubbed) — the `w` class from `<div class=\"w\">` survives.
        assert!(js.contains("\"w\"") || js.contains("'w'"), "compiled .sui body inlined");
        // The compiled view pulls the runtime — it must stay EXTERNAL (resolved
        // to /_sui/runtime.js at runtime), never a second inlined copy.
        assert!(js.contains("@ship/sui-runtime"), "runtime kept external, single instance");
    }
}
