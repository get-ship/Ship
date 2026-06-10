//! Deny-by-default `ModuleLoader` — the closed world for imports.
//!
//! deno_core ships no package manager; the embedder owns module resolution. We
//! resolve EXACTLY two specifiers: the host's main module, and `ship` (the
//! scoped SDK the host hands in). Every other import is refused. No npm, no
//! http, no file — nothing ambient. This is the import-side twin of the host's
//! capability gate: the isolate can import only what the host hands it.
//!
//! This loader is pure deno_core — it holds no auth, no gateway, no SDK *policy*.
//! It is constructed with two already-resolved source strings (`sdk_src`,
//! `main_src`); deciding what the SDK contains (e.g. scoping it to grants) is the
//! host's job, upstream of here.

use deno_core::error::ModuleLoaderError;
use deno_core::{
    ModuleLoadOptions, ModuleLoadReferrer, ModuleLoadResponse, ModuleLoader, ModuleSource,
    ModuleSourceCode, ModuleSpecifier, ModuleType, ResolutionKind,
};
use deno_error::JsErrorBox;

/// The host's main module specifier — the entry point.
pub const MAIN: &str = "ship:main";
/// The scoped-SDK module specifier — the one thing `import "ship"` resolves to.
pub const SDK: &str = "ship:sdk";

/// A two-module closed world: the main module and the SDK, nothing else.
pub struct ShipModuleLoader {
    sdk_src: String,
    main_src: String,
}

impl ShipModuleLoader {
    /// `sdk_src` is served for `import "ship"`; `main_src` is the entry module.
    pub fn new(sdk_src: String, main_src: String) -> Self {
        Self { sdk_src, main_src }
    }
}

impl ModuleLoader for ShipModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        _referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, ModuleLoaderError> {
        match specifier {
            // The one importable module: the scoped SDK.
            "ship" => ModuleSpecifier::parse(SDK).map_err(JsErrorBox::from_err),
            // Already-canonical internal specifiers.
            MAIN | SDK => ModuleSpecifier::parse(specifier).map_err(JsErrorBox::from_err),
            // Everything else is refused — deny-by-default, at resolve-time so
            // not even an error message reveals that `fs` is a known module.
            other => Err(JsErrorBox::generic(format!(
                "import denied: `{other}` — only `ship` is importable in this runtime"
            ))),
        }
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        let src = match module_specifier.as_str() {
            SDK => self.sdk_src.clone(),
            MAIN => self.main_src.clone(),
            other => {
                return ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!(
                    "load denied: `{other}`"
                ))))
            }
        };
        ModuleLoadResponse::Sync(Ok(ModuleSource::new(
            ModuleType::JavaScript,
            ModuleSourceCode::String(src.into()),
            module_specifier,
            None,
        )))
    }
}
