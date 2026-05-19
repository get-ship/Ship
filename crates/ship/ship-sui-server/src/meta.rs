//! `GET /api/_meta` — drift-detection endpoint.
//!
//! Returns the compiler identity stamp baked into every compiled
//! artifact alongside the daemon's own build identity. The shell
//! fetches this at startup and compares it against the
//! `_compilerVersion` marker in the bundle it loaded. Any mismatch
//! is surfaced as a banner / console warning rather than the silent
//! breakage we hit with the `fade` export regression.
//!
//! The daemon supplies its own build info via
//! [`axum::Extension<DaemonBuildInfo>`] because `build_info::build_info!()`
//! is a macro that must expand inside the binary crate (cargo scopes
//! `rustc-env` per crate). Compiler info comes from `sui_compiler`
//! directly — it bakes the same env vars via its own `build.rs`.

use axum::{Extension, Json};
use serde::Serialize;
use serde_json::{json, Value};

/// Daemon-side build identity, passed in as an Axum Extension so the
/// handler can stay generic. Mirrors the relevant subset of
/// `build_info::BuildInfo` — `&'static str` fields stay static across
/// the daemon lifetime.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct DaemonBuildInfo {
    pub cargo: &'static str,
    pub git: &'static str,
    pub dirty: bool,
    pub built_at: &'static str,
}

impl DaemonBuildInfo {
    pub fn to_json(self) -> Value {
        json!({
            "cargo": self.cargo,
            "git": self.git,
            "dirty": self.dirty,
            "builtAt": self.built_at,
        })
    }
}

/// `GET /api/_meta` handler.
pub async fn serve_meta(daemon: Option<Extension<DaemonBuildInfo>>) -> Json<Value> {
    let daemon = daemon.map(|e| e.0).unwrap_or(DaemonBuildInfo {
        cargo: "unknown",
        git: "unknown",
        dirty: false,
        built_at: "unknown",
    });
    let compiler = sui_compiler::compiler_version();
    Json(json!({
        "compilerVersion": compiler.to_json(),
        "daemonVersion": daemon.to_json(),
        "builtAt": daemon.built_at,
    }))
}
