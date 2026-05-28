//! Deck `app` build target (#170) — bundles a SUI SPA app (the hub,
//! `apps/shell`, and the space scaffold, `apps/spaces`) to a static dir.
//!
//! This is the deck-owned replacement for the retired `shell-app.ts`
//! (`Bun.build`) — one bundler engine (Rolldown) for framework + views +
//! apps. Driven by `deck build` (`crates/deck/deck-cli`). The app's `.sui`
//! sources are
//! pre-compiled to `.compiled.js` on disk by `sui-build` before this runs, so
//! the graph is plain JS/TS; we only bundle + externalize the framework
//! singletons (which resolve to `/_sui/*` via the document's importmap).
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
const FRAMEWORK_ASSETS: [(&str, &str); 5] = [
    ("sui-runtime.js", "_sui/runtime.js"),
    ("sui-dom.js", "_sui/dom.js"),
    ("sui-sdk-client.js", "_sui/sdk-client.js"),
    ("sui-components.js", "_sui/components.js"),
    ("lucide-icons.json", "_sui/lucide-icons.json"),
];

pub struct AppBuild {
    pub out_dir: PathBuf,
    pub js_href: String,
    pub css_href: String,
    pub framework_assets: usize,
}

/// Externalizes exactly the document's importmap specifiers, so the bundle
/// leaves `@ship/sui-runtime` (etc.) as bare imports the browser resolves to
/// `/_sui/*` — the single-runtime guarantee. No `.sui` compilation here (the
/// app graph is pre-compiled by `sui-build`).
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
pub async fn bundle_app(
    repo_root: &Path,
    shell_root: &Path,
    out_dir: &Path,
    minify: bool,
    bundle_framework: bool,
    framework_assets_dir: &Path,
    deck_out_dir: &Path,
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
    let plugin = AppPlugin { externals };
    let mut bundler = BundlerBuilder::default()
        .with_options(options)
        .with_plugins(vec![Arc::new(plugin)])
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
        std::fs::create_dir_all(sui_dir.join("sui-components"))?;
        for (src, dist_rel) in FRAMEWORK_ASSETS {
            let from = framework_assets_dir.join(src);
            if !from.exists() {
                bail!("app build: bundle_framework: missing framework asset {}", from.display());
            }
            std::fs::copy(&from, out_dir.join(dist_rel))?;
            framework_assets += 1;
        }
        let islands = deck_out_dir.join("sui-components");
        if !islands.exists() {
            bail!(
                "app build: bundle_framework: no {} (run the bundler / sui-build first)",
                islands.display()
            );
        }
        for entry in std::fs::read_dir(&islands)? {
            let entry = entry?;
            let name = entry.file_name();
            if entry.file_type()?.is_file() && name.to_string_lossy().ends_with(".js") {
                std::fs::copy(entry.path(), sui_dir.join("sui-components").join(&name))?;
                framework_assets += 1;
            }
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
        let build = bundle_app(r, r, &out, false, false, r, r).await.expect("bundle_app");

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
}
