//! Populate `~/.ship/runtime/` from local sources.
//!
//! Three sources:
//!   - **Explicit override** via `InstallOptions::bun_source` /
//!     `InstallOptions::sui_compiler_source` / `InstallOptions::bundler_ts_source`
//!     — used by `just dev-runtime-install` and tests.
//!   - **Auto-discovery** — looks for `bun` on `PATH`, `sui-compiler` walking
//!     up from cwd, and `bundler.ts` in `runtime-assets/bundler.ts` relative to
//!     the workspace root.
//!
//! After the binaries and script land, `ensure_present` verifies the result.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ShipHome;

use super::ensure::{ToolchainError, ensure_present};

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("could not locate `bun` binary: {0}.\n  Pass --bun <path> or place bun on PATH.")]
    MissingBun(String),
    #[error("could not locate `sui-compiler` binary: {0}.\n  Pass --sui-compiler <path> or run `cargo build -p sui-compiler --release`.")]
    MissingSuiCompiler(String),
    #[error("could not locate `bundler.ts`: {0}.\n  Pass --bundler-ts <path> or ensure runtime-assets/bundler.ts exists in the workspace.")]
    MissingBundlerTs(String),
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("toolchain failed verification after install: {0}")]
    Verification(#[from] ToolchainError),
}

#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Explicit path to the Bun binary to copy in.
    pub bun_source: Option<PathBuf>,
    /// Explicit path to the sui-compiler binary to copy in.
    pub sui_compiler_source: Option<PathBuf>,
    /// Explicit path to bundler.ts to copy in.
    pub bundler_ts_source: Option<PathBuf>,
    /// When true, wipe `~/.ship/runtime/` before reinstalling.
    pub clean: bool,
}

/// Idempotent install. Safe to call repeatedly with the same options.
pub fn install(home: &ShipHome, opts: InstallOptions) -> Result<(), InstallError> {
    if opts.clean {
        wipe(&home.runtime_dir())?;
    }
    mkdir(&home.runtime_dir())?;

    let bun_src = locate_bun(opts.bun_source.as_deref())?;
    let compiler_src = locate_sui_compiler(opts.sui_compiler_source.as_deref())?;
    let bundler_ts_src = locate_bundler_ts(opts.bundler_ts_source.as_deref())?;

    copy_executable(&bun_src, &home.runtime_bun())?;
    copy_executable(&compiler_src, &home.runtime_sui_compiler())?;
    copy_file(&bundler_ts_src, &home.runtime_bundler_ts())?;

    ensure_present(home)?;
    Ok(())
}

/// Equivalent to `install` with `clean = true`.
pub fn update(home: &ShipHome, mut opts: InstallOptions) -> Result<(), InstallError> {
    opts.clean = true;
    install(home, opts)
}

fn locate_bun(explicit: Option<&Path>) -> Result<PathBuf, InstallError> {
    if let Some(p) = explicit {
        if !p.exists() {
            return Err(InstallError::MissingBun(format!("{} does not exist", p.display())));
        }
        return Ok(p.to_path_buf());
    }
    which("bun").map_err(InstallError::MissingBun)
}

fn locate_sui_compiler(explicit: Option<&Path>) -> Result<PathBuf, InstallError> {
    if let Some(p) = explicit {
        if !p.exists() {
            return Err(InstallError::MissingSuiCompiler(format!(
                "{} does not exist",
                p.display()
            )));
        }
        return Ok(p.to_path_buf());
    }
    walk_for_target_binary("sui-compiler").map_err(InstallError::MissingSuiCompiler)
}

fn locate_bundler_ts(explicit: Option<&Path>) -> Result<PathBuf, InstallError> {
    if let Some(p) = explicit {
        if !p.exists() {
            return Err(InstallError::MissingBundlerTs(format!(
                "{} does not exist",
                p.display()
            )));
        }
        return Ok(p.to_path_buf());
    }
    walk_for_bundler_ts().map_err(InstallError::MissingBundlerTs)
}

fn walk_for_target_binary(name: &str) -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
    let mut cur = cwd.as_path();
    let mut topmost: Option<PathBuf> = None;
    for _ in 0..16 {
        if cur.join("Cargo.toml").exists() {
            topmost = Some(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }
    let root = topmost.ok_or_else(|| "no Cargo workspace found above cwd".to_string())?;
    let release = root.join("target").join("release").join(name);
    if release.exists() {
        return Ok(release);
    }
    let debug = root.join("target").join("debug").join(name);
    if debug.exists() {
        return Ok(debug);
    }
    Err(format!(
        "neither {}/target/release/{name} nor target/debug/{name} found",
        root.display()
    ))
}

fn walk_for_bundler_ts() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
    let mut cur = cwd.as_path();
    for _ in 0..16 {
        let candidate = cur.join("runtime-assets").join("bundler.ts");
        if candidate.exists() {
            return Ok(candidate);
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }
    Err("runtime-assets/bundler.ts not found above cwd".to_string())
}

fn which(bin: &str) -> Result<PathBuf, String> {
    let output = Command::new("which")
        .arg(bin)
        .output()
        .map_err(|e| format!("which {bin}: {e}"))?;
    if !output.status.success() {
        return Err(format!("`{bin}` not found on PATH"));
    }
    let path = std::str::from_utf8(&output.stdout)
        .map_err(|e| format!("which output not utf8: {e}"))?
        .trim()
        .to_string();
    Ok(PathBuf::from(path))
}

fn wipe(dir: &Path) -> Result<(), InstallError> {
    if dir.exists() {
        std::fs::remove_dir_all(dir).map_err(|source| InstallError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

fn mkdir(dir: &Path) -> Result<(), InstallError> {
    std::fs::create_dir_all(dir).map_err(|source| InstallError::Io {
        path: dir.to_path_buf(),
        source,
    })
}

fn copy_executable(src: &Path, dst: &Path) -> Result<(), InstallError> {
    std::fs::copy(src, dst).map_err(|source| InstallError::Io {
        path: dst.to_path_buf(),
        source,
    })?;
    set_executable(dst)?;
    Ok(())
}

fn copy_file(src: &Path, dst: &Path) -> Result<(), InstallError> {
    std::fs::copy(src, dst).map_err(|source| InstallError::Io {
        path: dst.to_path_buf(),
        source,
    })?;
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), InstallError> {
    use std::os::unix::fs::PermissionsExt;
    let mut perm = std::fs::metadata(path)
        .map_err(|source| InstallError::Io { path: path.to_path_buf(), source })?
        .permissions();
    perm.set_mode(0o755);
    std::fs::set_permissions(path, perm).map_err(|source| InstallError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), InstallError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[cfg(unix)]
    fn make_fake_executable(path: &Path, body: &str) {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).unwrap();
        }
        std::fs::write(path, body).unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(path).unwrap().permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(path, perm).unwrap();
    }

    fn make_file(path: &Path, body: &str) {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).unwrap();
        }
        std::fs::write(path, body).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn install_with_explicit_sources_populates_runtime_dir() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());

        let bun_src = tmp.path().join("fake-bun");
        let compiler_src = tmp.path().join("fake-compiler");
        let bundler_ts_src = tmp.path().join("bundler.ts");
        make_fake_executable(&bun_src, "#!/bin/sh\necho fake-bun \"$@\"\n");
        make_fake_executable(&compiler_src, "#!/bin/sh\necho fake-compiler\n");
        make_file(&bundler_ts_src, "// bundler worker\n");

        install(
            &home,
            InstallOptions {
                bun_source: Some(bun_src),
                sui_compiler_source: Some(compiler_src),
                bundler_ts_source: Some(bundler_ts_src),
                clean: false,
            },
        )
        .unwrap();

        assert!(home.runtime_bun().is_file());
        assert!(home.runtime_sui_compiler().is_file());
        assert!(home.runtime_bundler_ts().is_file());
    }

    #[cfg(unix)]
    #[test]
    fn install_is_idempotent() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());

        let bun_src = tmp.path().join("fake-bun");
        let compiler_src = tmp.path().join("fake-compiler");
        let bundler_ts_src = tmp.path().join("bundler.ts");
        make_fake_executable(&bun_src, "#!/bin/sh\nexit 0\n");
        make_fake_executable(&compiler_src, "#!/bin/sh\nexit 0\n");
        make_file(&bundler_ts_src, "// bundler\n");

        for _ in 0..3 {
            install(
                &home,
                InstallOptions {
                    bun_source: Some(bun_src.clone()),
                    sui_compiler_source: Some(compiler_src.clone()),
                    bundler_ts_source: Some(bundler_ts_src.clone()),
                    clean: false,
                },
            )
            .unwrap();
        }
        assert!(home.runtime_bun().is_file());
        assert!(home.runtime_bundler_ts().is_file());
    }

    #[test]
    fn install_errors_when_bun_source_missing() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let err = install(
            &home,
            InstallOptions {
                bun_source: Some(tmp.path().join("nope")),
                sui_compiler_source: Some(tmp.path().join("nope2")),
                bundler_ts_source: Some(tmp.path().join("nope3")),
                clean: false,
            },
        )
        .unwrap_err();
        assert!(matches!(err, InstallError::MissingBun(_)));
    }

    #[cfg(unix)]
    #[test]
    fn update_clears_runtime_dir_before_reinstalling() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());

        std::fs::create_dir_all(home.runtime_dir()).unwrap();
        let stray = home.runtime_dir().join("stray.txt");
        std::fs::write(&stray, b"garbage").unwrap();

        let bun_src = tmp.path().join("fake-bun");
        let compiler_src = tmp.path().join("fake-compiler");
        let bundler_ts_src = tmp.path().join("bundler.ts");
        make_fake_executable(&bun_src, "#!/bin/sh\nexit 0\n");
        make_fake_executable(&compiler_src, "#!/bin/sh\nexit 0\n");
        make_file(&bundler_ts_src, "// bundler\n");

        update(
            &home,
            InstallOptions {
                bun_source: Some(bun_src),
                sui_compiler_source: Some(compiler_src),
                bundler_ts_source: Some(bundler_ts_src),
                clean: true,
            },
        )
        .unwrap();

        assert!(!stray.exists(), "stray file should have been wiped");
        assert!(home.runtime_bun().is_file());
    }
}
