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
use rolldown_common::{ChunkFilenamesOutputOption, ResolvedExternal};

/// deck's baked framework bundles. Each entry is `(package_alias,
/// source_filename, output_url_relative_to_project)`. The bundler copies
/// `source_filename` from the workspace's prebuilt framework dir to
/// `output_url_relative_to_project` in the project's output, then emits
/// an importmap entry mapping `package_alias → /output_url_relative_to_project`.
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
    ("@ship/sdk-client", "sui-sdk-client.js", "_deck/sdk-client.js"),
    ("@ship/deck/route", "deck-route.js", "_deck/route.js"),
];

/// Generate the importmap JSON from FRAMEWORK_ASSETS. The keys are the
/// package aliases; the values are the absolute URL the host serves the
/// framework bundle at (`/<dist_rel>` for the project's output, which shipd
/// serves at the project root). Output is pretty-printed with 2-space indent
/// to match the existing `apps/shell/index.html` style.
fn generate_importmap(dist_prefix: &str) -> String {
    let mut imports: std::collections::BTreeMap<&str, String> =
        std::collections::BTreeMap::new();
    for (alias, _src, dist_rel) in FRAMEWORK_ASSETS {
        imports.insert(alias, format!("{dist_prefix}/{dist_rel}"));
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
    component_source_roots: &[PathBuf],
    project: &Path,
    out_dir: &Path,
    minify: bool,
    bundle_framework: bool,
    framework_assets_dir: &Path,
    _deck_out_dir: &Path,
) -> Result<AppBuild> {
    // 1. Generate the route table from `<project>/routes/`. The dev doesn't
    // author a route config — the routes/ tree IS the source. The bundler
    // emits `routes.gen.ts` to the same dir; that file is the entry that
    // `index.html` (also generated) references.
    let routes_spec = crate::routes::RouteGenSpec {
        dir: project.join("routes"),
        out: project.join("routes.gen.ts"),
        label: project
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("app")
            .to_string(),
    };
    crate::routes::generate_routes(&[routes_spec]).context("generate routes.gen.ts")?;

    // 2. Read the project name from `package.json#name`. Used as the page
    // <title> and the chrome's brand string. Required: the template
    // substitutes it in.
    let project_name = read_package_name(project)
        .with_context(|| format!("read package.json at {}", project.display()))?;

    // 3. Generate the importmap from FRAMEWORK_ASSETS. The dist_prefix is
    // the URL prefix the runtime serves the project's output at (empty for
    // the project root, e.g. `/space` for the space scaffold). Default
    // is empty (project served at the daemon root).
    let dist_prefix = std::env::var("DECK_DIST_PREFIX").unwrap_or_default();
    let importmap_json = generate_importmap(&dist_prefix);
    let externals = importmap_externals_from_json(&importmap_json)?;

    // 4. Render `index.html` from the template. The dev doesn't author
    // index.html — it's deck's entry contract. The template has placeholders
    // for the project name, the importmap, and the entry module name.
    let entry_module = "routes.gen.ts";
    let html = crate::templates::render_index_html(
        &project_name,
        &importmap_json,
        entry_module,
    );

    // Wipe a conventional output dir so stale hashed chunks don't accumulate.
    let base = out_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if out_dir.exists() && (base == "dist" || out_dir.to_string_lossy().contains("/.deck")) {
        std::fs::remove_dir_all(out_dir).ok();
    }
    let assets_dir = out_dir.join("assets");
    std::fs::create_dir_all(&assets_dir)?;

    // 5. Bundle the generated `routes.gen.ts` as the entry. The dev's source
    // tree has no `main.ts`; the bundler generates the entry from the
    // routes/ walk and bundles it. AppPlugin runs FIRST so importmap
    // specifiers (incl. `@ship/sui-runtime/*` subpaths) win as externals;
    // SuiPlugin then compiles `.sui` siblings and re-anchors the remaining
    // `@ship/<pkg>` (e.g. sui-charts, pipelines) to workspace source so the
    // app graph compiles them in-process.
    let routes_gen = project.join(entry_module);
    if !routes_gen.exists() {
        bail!("app build: expected generated entry at {}", routes_gen.display());
    }
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
        ..Default::default()
    };
    let app_plugin = AppPlugin { externals };
    let resolver = crate::sui_plugin::component_index(repo_root, packs);
    let sui_plugin = crate::sui_plugin::SuiPlugin {
        repo_root: repo_root.to_path_buf(),
        emit_component_imports: true,
        component_index: resolver.index,
        shadowed_tags: resolver.shadowed,
        component_source_roots: Arc::new(component_source_roots.to_vec()),
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
    let js_name = std::fs::read_dir(&assets_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|n| n.starts_with("entry-") && n.ends_with(".js"))
        .context("app build: no entry chunk assets/entry-*.js emitted")?;
    // The asset prefix is the per-app config (e.g. `/hub/_assets/`). The
    // bundler writes assets at `<out_dir>/assets/`; shipd serves the
    // per-app's `out_dir` at the asset_prefix path (see apps.jsonc).
    // The project name is the segment — `apps/hub` → `/hub/_assets/`.
    let project_segment = project
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("app");
    let asset_prefix = format!("/{project_segment}/_assets/");
    let js_href = format!("{asset_prefix}{js_name}");

    // 7. Copy public/ verbatim (favicons, logos referenced by root-absolute href).
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
    // the entry chunk.
    let final_html = html.replace(
        &format!("/{entry_module}"),
        &js_href,
    );
    std::fs::write(out_dir.join("index.html"), final_html)?;

    // 9. Self-contained build: materialize the /_deck/* code surface locally.
    let mut framework_assets = 0;
    if bundle_framework {
        let sui_dir = out_dir.join("_deck");
        std::fs::create_dir_all(&sui_dir)?;
        for (_alias, src, dist_rel) in FRAMEWORK_ASSETS {
            // De-dup by source filename: multiple aliases (e.g.
            // `@ship/sui-runtime`, `@ship/sui-runtime/route`) share one
            // source file. Track by source path to skip repeats.
            let from = framework_assets_dir.join(src);
            if !from.exists() {
                bail!("app build: bundle_framework: missing framework asset {}", from.display());
            }
            let dst = out_dir.join(dist_rel);
            if dst.exists() {
                continue;
            }
            std::fs::copy(&from, dst)?;
            framework_assets += 1;
        }
    }

    // Refresh the runtime catalog so `/_deck/catalog/workspace.json` (served
    // by shipd) carries every entry the new .sui emit just produced. The
    // watcher path (Bundler::build_all) refreshes too; this closes the gap
    // for the CLI `build` subcommand, which calls bundle_app directly.
    match crate::catalog::build_catalog(repo_root) {
        Ok(ws) => {
            let dir = repo_root.join(".deck/catalogs");
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(
                dir.join("workspace.json"),
                crate::catalog::stringify_catalog(&ws),
            );
            tracing::info!(workspace = ws.count, "bundler: catalog refreshed (bundle_app)");
        }
        Err(e) => tracing::warn!(error = %e, "bundler: catalog refresh failed (bundle_app)"),
    }

    Ok(AppBuild {
        out_dir: out_dir.to_path_buf(),
        js_href,
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
        write(
            &r.join("routes/studio/components/[name]/page.sui"),
            "<div>{name()}</div>\n",
        );
        write(&r.join("public/favicon.ico"), "ICO");

        let out = r.join("dist");
        let build = bundle_app(r, &[], &[], r, &out, false, false, r, r)
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
        let routes_gen = std::fs::read_to_string(r.join("routes.gen.ts")).unwrap();
        assert!(routes_gen.contains("defineRoutes"), "entry registers patterns with the router");
        assert!(routes_gen.contains("/studio/components/:name"), "pattern from routes/");

        // Behavior 3: the framework singletons are externalized in the
        // bundle (the importmap is the runtime's source of truth, not
        // the bundler — but the importmap must at least NAME the assets).
        assert!(html.contains("@ship/sui-runtime"), "framework runtime in importmap");
        assert!(html.contains("@ship/sdk-client"), "framework sdk in importmap");

        // Behavior 4: the entry chunk is produced by the bundler.
        assert!(build.js_href.starts_with("/assets/entry-") && build.js_href.ends_with(".js"));

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
        bundle_app(r, &[], &[], r, &out, false, false, r, r)
            .await
            .expect("bundle_app should compile the .sui entry in-process");
    }
}
