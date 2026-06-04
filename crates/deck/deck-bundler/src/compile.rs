//! sui-compiler integration for the Rust bundler — IN-PROCESS (#255).
//!
//! The bundler calls `sui_compiler::compile()` directly from its Rolldown
//! `load` hook. One toolchain: the compiler is the crate the bundler links, so
//! there is no separate `sui-compiler` binary to locate, no installed-vs-debug-
//! vs-release drift, and a flag the bundler passes can never be rejected by a
//! stale binary. `just build` produces a bundler carrying *the* current
//! compiler — build = works, no gate, no version dance.

use std::path::{Path, PathBuf};

use sui_compiler::{compile, CompileMode as SuiMode, CompileOpts, ComponentMeta};

/// A `.sui` compiles as a custom-element (`component`) or a view. Component mode
/// passes a tag name so the element is registered; view mode emits a route module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    Component,
    View,
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("read {0}: {1}")]
    Read(PathBuf, #[source] std::io::Error),
    /// A rendered compiler diagnostic (with source location). Consumers
    /// stringify it (the HMR overlay, the bundle error); no structured fields.
    #[error("{0}")]
    Compile(String),
}

/// Compile one `.sui` to JS via `sui_compiler::compile` (in-process). Auto-wires
/// a sibling controller (`page.sui`→`controller.ts`, `<stem>.sui`→
/// `<stem>.controller.ts`) as `import setup from "./<controller>"`.
/// `emit_component_imports` opts the view into the per-view resolver — it emits
/// `@sui-tag/<tag>` registration imports the bundler resolves to source.
pub fn compile_sui(
    sui_file: &Path,
    mode: CompileMode,
    stem: &str,
    emit_component_imports: bool,
) -> Result<String, CompileError> {
    let source = std::fs::read_to_string(sui_file)
        .map_err(|e| CompileError::Read(sui_file.to_path_buf(), e))?;
    let opts = CompileOpts {
        setup_url: paired_controller(sui_file, stem),
        mode: match mode {
            CompileMode::Component => SuiMode::Component,
            CompileMode::View => SuiMode::View,
        },
        source_path: Some(sui_file.to_path_buf()),
        styles: None,
        component_meta: (mode == CompileMode::Component)
            .then(|| ComponentMeta { name: stem.to_string(), shadow: true }),
        // The bundler reads only the JS — dts/ssr are the CLI bake's job, and
        // the CEM is walked from disk by the catalog, not returned here.
        emit_dts: false,
        emit_ssr: false,
        emit_component_imports,
    };
    match compile(&source, opts) {
        Ok(out) => Ok(out.js),
        Err(e) => Err(CompileError::Compile(e.render(&source))),
    }
}

/// `import setup from "./<controller>"` when a sibling controller exists next to
/// the `.sui`. Mirrors the CLI's `paired_setup` for the bundler (`--out`/
/// in-process) case: relative `./` import that Rolldown resolves to the sibling.
fn paired_controller(sui_file: &Path, stem: &str) -> Option<String> {
    let parent = sui_file.parent()?;
    let filename = if stem == "page" {
        "controller.ts".to_string()
    } else {
        format!("{stem}.controller.ts")
    };
    parent.join(&filename).exists().then(|| format!("./{filename}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn compiles_view_to_js_in_process() {
        let src = tempdir().unwrap();
        let sui = src.path().join("page.sui");
        std::fs::write(&sui, "<div class=\"hello\">hi {1 + 1}</div>\n").unwrap();
        let js = compile_sui(&sui, CompileMode::View, "page", false).expect("view compiles");
        assert!(js.contains("@ship/sui-runtime"), "emits a runtime import; got:\n{js}");
    }

    #[test]
    fn surfaces_compile_errors_with_rendered_diagnostic() {
        let src = tempdir().unwrap();
        let sui = src.path().join("broken.sui");
        std::fs::write(&sui, "<div>{#if cond()}oops</div>\n").unwrap();
        let err = compile_sui(&sui, CompileMode::View, "broken", false).unwrap_err();
        assert!(matches!(err, CompileError::Compile(_)));
    }

    #[test]
    fn component_imports_flag_emits_tag_registration() {
        let src = tempdir().unwrap();
        let sui = src.path().join("page.sui");
        std::fs::write(&sui, "<div><sui-table></sui-table></div>\n").unwrap();
        let on = compile_sui(&sui, CompileMode::View, "page", true).expect("compiles");
        assert!(on.contains("@sui-tag/sui-table"), "flag emits the registration import");
        let off = compile_sui(&sui, CompileMode::View, "page", false).expect("compiles");
        assert!(!off.contains("@sui-tag/"), "off: no component imports");
    }
}
