//! Dev-time binary location: find a freshly-built tool under `target/`.
//!
//! Shared by the toolchain installer and the bundler's sui-compiler locator so
//! both walk for `target/{release,debug}/<name>` the one way. L0 substrate: no
//! layer above it owns this, and both `ship-platform` and `deck` depend on it.

use std::path::PathBuf;

/// Walk up from cwd to the topmost Cargo workspace, then return the freshest
/// `target/{release,debug}/<name>` (release preferred when present).
pub fn walk_for_target_binary(name: &str) -> Result<PathBuf, String> {
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
