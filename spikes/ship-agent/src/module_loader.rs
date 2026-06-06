//! Deny-by-default ModuleLoader — the closed world for imports.
//!
//! deno_core ships no package manager; the embedder owns module resolution. We
//! resolve EXACTLY two specifiers: the agent's own main module, and `ship` (the
//! generated scoped SDK). Every other import is refused. No npm, no http, no
//! file — nothing ambient. This is the import-side twin of the capability gate:
//! the agent can import only what we hand it.

use deno_core::error::ModuleLoaderError;
use deno_core::{
    ModuleLoadOptions, ModuleLoadReferrer, ModuleLoadResponse, ModuleLoader, ModuleSource,
    ModuleSourceCode, ModuleSpecifier, ModuleType, ResolutionKind,
};
use deno_error::JsErrorBox;

pub const MAIN: &str = "ship:main";
pub const SDK: &str = "ship:sdk";

pub struct ShipModuleLoader {
    sdk_src: String,
    agent_src: String,
}

impl ShipModuleLoader {
    pub fn new(sdk_src: String, agent_src: String) -> Self {
        Self { sdk_src, agent_src }
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
            // Everything else is refused — deny-by-default.
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
            MAIN => self.agent_src.clone(),
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
