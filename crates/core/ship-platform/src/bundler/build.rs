//! Per-target build — the two phases composed (#163 slice 2c).
//!
//! Combines `compile` (phase 1: `.sui` → `.js`) and `esbuild` (phase 2: bundle)
//! for one `BundleTarget`:
//!   - Real `.ts` entry  → esbuild directly (framework bundles).
//!   - Real `.sui` entry → compile to staging, esbuild the compiled `.js` (views).
//!   - Virtual aggregate → compile each source `.sui`, generate an entry that
//!     imports each compiled `.js`, esbuild the entry (separate-bundle islands,
//!     marketing).
//!
//! `sui-compiler build --out <dir>` emits the controller import RELATIVE TO the
//! out dir, so compiling into staging keeps controller resolution correct for
//! esbuild — no resolver plugin needed (the model the esbuild Go binary forces).

use std::path::{Path, PathBuf};

use super::compile::{CompileError, CompileMode, compile_sui};
use super::esbuild::{EsbuildError, EsbuildOptions, run_esbuild};
use super::proto::BundleManifest;
use super::targets::{BundleLayout, BundleTarget, TargetEntry};

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("compile failed for {target}: {source}")]
    Compile {
        target: String,
        #[source]
        source: CompileError,
    },
    #[error("bundle failed for {target}: {source}")]
    Bundle {
        target: String,
        #[source]
        source: EsbuildError,
    },
    #[error("io error for {target} at {path}: {source}")]
    Io {
        target: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Component source roots — a `.sui` under one of these is a custom-element
/// regardless of filename/dir match (dirs pack multiple tags). Mirrors the
/// worker's `COMPONENT_ROOTS` + `detectMode`.
fn detect_mode(abs_path: &Path, repo_root: &Path) -> (CompileMode, String) {
    let stem = abs_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    for rel in ["packages/components/ui-components/src", "apps/shell/src/components"] {
        if abs_path.starts_with(repo_root.join(rel)) {
            return (CompileMode::Component, stem);
        }
    }
    let parent = abs_path
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

/// Externals for a target: every bundle except `sui-runtime` itself forces
/// `@ship/sui-runtime` external so all SUI code shares one runtime instance.
/// (Plus the target's declared externals.) Mirrors the worker's injection.
fn externals_for(target: &BundleTarget) -> Vec<String> {
    if target.name == "sui-runtime" {
        return target.externals.clone();
    }
    let mut set = target.externals.clone();
    if !set.iter().any(|e| e == "@ship/sui-runtime") {
        set.push("@ship/sui-runtime".to_string());
    }
    set
}

/// Build one target. `staging` is a caller-owned scratch dir (one per build);
/// `compiler_bin`/`esbuild_bin` are pre-located. Output lands at
/// `layout.assets_dir_for(name)/out_file`.
pub fn build_target(
    layout: &BundleLayout,
    target: &BundleTarget,
    compiler_bin: &Path,
    esbuild_bin: &Path,
    staging: &Path,
    production: bool,
) -> Result<BundleManifest, BuildError> {
    let entry = resolve_entry(layout, target, compiler_bin, staging)?;

    let out_path = layout.assets_dir_for(&target.name).join(&target.out_file);
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| BuildError::Io {
            target: target.name.clone(),
            path: parent.to_path_buf(),
            source,
        })?;
    }

    run_esbuild(
        esbuild_bin,
        &EsbuildOptions {
            entry,
            outfile: out_path.clone(),
            externals: externals_for(target),
            production,
            cwd: layout.repo_root.clone(),
        },
    )
    .map(|r| BundleManifest {
        target: target.name.clone(),
        out_path: r.out_path.display().to_string(),
        bytes: r.bytes,
        inputs: target_sources(target),
    })
    .map_err(|source| BuildError::Bundle { target: target.name.clone(), source })
}

/// Produce the esbuild entry: compile `.sui` sources into `staging` as needed,
/// generating an aggregate entry for virtual targets.
fn resolve_entry(
    layout: &BundleLayout,
    target: &BundleTarget,
    compiler_bin: &Path,
    staging: &Path,
) -> Result<PathBuf, BuildError> {
    match &target.entry {
        TargetEntry::Real(path) => {
            if path.extension().is_some_and(|x| x == "sui") {
                compile_one(layout, target, compiler_bin, staging, path, 0)
            } else {
                Ok(path.clone())
            }
        }
        TargetEntry::Virtual { sources } => {
            let mut imports = String::new();
            for (i, src) in sources.iter().enumerate() {
                let js = compile_one(layout, target, compiler_bin, staging, src, i)?;
                imports.push_str(&format!("import {:?}\n", js.display().to_string()));
            }
            let entry = staging.join("_entry.ts");
            std::fs::write(&entry, imports).map_err(|source| BuildError::Io {
                target: target.name.clone(),
                path: entry.clone(),
                source,
            })?;
            Ok(entry)
        }
    }
}

/// Compile one `.sui` into a per-index subdir of `staging` (so same-stem sources
/// in one aggregate don't collide), returning the compiled `.js` path.
fn compile_one(
    layout: &BundleLayout,
    target: &BundleTarget,
    compiler_bin: &Path,
    staging: &Path,
    sui: &Path,
    idx: usize,
) -> Result<PathBuf, BuildError> {
    let (mode, stem) = detect_mode(sui, &layout.repo_root);
    let out_dir = staging.join(format!("c{idx}"));
    std::fs::create_dir_all(&out_dir).map_err(|source| BuildError::Io {
        target: target.name.clone(),
        path: out_dir.clone(),
        source,
    })?;
    let js_path = compile_sui(compiler_bin, sui, &out_dir, mode, &stem)
        .map(|r| r.js_path)
        .map_err(|source| BuildError::Compile { target: target.name.clone(), source })?;

    // The compiler emits the controller as a SIBLING relative import
    // (`from "./<stem>.controller.ts"`), assuming output sits next to source.
    // Output lives in staging, so rewrite relative specifiers to absolute paths
    // anchored at the real source dir — esbuild then resolves the controller
    // (and its own relative deps, from their real location) with no plugin.
    let source_dir = sui.parent().unwrap_or(&layout.repo_root);
    let js = std::fs::read_to_string(&js_path).map_err(|source| BuildError::Io {
        target: target.name.clone(),
        path: js_path.clone(),
        source,
    })?;
    let rewritten = rewrite_relative_imports(&js, source_dir);
    std::fs::write(&js_path, rewritten).map_err(|source| BuildError::Io {
        target: target.name.clone(),
        path: js_path.clone(),
        source,
    })?;
    Ok(js_path)
}

/// Rewrite relative `from "./x"` / `from "../x"` specifiers to absolute paths
/// resolved against `source_dir`. The compiler only emits double-quoted `from`
/// specifiers (the controller import + the externalized `@ship/*`), so this
/// covers the surface; bare/`@`-prefixed specifiers pass through unchanged.
fn rewrite_relative_imports(js: &str, source_dir: &Path) -> String {
    const NEEDLE: &str = "from \"";
    let mut out = String::with_capacity(js.len());
    let mut rest = js;
    while let Some(pos) = rest.find(NEEDLE) {
        let spec_start = pos + NEEDLE.len();
        out.push_str(&rest[..spec_start]); // up to + including the opening quote
        let after = &rest[spec_start..];
        match after.find('"') {
            Some(close) => {
                let spec = &after[..close];
                if spec.starts_with("./") || spec.starts_with("../") {
                    let abs = normalize_path(&source_dir.join(spec));
                    out.push_str(&abs.to_string_lossy());
                } else {
                    out.push_str(spec);
                }
                out.push('"');
                rest = &after[close + 1..];
            }
            None => {
                out.push_str(after);
                return out;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Lexically normalize `.`/`..` in a path (no filesystem access).
fn normalize_path(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn target_sources(target: &BundleTarget) -> Vec<String> {
    match &target.entry {
        TargetEntry::Real(p) => vec![p.display().to_string()],
        TargetEntry::Virtual { sources } => {
            sources.iter().map(|p| p.display().to_string()).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn compiler_bin() -> PathBuf {
        crate::toolchain::install::walk_for_target_binary("sui-compiler")
            .expect("sui-compiler must be built for tests")
    }
    fn esbuild_bin() -> PathBuf {
        crate::toolchain::install::discover_esbuild().expect("esbuild must be in node_modules")
    }

    fn write(path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    /// A view `.sui` compiles then bundles end-to-end, runtime kept external.
    #[test]
    fn builds_view_sui_end_to_end() {
        let repo = tempdir().unwrap();
        let staging = tempdir().unwrap();
        write(&repo.path().join("views/demo/page.sui"), "<div class=\"d\">hi {1 + 1}</div>\n");

        let layout = BundleLayout::new(repo.path().to_path_buf(), None);
        let target = layout
            .all_targets()
            .into_iter()
            .find(|t| t.name == "views/demo/view")
            .expect("demo view discovered");

        let manifest = build_target(
            &layout,
            &target,
            &compiler_bin(),
            &esbuild_bin(),
            staging.path(),
            false,
        )
        .expect("view should build");

        assert_eq!(manifest.target, "views/demo/view");
        assert!(manifest.bytes > 0);
        let out = repo.path().join(".deck/views/demo/view.js");
        assert!(out.exists(), "bundle written to .deck");
        let body = std::fs::read_to_string(&out).unwrap();
        assert!(
            body.contains("@ship/sui-runtime"),
            "runtime singleton kept external"
        );
    }
}
