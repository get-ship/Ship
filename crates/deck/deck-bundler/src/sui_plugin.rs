//! SUI resolve/load plugin for Rolldown. Handles:
//!   - `resolve_id`: redirect `./X.compiled.js` siblings to a virtual id,
//!     externalize the framework singletons, re-anchor `@ship/<pkg>` to
//!     workspace `src/`, resolve `@sui-tag/<tag>` to the component source.
//!   - `load`: compile `.sui` (and `.compiled.js`→sibling `.sui`) via
//!     sui-compiler, returning JS.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{anyhow, Result};
use rolldown::plugin::{
    HookLoadArgs, HookLoadOutput, HookLoadReturn, HookResolveIdArgs, HookResolveIdOutput,
    HookResolveIdReturn, HookUsage, Plugin, PluginContext, SharedLoadPluginContext,
};
use rolldown_common::{ModuleType, ResolvedExternal};

use deck_config::FirstPartyApp;

use crate::compile::{compile_sui, CompileMode};

const SINGLETONS: [&str; 3] = ["@ship/sui-runtime", "@ship/sui-dom", "@ship/sdk-client"];

/// Virtual id for an unresolved `@sui-tag/<tag>` import — a shared empty module
/// so the import is a harmless no-op. `\0` marks it virtual (Rollup/Rolldown
/// convention: never read/written on disk).
const SUI_TAG_NOOP: &str = "\0sui-tag-noop";

/// Component source roots — a `.sui` under one is a compile-as-component
/// shortcut (saves the parent-name heuristic a step for deck-own packages).
/// Host-app component roots are NOT listed here — the naming heuristic
/// (`stem == parent`) catches them via the `<tag>/<tag>.sui` convention.
pub(crate) fn detect_mode(abs: &Path, repo_root: &Path) -> (CompileMode, String) {
    let stem = abs.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
    // Deck-own component source: always compile as Component.
    if abs.starts_with(repo_root.join("packages/components/ui-components/src")) {
        return (CompileMode::Component, stem);
    }
    // Generic heuristic: `<tag>/<tag>.sui` → Component; anything else → View.
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

pub(crate) fn normalize(p: &Path) -> PathBuf {
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
    /// View builds opt in: each `.sui` compiled in this build emits
    /// `@sui-tag/<tag>` registration imports (the per-view resolver). Off for
    /// framework/app builds.
    pub(crate) emit_component_imports: bool,
    /// `<tag>` → component source `.sui`, for resolving `@sui-tag/<tag>`.
    /// Empty when `emit_component_imports` is false.
    pub(crate) component_index: Arc<HashMap<String, PathBuf>>,
}

impl SuiPlugin {
    /// Compile a `.sui` to JS via `sui_compiler::compile` IN-PROCESS (#255):
    /// the compiler is the linked crate, not a spawned binary — no temp dir, no
    /// file roundtrip, no version drift, no flag that a stale binary can reject.
    fn compile(&self, sui: &Path) -> Result<String> {
        let (mode, stem) = detect_mode(sui, &self.repo_root);
        compile_sui(sui, mode, &stem, self.emit_component_imports)
            .map_err(|e| anyhow!("compile {}: {e}", sui.display()))
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
        // `@sui-tag/<tag>` — the per-view resolver. Map to the component's
        // source `.sui` (so it compiles into the view bundle); an unknown tag
        // (view-private, or no CEM yet) → virtual no-op. Emitted only under
        // `--emit-component-imports`.
        if let Some(tag) = spec.strip_prefix("@sui-tag/") {
            return Ok(Some(match self.component_index.get(tag) {
                Some(src) => HookResolveIdOutput::from_id(src.to_string_lossy().to_string()),
                // Unknown tag (view-private, or no CEM yet): resolve to a shared
                // empty module — a harmless no-op. NOT external, which would
                // leave an unresolvable `@sui-tag/<tag>` specifier in the served view.
                None => HookResolveIdOutput::from_id(SUI_TAG_NOOP.to_string()),
            }));
        }
        // `*.compiled.js` → virtual id; `load` compiles the sibling `.sui`. The
        // `.compiled.js` is synthetic (never on disk), so we must redirect here
        // rather than fall through to Rolldown's native resolver, which would
        // fail to resolve a nonexistent file. Two shapes reach here: relative
        // `./X.compiled.js` siblings (umbrella `index.ts` imports), and the
        // ABSOLUTE paths the virtual-aggregate entry emits — both must become
        // a virtual id.
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
        // (`@ship/tokens`) resolves to `src/index.ts`; a subpath
        // (`@ship/tokens/themes`) resolves to the matching file
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
                // The 3-bucket component reorg (#219) nests first-party
                // component packages two levels deep under `packages/components/
                // {sui,ship,ui-components}/<pkg>`. Without these 2-level
                // umbrellas the resolver can't find e.g. @ship/sui-primitives
                // (packages/components/sui/primitives), leaves the import bare,
                // and the served umbrella (`/_sui/components.js`) fails to load
                // with "Failed to resolve module specifier @ship/sui-primitives"
                // → no <sui-*> primitive registers.
                for umbrella in [
                    "",
                    "sui",
                    "deck",
                    "components",
                    "components/sui",
                    "components/ship",
                    "components/ui-components",
                ] {
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
        if id == SUI_TAG_NOOP {
            // Empty no-op for an unresolved component tag. Tree-shaken away
            // if nothing keeps it.
            return Ok(Some(HookLoadOutput {
                code: String::new().into(),
                module_type: Some(ModuleType::Js),
                ..Default::default()
            }));
        }
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

/// Cached `<tag>` → source `.sui` index for a repo (the per-view resolver's
/// lookup). The CEM walk is process-stable; build once per repo root and share.
/// A component added mid-process isn't picked up until restart — until then its
/// tag resolves to a virtual no-op.
///
/// `first_party_app` comes from `deck.jsonc` and drives the shell-local CEM
/// projection (which app root gets a special origin label). `None` in
/// zero-config hosts — the projection simply doesn't apply.
pub(crate) fn component_index(
    repo_root: &Path,
    first_party_app: Option<&FirstPartyApp>,
) -> Arc<HashMap<String, PathBuf>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<HashMap<String, PathBuf>>>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(idx) = guard.get(repo_root) {
        return idx.clone();
    }
    let idx = Arc::new(crate::catalog::tag_source_index(repo_root, first_party_app));
    guard.insert(repo_root.to_path_buf(), idx.clone());
    idx
}
