//! Deck `app` build target (#170) — bundles a SUI SPA project (the hub
//! `apps/hub/`, the space scaffold `apps/spaces/`, or any other host
//! that uses the `routes/` convention) to a static dir.
//!
//! This is the deck-owned replacement for the retired `shell-app.ts`
//! (`Bun.build`) — one bundler engine (Rolldown) for framework + views +
//! projects. Driven by `deck build` (`crates/deck/deck-cli`). The project
//! graph compiles its own `.sui` in-process via the shared `SuiPlugin`
//! (`rolldown_bundle.rs`) — the exact resolve/load hooks the view/component
//! build uses — so no on-disk `.compiled.js` pre-step is needed.
//! `AppPlugin` runs alongside it to externalize the framework singletons
//! (which resolve to `/_deck/*` via the document's importmap); it runs FIRST so
//! importmap specifiers (incl. `@ship/sui-runtime/*` subpaths) stay external
//! rather than being re-anchored to source by `SuiPlugin`.
//!
//! The dev's authored source is THREE files only: `package.json` (the
//! manifest: name → page <title>), `routes/` (file-routed chrome), and
//! `public/` (static assets). The bundler generates the rest: `index.html`
//! (from a template, with the importmap substituted), `routes.gen.ts` (the
//! entry that `index.html` references), and the bundled entry chunk.
//! Project builds live at `<project>/.deck/` by default.

use std::borrow::Cow;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use rolldown::plugin::{
    HookResolveIdArgs, HookResolveIdOutput, HookResolveIdReturn, HookUsage, Plugin, PluginContext,
};
use rolldown::{BundlerBuilder, BundlerOptions, InputItem, OutputFormat};
use rolldown_common::{ChunkFilenamesOutputOption, ModuleType, ResolvedExternal};

/// deck's BAKED framework singletons — assets deck itself owns and ships
/// in every app build. Each entry is `(package_alias, source_filename,
/// output_url_relative_to_project)`. The bundler copies `source_filename`
/// from the workspace's prebuilt framework dir to
/// `output_url_relative_to_project` in the project's output, then emits
/// an importmap entry mapping `package_alias → /output_url_relative_to_project`.
///
/// Scope of this list: deck-owned runtime (the SUI runtime + DOM bindings
/// + the router the framework owns). What is NOT here: any framework
/// CLIENT (`@ship/sdk-client`, etc.). Framework clients are per-app
/// registrations the HOST supplies through `AppServeConfig::framework_clients`
/// (see `crates/deck/deck-config`). The bundler reads the host's
/// `deck.jsonc` and merges those entries into the importmap + asset-copy
/// pipeline alongside this list. **Deck hardcodes no ship path**; the
/// host binds its own concrete inside the impl.
///
/// Lucide icons are NOT here — the daemon serves them per-icon from its
/// embedded set at `/_deck/icons/{name}.svg`. Static-only deploys without
/// a daemon use `<sui-icon src=…>` (CDN escape hatch).
///
/// The package_alias MAY include a subpath: `@ship/sui-runtime/route` etc.
/// All aliases for one source bundle point at the same output URL — the
/// runtime exports a single module entry. (e.g. `@ship/sui-runtime/route`,
/// `@ship/sui-runtime/template-hooks`, `@ship/sui-runtime/patch` all share
/// the runtime bundle.)
const FRAMEWORK_ASSETS: &[(&str, &str, &str)] = &[
    ("@ship/sui-runtime", "sui-runtime.js", "_deck/runtime.js"),
    ("@ship/sui-runtime/route", "sui-runtime.js", "_deck/runtime.js"),
    ("@ship/sui-runtime/template-hooks", "sui-runtime.js", "_deck/runtime.js"),
    ("@ship/sui-runtime/patch", "sui-runtime.js", "_deck/runtime.js"),
    ("@ship/sui-dom", "sui-dom.js", "_deck/dom.js"),
    // `@ship/deck/route` lives in the framework because the bundler
    // generates an entry that imports it. The runtime exports
    // `defineRoutes`, `route`, `navigate`, `setEditMode` — every app's
    // entry needs this singleton. The bundle file is built in
    // `crates/deck/deck-core/build.rs` and shipped via
    // `include_str!` in `assets.rs`.
    ("@ship/deck/route", "deck-route.js", "_deck/route.js"),
];

/// Generate the importmap JSON. The keys are the package aliases (deck's
/// `FRAMEWORK_ASSETS` singletons + every host-registered client the per-app
/// `AppServeConfig::framework_clients` map declares); the values are the
/// absolute URL the host serves each bundle at (`/<dist_rel>` for the
/// project's output, which shipd serves at the project root).
///
/// Host-registered clients ride the SAME `_deck/<dist_basename>` URL
/// convention as the framework singletons: the host declares an alias
/// (e.g. `@ship/sdk-client`) + a `dist` path (e.g. `_deck/sdk-client.js`)
/// and the bundler binds the alias to `/_deck/sdk-client.js` — so the
/// daemon's static-asset handler serves client assets on the same code
/// path as the runtime/dom/route singletons. No special-case route; just
/// one URL convention, one handler.
///
/// Output is pretty-printed with 2-space indent to match the existing
/// `apps/shell/index.html` style.
fn generate_importmap(
    dist_prefix: &str,
    framework_clients: &std::collections::BTreeMap<String, deck_config::FrameworkClientEntry>,
) -> String {
    let mut imports: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    for (alias, _src, dist_rel) in FRAMEWORK_ASSETS {
        imports.insert((*alias).to_string(), format!("{dist_prefix}/{dist_rel}"));
    }
    // Host-registered clients: `<alias>` → `/_deck/<basename>`.
    // Same prefix as FRAMEWORK_ASSETS so the daemon's existing static
    // handler serves them on the same code path; the `dist` field's
    // basename is what the importmap binds.
    for (alias, entry) in framework_clients {
        let basename = std::path::Path::new(&entry.dist)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&entry.dist);
        imports.insert(alias.clone(), format!("{dist_prefix}/_deck/{basename}"));
    }
    let body = imports
        .iter()
        .map(|(k, v)| format!("        \"{k}\": \"{v}\""))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("{{\n      \"imports\": {{\n{body}\n      }}\n    }}")
}

/// Extract the bare specifier set from a rendered importmap JSON. The dev
/// doesn't author the importmap; the bundler writes it via `generate_importmap`
/// and reads it back here to get the externals list. The two views must agree:
/// any package the importmap resolves to `/foo` is a runtime external; the
/// view bundle's ESM imports of that package stay as bare specifiers.
fn importmap_externals_from_json(importmap_json: &str) -> Result<HashSet<String>> {
    let v: serde_json::Value = serde_json::from_str(importmap_json)
        .context("generated importmap is not valid JSON")?;
    let imports = v
        .get("imports")
        .and_then(|x| x.as_object())
        .context("generated importmap has no `imports` object")?;
    if imports.is_empty() {
        bail!("generated importmap has no imports");
    }
    Ok(imports.keys().cloned().collect())
}

pub struct AppBuild {
    pub out_dir: PathBuf,
    pub js_href: String,
    /// Absolute URL paths for every CSS asset emitted under
    /// `<out_dir>/assets/` by Rolldown's native asset pipeline
    /// (see `module_types: { ".css" -> Asset }` above). The bundler
    /// injects a `<link rel="stylesheet">` for each one into the
    /// generated `index.html`. Empty when the project imports no CSS.
    pub css_hrefs: Vec<String>,
    pub framework_assets: usize,
}

/// Externalizes the framework singleton specifiers declared in
/// FRAMEWORK_ASSETS. The bundler generates the importmap at build time
/// (from the same FRAMEWORK_ASSETS list); the AppPlugin runs FIRST so
/// the importmap specifiers win as externals rather than being re-anchored
/// to source by `SuiPlugin`. `.sui` compilation is handled by the sibling
/// `SuiPlugin` in the same bundle pass.
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
/// Read the project name from `<project>/package.json#name`. The dev's
/// `package.json` is the manifest: the bundler reads it for the page <title>
/// and the chrome's brand string. Required.
fn read_package_name(project: &Path) -> Result<String> {
    let pkg_path = project.join("package.json");
    let body = std::fs::read_to_string(&pkg_path)
        .with_context(|| format!("read {}", pkg_path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&body)
        .with_context(|| format!("parse {}", pkg_path.display()))?;
    v.get("name")
        .and_then(|n| n.as_str())
        .map(String::from)
        .with_context(|| format!("package.json at {} has no `name` field", project.display()))
}

/// Build a SUI SPA project to `out_dir`. `repo_root` is the workspace root
/// (cwd for module resolution); `project` is the dev's authored source
/// (`package.json` + `routes/` + `public/`). The dev does NOT author
/// `index.html` or any entry script — those are generated from a template
/// and the routes/ walk, respectively.
pub async fn bundle_app(
    repo_root: &Path,
    packs: &[String],
    project: &Path,
    out_dir: &Path,
    minify: bool,
    bundle_framework: bool,
    framework_assets_dir: &Path,
    _deck_out_dir: &Path,
) -> Result<AppBuild> {
    // 0. Load the host's per-app `deck.jsonc` (optional). The host's
    // registration is what wires framework CLIENTS (no framework hardcode)
    // and what configures the host-owned entry bootstrap (`entry.gen.ts`).
    // The file is optional — an app without one still builds, just with
    // no host-registered clients and no `entry.gen.ts` (the old
    // `routes.gen.ts`-as-entry shape).
    let app_cfg_path = project.join("deck.jsonc");
    let app_cfg = if app_cfg_path.exists() {
        let raw = std::fs::read_to_string(&app_cfg_path)
            .with_context(|| format!("read {}", app_cfg_path.display()))?;
        Some(deck_config::AppServeConfig::parse_deck_jsonc(&raw)
            .with_context(|| format!("parse {}", app_cfg_path.display()))?)
    } else {
        None
    };
    let framework_clients: std::collections::BTreeMap<String, deck_config::FrameworkClientEntry> =
        app_cfg.as_ref().map(|c| c.framework_clients.clone()).unwrap_or_default();
    let entry_bootstrap: Option<deck_config::EntryBootstrap> =
        app_cfg.as_ref().and_then(|c| c.entry.clone());

    // 1. Generate the route table from `<project>/routes/`. The dev doesn't
    // author a route config — the routes/ tree IS the source. The bundler
    // emits `routes.gen.ts` to the same dir.
    let routes_spec = crate::routes::RouteGenSpec {
        dir: project.join("routes"),
        out: project.join("routes.gen.ts"),
        label: project
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("app")
            .to_string(),
        // Plumb the host's entry bootstrap into the route-table spec so
        // `generate_routes` ALSO emits `entry.gen.ts` (when declared) and
        // keeps `routes.gen.ts` data-only. Without this, the data file
        // would re-acquire the router call + token import.
        entry_bootstrap: entry_bootstrap.clone(),
    };
    crate::routes::generate_routes(&[routes_spec]).context("generate routes.gen.ts")?;

    // 1b. Component catalog for this app: aggregate CEM modules from component
    // pack deps and emit `components.gen.ts`. No-op when the project has no
    // `package.json` dependencies on component packs.
    crate::cem::generate_components_gen(project, repo_root)
        .context("generate components.gen.ts")?;

    // 2. Read the project name from `package.json#name`. The dev's
    // npm name is the app's identity; the URL namespace is the
    // last `/`-segment (so `@ship/hub` → `hub`). One source of
    // truth: `package.json#name`.
    let project_name = read_package_name(project)
        .with_context(|| format!("read package.json at {}", project.display()))?;
    let url_namespace = project_name
        .rsplit_once('/')
        .map(|(_, tail)| tail)
        .unwrap_or(&project_name)
        .to_string();
    // 3. Generate the importmap. The framework serves the runtime singletons
    // at /_deck/* — the workspace root. Host-registered clients ride the
    // same `/_deck/<basename>` URL convention so the daemon's static-asset
    // handler serves them on the same code path as the framework singletons.
    // No per-app dist prefix; the app's name is its URL namespace.
    let importmap_json = generate_importmap("", &framework_clients);
    let externals = importmap_externals_from_json(&importmap_json)?;
    // 4. Render `index.html` from the template. The dev doesn't author
    // index.html — it's deck's entry contract. The template has placeholders
    // for the project name, the importmap, and the entry module name.
    // The entry module is `entry.gen.ts` when the host declares one (the
    // `entry.bootstrap` shape in `deck.jsonc`); otherwise fall back to the
    // historical `routes.gen.ts`-as-entry shape so legacy builds keep working.
    let entry_module = if entry_bootstrap.is_some() {
        "entry.gen.ts"
    } else {
        "routes.gen.ts"
    };
    let html = crate::templates::render_index_html(
        &project_name,
        &importmap_json,
        entry_module,
    );
    let base = out_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if out_dir.exists() && (base == "dist" || out_dir.to_string_lossy().contains("/.deck")) {
        std::fs::remove_dir_all(out_dir).ok();
    }
    let assets_dir = out_dir.join("assets");
    std::fs::create_dir_all(&assets_dir)?;

    // 5. Bundle the generated entry as the SPA entry. The dev's source
    // tree has no `main.ts`; the bundler generates it from the
    // routes/ walk (data only) plus, when the host declares one, a
    // host-owned `entry.gen.ts` (client wiring + mountRouter call).
    // AppPlugin runs FIRST so importmap specifiers (incl. `@ship/sui-runtime/*`
    // subpaths and every host-registered client alias) win as externals;
    // SuiPlugin then compiles `.sui` siblings and re-anchors the remaining
    // `@ship/<pkg>` (e.g. sui-charts, pipelines) to workspace source so the
    // app graph compiles them in-process.
    let routes_gen = project.join(entry_module);
    if !routes_gen.exists() {
        bail!("app build: expected generated entry at {}", routes_gen.display());
    }
    // CSS ships as a hashed asset via Rolldown's native `ModuleType::Asset`
    // path (`rolldown_plugin_asset_module`). The `SuiPlugin::load` hook only
    // handles `.sui` / `.compiled.js` and returns `Ok(None)` for everything
    // else, so the built-in `AssetModulePlugin` runs and emits a hashed
    // `assets/<name>-<hash>.css` file. The chunk references it via a
    // placeholder that `render_chunk` resolves to the asset filename;
    // we then lift that link into the generated `index.html` below.
    // (Rolldown 1.0 removed `ModuleType::Css` — issue rolldown/rolldown#4271;
    // the asset type is the supported way to ship a CSS file.)
    let mut module_types: rustc_hash::FxHashMap<String, ModuleType> =
        rustc_hash::FxHashMap::default();
    module_types.insert(".css".to_string(), ModuleType::Asset);
    let options = BundlerOptions {
        input: Some(vec![InputItem {
            name: Some("entry".to_string()),
            import: routes_gen.to_string_lossy().to_string(),
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
        module_types: Some(module_types),
        ..Default::default()
    };
    let app_plugin = AppPlugin { externals };
    let resolver = crate::sui_plugin::component_index(repo_root, packs);
    let sui_plugin = crate::sui_plugin::SuiPlugin {
        repo_root: repo_root.to_path_buf(),
        emit_component_imports: true,
        component_index: resolver.index,
        shadowed_tags: resolver.shadowed,
        component_roots: crate::catalog::component_package_roots(repo_root),
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
    // 6. The entry chunk is `assets/entry-<hash>.js` (input name "entry").
    // The script href in the generated HTML is the final URL — the
    // bundler is the only place that knows the project name (the
    // framework serves the file at `/<name>/<file>`). No string
    let js_name = std::fs::read_dir(&assets_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|n| n.starts_with("entry-") && n.ends_with(".js"))
        .context("app build: no entry chunk assets/entry-*.js emitted")?;
    let js_href = format!("/{url_namespace}/{js_name}");
    // 6b. Discover every CSS asset Rolldown emitted under `assets/`. The
    // `AssetModulePlugin` writes each `.css` import as `assets/<name>-<hash>.css`
    // (content-hashed; default `asset_filenames = "assets/[name]-[hash][extname]"`).
    // We link them directly from `index.html` — Vite-style: the SPA entry is JS,
    // but global / token / shared CSS is a stylesheet the browser fetches in
    // parallel, before the JS runs. The SuiPlugin's `load` hook returns `Ok(None)`
    // for `.css`, so the builtin `AssetModulePlugin` handles it (it runs AFTER
    // user plugins via `PluginHookMeta.order = Post`).
    let mut css_names: Vec<String> = std::fs::read_dir(&assets_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".css"))
        .collect();
    css_names.sort();
    let css_hrefs: Vec<String> = css_names
        .iter()
        .map(|n| format!("/{url_namespace}/{n}"))
        .collect();
    let public_dir = project.join("public");
    if public_dir.exists() {
        for entry in std::fs::read_dir(&public_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                std::fs::copy(entry.path(), out_dir.join(entry.file_name()))?;
            }
        }
    }
    // 8. Write the rendered index.html (the dev doesn't author it; the
    // template was rendered above in step 4). The script tag references
    // the entry chunk; CSS assets (if any) get a `<link rel="stylesheet">`
    // injected BEFORE the entry script so the browser fetches and applies
    // them in parallel with the module load — same first-paint guarantee
    // the per-view synthetic CSS render lifts (orthogonal layer — this is
    // the GLOBAL/linkable CSS path; the view-synthetic one handles view
    // chrome inline in SSR/CSR first-paint).
    let mut final_html = html.replace(
        &format!("/{entry_module}"),
        &js_href,
    );
    if !css_hrefs.is_empty() {
        let link_tags = css_hrefs
            .iter()
            .map(|href| format!("  <link rel=\"stylesheet\" href=\"{href}\">"))
            .collect::<Vec<_>>()
            .join("\n");
        // Anchor immediately before the entry `<script type="module">`. The
        // template renders `<script type="module" src="/<entry_module>">`
        // first as a relative substitute; after the `.replace()` above it
        // carries the final hashed href.
        final_html = final_html.replace(
            "  <script type=\"module\"",
            &format!("{link_tags}\n  <script type=\"module\""),
        );
    }
    std::fs::write(out_dir.join("index.html"), final_html)?;
    let mut framework_assets = 0;
    if bundle_framework {
        let sui_dir = out_dir.join("_deck");
        std::fs::create_dir_all(&sui_dir)?;
        // Deck-owned framework singletons. De-dup by source filename —
        // multiple aliases (e.g. `@ship/sui-runtime`, `@ship/sui-runtime/route`)
        // share one source file. Track by source path to skip repeats.
        let mut copied_sources: std::collections::HashSet<String> = Default::default();
        for (_alias, src, dist_rel) in FRAMEWORK_ASSETS {
            let from = framework_assets_dir.join(src);
            if !from.exists() {
                bail!("app build: bundle_framework: missing framework asset {}", from.display());
            }
            let dst = out_dir.join(dist_rel);
            if dst.exists() || !copied_sources.insert(src.to_string()) {
                continue;
            }
            std::fs::copy(&from, dst)?;
            framework_assets += 1;
        }
        // Host-registered framework clients. `from` is workspace-relative
        // (e.g. `.deck/framework/sui-sdk-client.js`); `dist` is the
        // project's `<out>/<dist>` (e.g. `_deck/sdk-client.js`). The
        // importmap binds the alias to the `dist` basename — copied below.
        for (alias, entry) in &framework_clients {
            let from_path = std::path::Path::new(&entry.from);
            // Workspace-relative paths (`from`) are anchored at the repo root.
            // An absolute path is honored as-is so a host can pull a
            // pre-built client from anywhere on disk.
            let from = if from_path.is_absolute() {
                from_path.to_path_buf()
            } else {
                repo_root.join(from_path)
            };
            if !from.exists() {
                bail!(
                    "app build: bundle_framework: missing framework client `{}` (from {})",
                    alias,
                    from.display()
                );
            }
            let dst = out_dir.join(&entry.dist);
            if dst.exists() {
                continue;
            }
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&from, &dst)?;
            framework_assets += 1;
        }
    }
    // (formerly refreshed the runtime catalog by writing
    // `<repo_root>/.deck/catalogs/workspace.json`) — purged:
    // the framework no longer writes a monorepo-root `.deck/catalogs/`
    // (deck-purge-root). The catalog walker
    // (`crate::catalog::build_catalog`) stays available for hosts that
    // want to compute the catalog in-memory for their own devtools; it
    // just does not get written to disk by the framework.
    let _ = repo_root;

    Ok(AppBuild {
        out_dir: out_dir.to_path_buf(),
        js_href,
        css_hrefs,
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

    /// The dev authors `package.json`, `routes/`, `public/`. The bundler
    /// generates `index.html` (with the importmap), `routes.gen.ts` (the
    /// entry), the entry chunk, and copies `public/`. Behavioral
    /// assertions, not text matching — a future index.html template
    /// change shouldn't break this test.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_generates_index_and_routes_gen() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"test-app","private":true,"type":"module"}"#,
        );
        // A GENERIC dynamic route — deliberately not any app's real route, so
        // this framework test never couples to a specific app's IA. The
        // emitter statically imports + compiles each route view, so the
        // fixture must compile: `id` is bound by the sibling controller.
        write(&r.join("routes/widgets/[id]/page.sui"), "<div>{id()}</div>\n");
        write(
            &r.join("routes/widgets/[id]/controller.ts"),
            "import { controller, signal } from '@ship/sui-runtime'\n\
export default controller({ scope: () => ({ id: signal('demo') }) })\n",
        );
        write(&r.join("public/favicon.ico"), "ICO");

        let out = r.join("dist");
        let build = bundle_app(r, &[], r, &out, false, false, r, r)
            .await
            .expect("bundle_app");

        // Behavior 1: the dev's `index.html` doesn't exist; the bundler
        // generated one at `<out>/index.html`.
        assert!(!r.join("index.html").exists(), "dev did not author index.html");
        assert!(out.join("index.html").exists(), "bundler generated index.html");
        // The generated HTML is valid and references the entry chunk.
        let html = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains(&build.js_href), "html references bundled entry chunk");

        // Behavior 2: the entry is the generated `routes.gen.ts` (NOT a
        // dev-authored `src/main.ts`). The bundled output references the
        // routes directory's `page.sui` pattern.
        assert!(!r.join("src/main.ts").exists(), "dev did not author main.ts");
        // The bundler generated the entry, and the dynamic route bundled —
        // bundle_app() above would have errored if the route view failed to
        // compile into the graph. We do NOT string-match the generated TS:
        // that's presentation, and the router's behavior is covered by
        // @ship/deck/route's mount-router tests.
        assert!(r.join("routes.gen.ts").exists(), "bundler generated the routes.gen.ts entry");

        // Behavior 3: the framework singletons are externalized in the
        // bundle (the importmap is the runtime's source of truth, not
        // the bundler — but the importmap must at least NAME the assets).
        assert!(html.contains("@ship/sui-runtime"), "framework runtime in importmap");
        // Deck must NOT hardcode any framework CLIENT (no `@ship/sdk-client`).
        // Framework clients are per-app registrations the host supplies
        // through `AppServeConfig::framework_clients`. A default build
        // (no host deck.jsonc) emits NO client alias — the framework
        // names no ship path.
        assert!(
            !html.contains("@ship/sdk-client"),
            "framework must NOT hardcode `@ship/sdk-client` — host-registered (got: {html})",
        );

        // Behavior 4: the entry chunk href is the final served path.
        // The bundler writes `/<url_namespace>/entry-<hash>.js` so
        // the framework can serve it at the corresponding route
        // without a string rewrite. The URL namespace is the last
        // `/`-segment of `package.json#name` (`@ship/hub` → `hub`).
        assert!(build.js_href.starts_with("/test-app/entry-") && build.js_href.ends_with(".js"),
                "expected /test-app/entry-*.js, got {}", build.js_href);
        // that knows the project name; the framework serves the
        // file at `/<name>/<file>`.
        assert!(build.js_href.starts_with("/test-app/entry-") && build.js_href.ends_with(".js"),
                "expected /test-app/entry-*.js, got {}", build.js_href);
        // Behavior 5: public/ assets are copied to out/.
        assert!(out.join("favicon.ico").exists(), "public/ copied");
    }

    /// The dev doesn't write a main.ts. The entry is `routes.gen.ts`,
    /// which the bundler generates. A `.sui` file inside `routes/`
    /// must compile IN-PROCESS (no `.compiled.js` artifact) and be
    /// available to the bundle. A failure to compile would surface as
    /// a panic from the bundler, not as a missing chunk.
    #[tokio::test(flavor = "multi_thread")]
    async fn app_build_compiles_sui_in_process() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"test-app","private":true,"type":"module"}"#,
        );
        write(
            &r.join("routes/[name]/page.sui"),
            "<div class=\"w\">hi {1 + 1}</div>\n",
        );

        let out = r.join("dist");
        bundle_app(r, &[], r, &out, false, false, r, r)
            .await
            .expect("bundle_app should compile the .sui entry in-process");
    }

    /// When a deck app's module graph imports a `.css` file, the per-app
    /// `bundle_app` builds it through Rolldown's NATIVE asset path
    /// (`module_types: { ".css" -> Asset }` → `AssetModulePlugin`), emits
    /// a content-hashed file under `<out>/assets/`, and lifts the link
    /// into the generated `index.html` so the browser fetches the
    /// stylesheet in parallel with the SPA entry module.
    ///
    /// Gate per `.artifacts/job-spec.md`: a CSS file imported by the entry
    /// MUST end up as a hashed `.css` under `assets/` AND `<link
    /// rel="stylesheet" href="/<ns>/<file>.css">` MUST appear in the
    /// generated HTML — same URL convention the entry JS uses
    /// (`/<namespace>/<file>`), so the daemon's `serve_app_asset`
    /// (`GET /<app>/{file}`) serves it.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_emits_and_links_css_assets() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"css-test-app","private":true,"type":"module"}"#,
        );
        // A minimal route so the entry isn't empty. The CSS import goes
        // on the sibling controller so it reaches the entry graph (the
        // entry is `routes.gen.ts`, which statically imports this).
        write(
            &r.join("routes/page.sui"),
            "<div class=\"css-probe\">css</div>\n",
        );
        write(
            &r.join("routes/controller.ts"),
            "import './styles.css';\n\
             import { controller } from '@ship/sui-runtime'\n\
             export default controller({ scope: () => ({}) })\n",
        );
        write(
            &r.join("routes/styles.css"),
            ".css-probe { color: rebeccapurple; }\n",
        );

        let out = r.join("dist");
        let build = bundle_app(r, &[], r, &out, false, false, r, r)
            .await
            .expect("bundle_app");

        // Gate 1: at least one hashed `.css` asset emitted under
        // `<out>/assets/`. The default Rolldown asset filename is
        // `assets/[name]-[hash][extname]` → `assets/styles-<hash>.css`.
        let css_on_disk: Vec<String> = std::fs::read_dir(out.join("assets"))
            .expect("assets dir emitted")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".css"))
            .collect();
        assert!(
            !css_on_disk.is_empty(),
            "expected a hashed .css asset under <out>/assets/, got none"
        );
        assert!(
            css_on_disk.iter().all(|n| n.starts_with("styles-")),
            "CSS asset should be content-hashed off the source stem (got {css_on_disk:?})"
        );

        // Gate 2: AppBuild exposes the absolute URLs the index.html link
        // references — one per emitted file, all under `/<namespace>/`.
        assert_eq!(build.css_hrefs.len(), css_on_disk.len(), "css_hrefs matches disk");
        for href in &build.css_hrefs {
            assert!(
                href.starts_with("/css-test-app/styles-") && href.ends_with(".css"),
                "css_href uses the same URL convention as the entry (/<ns>/<file>.css), got {href}"
            );
        }

        // Gate 3: the generated `index.html` links the CSS before the
        // entry `<script type="module">`. Order matters — the stylesheet
        // must apply BEFORE first paint, in parallel with module fetch.
        let html = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(
            html.contains("<link rel=\"stylesheet\""),
            "index.html must link the CSS asset"
        );
        for href in &build.css_hrefs {
            assert!(
                html.contains(href),
                "index.html must reference {href}"
            );
        }
        // The link must precede the entry script (link is fetched + applied
        // before module eval; same first-paint guarantee the view-synthetic
        // CSS render lifts — but for GLOBAL/linkable CSS, not view chrome).
        let link_pos = html.find("<link rel=\"stylesheet\"").unwrap();
        let script_pos = html.find("<script type=\"module\"").unwrap();
        assert!(
            link_pos < script_pos,
            "<link rel=stylesheet> must precede <script type=module>; got link@{link_pos} script@{script_pos}"
        );

        // Belt-and-braces: exactly one `<link rel="stylesheet">` per
        // emitted CSS asset (no synthesized empty links when nothing was
        // imported — the existing `bundle_app_generates_index_and_routes_gen`
        // covers the zero-import path implicitly).
        assert_eq!(
            html.matches("<link rel=\"stylesheet\"").count(),
            css_on_disk.len(),
            "exactly one <link rel=stylesheet> per emitted CSS asset"
        );
        // Belt-and-braces: exactly one `<link rel="stylesheet">` per
        // emitted CSS asset (no synthesized empty links when nothing was
        // imported — the existing `bundle_app_generates_index_and_routes_gen`
        // covers the zero-import path implicitly).
        assert_eq!(
            html.matches("<link rel=\"stylesheet\"").count(),
            css_on_disk.len(),
            "exactly one <link rel=stylesheet> per emitted CSS asset"
        );
    }

    /// Gate per `.artifacts/job-spec.md`: a freshly built sui app's
    /// `index.html` links the design-token stylesheet AND the emitted CSS
    /// under `assets/` defines `:root` tokens (e.g. `--background`). The
    /// token CSS enters the entry graph by convention: the generated
    /// `routes.gen.ts` imports `@ship/tokens/tokens.css` and Rolldown's
    /// native `AssetModulePlugin` bundles + links it. A static-only
    /// deploy (no daemon, no `view-tokens.css` serve route) has tokens.
    ///
    /// Mirrors the per-app build proof in
    /// `bundle_app_emits_and_links_css_assets` (the test it supersedes for
    /// the token path): the project must NOT hand-author the import —
    /// `routes.gen.ts` is generated, the import lands via the entry
    /// contract, and `var(--token)` resolves in the built bundle.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_bundles_design_tokens_via_entry() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        // Minimal workspace: the app project + the `@ship/tokens` package
        // that exports `./tokens.css`. `component_package_roots` walks the
        // workspace to build the `@ship/<name>` → source-dir map that
        // `SuiPlugin::resolve_id` uses to re-anchor the bare specifier.
        // Without this map the bundler cannot find the CSS, and the gate
        write(
            &r.join("package.json"),
            r#"{"name":"tokens-app","private":true,"type":"module",
                "workspaces":["packages/sui/*"]}"#,
        );
        write(
            &r.join("packages/sui/tokens/package.json"),
            r#"{"name":"@ship/tokens","type":"module",
                "exports":{".":"./src/index.ts","./tokens.css":"./src/tokens.css"}}"#,
        );
        // Bare-minimum tokens.css: a single `:root` block with the
        // `--background` token the spec names as the gate's exemplar.
        // The gate asserts both presence (the convention) and shape
        // (a real `:root` block with the named token).
        write(
            &r.join("packages/sui/tokens/src/tokens.css"),
            ":root { --background: oklch(20% 0 0); }\n",
        );
        write(
            &r.join("packages/sui/tokens/src/index.ts"),
            "export const __noop = 1;\n",
        );
        // Minimal route so the entry isn't empty. The token import goes
        // through the generated `routes.gen.ts` (not a hand-authored
        // controller) — that's the whole point of the gate.
        write(
            &r.join("routes/page.sui"),
            "<div class=\"token-probe\">tok</div>\n",
        );
        let out = r.join("dist");
        let build = bundle_app(r, &[], r, &out, false, false, r, r)
            .await
            .expect("bundle_app");
        // Gate 1: a hashed `tokens-<hash>.css` asset is emitted under
        // `<out>/assets/`. The default Rolldown asset filename is
        // `assets/[name]-[hash][extname]`. The stem comes from the source
        // file (`packages/sui/tokens/src/tokens.css` → `tokens-<hash>.css`).
        let css_on_disk: Vec<String> = std::fs::read_dir(out.join("assets"))
            .expect("assets dir emitted")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".css"))
            .collect();
        assert!(
            css_on_disk.iter().any(|n| n.starts_with("tokens-")),
            "expected a tokens-<hash>.css asset under <out>/assets/ (got: {css_on_disk:?})",
        );
        // Gate 2: the build exposes the absolute URL the link references
        // — `/<namespace>/<file>.css` — and the generated `index.html`
        // links it. Same URL convention as the entry JS, so the daemon's
        // `serve_app_asset` (`GET /<app>/{file}`) serves it for dev.
        assert!(
            build.css_hrefs.iter().any(|h| h.starts_with("/tokens-app/tokens-") && h.ends_with(".css")),
            "css_hrefs must include the bundled tokens asset, got: {:?}",
            build.css_hrefs,
        );
        let html = std::fs::read_to_string(out.join("index.html")).unwrap();
        for href in &build.css_hrefs {
            assert!(
                html.contains(href),
                "index.html must reference the bundled tokens href {href}",
            );
        }
        // The link must precede the entry script (parallel fetch, before
        // first paint — the same guarantee the merged CSS-asset path
        // establishes for hand-authored CSS).
        let link_pos = html.find("<link rel=\"stylesheet\"").expect("a <link rel=stylesheet> exists");
        let script_pos = html.find("<script type=\"module\"").expect("entry script exists");
        assert!(
            link_pos < script_pos,
            "token <link rel=stylesheet> must precede <script type=module>; got link@{link_pos} script@{script_pos}",
        );

        // Gate 3: the emitted token CSS defines `:root` tokens. The
        // presence of `--background` is the spec's named exemplar (it
        // resolves in every component that uses `var(--background)`).
        let tokens_path = out
            .join("assets")
            .join(css_on_disk.iter().find(|n| n.starts_with("tokens-")).unwrap());
        let tokens_css = std::fs::read_to_string(&tokens_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", tokens_path.display()));
        assert!(
            tokens_css.contains(":root"),
            "emitted tokens CSS must define a :root block (got {} bytes)",
            tokens_css.len(),
        );
        assert!(
            tokens_css.contains("--background:"),
            "emitted tokens CSS must define the --background role token (the spec's named exemplar)",
        );
    }

    /// Host-registered framework clients: when the per-app `deck.jsonc`
    /// declares `frameworkClients[<alias>]`, the bundler:
    ///   1. emits an importmap entry for the alias → `/_deck/<dist-basename>`,
    ///   2. copies the source asset (`from`) to `<out>/<dist>` when
    ///      `--bundle-framework` is set.
    /// The framework itself never hardcodes any client alias (no
    /// `@ship/sdk-client`, etc.) — every alias is the host's call.
    /// Host-registered framework clients appear in the generated importmap
    /// when the per-app `deck.jsonc` declares `frameworkClients[<alias>]`.
    /// The framework itself never hardcodes any client alias (no
    /// `@ship/sdk-client`, etc.) — every alias is the host's call. This
    /// test runs without `--bundle-framework`: the importmap is emitted
    /// at codegen time (before any framework-asset copy), so the alias
    /// is asserted regardless of bundle mode.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_emits_host_registered_client_importmap() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"client-test-app","private":true,"type":"module"}"#,
        );
        // Host-declared client: alias → from/dist. `from` is workspace-relative.
        // The source asset doesn't need to exist for this test — the importmap
        // is emitted before the framework-asset copy runs, so we only assert
        // the importmap binding here.
        write(
            &r.join("deck.jsonc"),
            r#"{
  "name": "client-test-app",
  "route": "/",
  "source": "client-test-app/.deck",
  "frameworkClients": {
    "@ship/sdk-client": {
      "from": ".deck/framework/sui-sdk-client.js",
      "dist": "_deck/sdk-client.js"
    }
  }
}"#,
        );
        // Minimal route so the entry isn't empty.
        write(&r.join("routes/page.sui"), "<div>x</div>\n");

        let out = r.join("dist");
        bundle_app(r, &[], r, &out, false, false, r, r)
            .await
            .expect("bundle_app");

        // Gate: importmap binds the host-registered alias to `/_deck/<dist-basename>`.
        let html = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(
            html.contains("\"@ship/sdk-client\": \"/_deck/sdk-client.js\""),
            "importmap must bind the host-registered alias to `/_deck/sdk-client.js` (got: {html})",
        );
        // Hard constraint: framework must NEVER hardcode `@ship/sdk-client`
        // as a literal in `FRAMEWORK_ASSETS`. A host-registered alias
        // comes from the host's `deck.jsonc`, not the framework's own list.
        assert!(
            !html.contains("@ship/sdk-client")
                || html.contains("@ship/sdk-client\": \"/_deck/sdk-client.js\""),
            "the only place `@ship/sdk-client` may appear is the host's importmap entry (got: {html})",
        );
    }

    /// `--bundle-framework` ALSO copies the host-registered client asset
    /// (alongside the deck-owned framework singletons) and counts it in
    /// `framework_assets`. This proves the host's client registration
    /// rides the same copy pipeline as `@ship/sui-runtime` etc.
    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_app_copies_host_registered_client_when_bundling() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            &r.join("package.json"),
            r#"{"name":"client-bundle-app","private":true,"type":"module"}"#,
        );
        // Stage the deck-owned framework singletons AND the host's client.
        // `bundle_framework=true` copies every entry to `<out>/<dist>`;
        // without the deck-owned ones staged, the build errors with
        // "missing framework asset sui-runtime.js".
        let framework_dir = r.join("framework");
        std::fs::create_dir_all(&framework_dir).unwrap();
        for name in ["sui-runtime.js", "sui-dom.js", "deck-route.js"] {
            std::fs::write(framework_dir.join(name), "// stub\n").unwrap();
        }
        let host_framework = r.join(".deck/framework");
        std::fs::create_dir_all(&host_framework).unwrap();
        std::fs::write(host_framework.join("sui-sdk-client.js"), "// stub\n").unwrap();
        write(
            &r.join("deck.jsonc"),
            r#"{
  "name": "client-bundle-app",
  "route": "/",
  "source": "client-bundle-app/.deck",
  "frameworkClients": {
    "@ship/sdk-client": {
      "from": ".deck/framework/sui-sdk-client.js",
      "dist": "_deck/sdk-client.js"
    }
  }
}"#,
        );
        write(&r.join("routes/page.sui"), "<div>x</div>\n");

        let out = r.join("dist");
        let build = bundle_app(r, &[], r, &out, false, true, &framework_dir, r)
            .await
            .expect("bundle_app");

        // Gate 1: the host-registered client asset was copied to <out>/<dist>.
        assert!(
            out.join("_deck/sdk-client.js").exists(),
            "`--bundle-framework` must copy the host-registered client asset to <out>/<dist>",
        );
        // Gate 2: framework_assets counter includes the host client (so it
        // shows up in the build log alongside the deck-owned singletons).
        assert!(
            build.framework_assets >= 4,
            "framework_assets must include 3 deck-owned singletons + 1 host client (got {})",
            build.framework_assets,
        );
    }
}
