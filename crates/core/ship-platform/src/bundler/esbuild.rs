//! esbuild Go-binary adapter for the Rust bundler (#163).
//!
//! The `Bundler` shells out to the standalone esbuild executable instead of
//! hosting esbuild's JS API through Bun. The B-spike (see
//! `.artifacts/bundler-seam.md`) proved that `esbuild --bundle` + three static
//! `--external:` flags reproduces `mirrorResolverPlugin`'s behavior for every
//! compiled artifact in the tree — the only `@ship/*` imports compiled code
//! carries are the framework singletons, so no resolver plugin is needed.

use std::path::{Path, PathBuf};
use std::process::Command;

use ship_substrate::ShipHome;

/// Framework singletons that MUST stay external for every bundle so all SUI
/// code shares the one runtime instance the host import map provides. Inlining
/// a copy creates split-brain signals (`state.$signals is undefined` at mount).
/// Mirrors `mirrorResolverPlugin`'s SHARED_SINGLETONS.
pub const SHARED_SINGLETON_EXTERNALS: [&str; 3] =
    ["@ship/sui-runtime", "@ship/sui-dom", "@ship/sdk-client"];

#[derive(Debug, thiserror::Error)]
pub enum EsbuildError {
    #[error("esbuild spawn failed: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("esbuild exited {code}:\n{stderr}")]
    Failed { code: i32, stderr: String },
    #[error("esbuild reported success but produced no output at {0}")]
    NoOutput(PathBuf),
}

/// One bundle invocation.
#[derive(Debug, Clone)]
pub struct EsbuildOptions {
    /// Entry module (already-compiled `.js`/`.ts`).
    pub entry: PathBuf,
    /// Output bundle path.
    pub outfile: PathBuf,
    /// Bare specifiers to keep external (typically `SHARED_SINGLETON_EXTERNALS`).
    pub externals: Vec<String>,
    /// Production build: minify + `NODE_ENV=production`. Dev adds the
    /// `development` export condition so dev-mode warnings/asserts resolve —
    /// matches the JS esbuild adapter's mode handling.
    pub production: bool,
    /// Working directory esbuild resolves relative imports against.
    pub cwd: PathBuf,
}

#[derive(Debug, Clone)]
pub struct EsbuildResult {
    pub out_path: PathBuf,
    pub bytes: usize,
}

/// Locate the esbuild binary: the installed runtime copy first, then a dev
/// fallback to node_modules discovery (so a contributor checkout bundles before
/// `ship runtime install`). One discovery path, shared with toolchain install.
pub fn locate_esbuild_binary(home: &ShipHome) -> Option<PathBuf> {
    let installed = home.runtime_esbuild();
    if installed.exists() {
        return Some(installed);
    }
    crate::toolchain::install::discover_esbuild()
}

/// Run one esbuild bundle. Synchronous one-shot; async callers wrap in
/// `tokio::task::spawn_blocking`.
pub fn run_esbuild(bin: &Path, opts: &EsbuildOptions) -> Result<EsbuildResult, EsbuildError> {
    let node_env = if opts.production { "production" } else { "development" };
    let mut cmd = Command::new(bin);
    cmd.arg(&opts.entry)
        .arg("--bundle")
        .arg("--format=esm")
        .arg("--platform=browser")
        .arg("--target=esnext")
        .arg(format!("--outfile={}", opts.outfile.display()))
        .arg(format!("--define:process.env.NODE_ENV=\"{node_env}\""))
        .current_dir(&opts.cwd);
    if opts.production {
        cmd.arg("--minify");
    } else {
        // Lit (and other libs) gate dev warnings behind the `development`
        // export condition; the JS adapter sets it for non-prod builds.
        cmd.arg("--conditions=development");
    }
    for ext in &opts.externals {
        cmd.arg(format!("--external:{ext}"));
    }

    let out = cmd.output().map_err(EsbuildError::Spawn)?;
    if !out.status.success() {
        return Err(EsbuildError::Failed {
            code: out.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }
    let bytes = std::fs::metadata(&opts.outfile)
        .map(|m| m.len() as usize)
        .map_err(|_| EsbuildError::NoOutput(opts.outfile.clone()))?;
    Ok(EsbuildResult { out_path: opts.outfile.clone(), bytes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn esbuild_bin() -> PathBuf {
        crate::toolchain::install::discover_esbuild()
            .expect("esbuild must be in node_modules for tests — run `bun install`")
    }

    /// The core B-spike claim, in Rust: a local import is inlined, a singleton
    /// import stays external via `--external:`. No plugin.
    #[test]
    fn inlines_local_and_keeps_singleton_external() {
        let bin = esbuild_bin();
        let tmp = tempdir().unwrap();
        std::fs::write(tmp.path().join("dep.ts"), "export const dep = 42;\n").unwrap();
        std::fs::write(
            tmp.path().join("entry.ts"),
            "import { dep } from './dep.ts';\n\
             import { signal } from '@ship/sui-runtime';\n\
             export const v = dep;\nexport { signal };\n",
        )
        .unwrap();
        let outfile = tmp.path().join("out.js");

        let res = run_esbuild(
            &bin,
            &EsbuildOptions {
                entry: tmp.path().join("entry.ts"),
                outfile: outfile.clone(),
                externals: vec!["@ship/sui-runtime".to_string()],
                production: false,
                cwd: tmp.path().to_path_buf(),
            },
        )
        .expect("bundle should succeed");

        assert_eq!(res.out_path, outfile);
        assert!(res.bytes > 0);
        let body = std::fs::read_to_string(&outfile).unwrap();
        assert!(body.contains("42"), "local dep must be inlined");
        assert!(
            body.contains("@ship/sui-runtime"),
            "singleton import must stay external, got:\n{body}"
        );
    }

    #[test]
    fn surfaces_esbuild_errors() {
        let bin = esbuild_bin();
        let tmp = tempdir().unwrap();
        std::fs::write(
            tmp.path().join("broken.ts"),
            "import { x } from './does-not-exist.ts';\nexport const y = x;\n",
        )
        .unwrap();
        let err = run_esbuild(
            &bin,
            &EsbuildOptions {
                entry: tmp.path().join("broken.ts"),
                outfile: tmp.path().join("out.js"),
                externals: vec![],
                production: false,
                cwd: tmp.path().to_path_buf(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, EsbuildError::Failed { .. }));
    }
}
