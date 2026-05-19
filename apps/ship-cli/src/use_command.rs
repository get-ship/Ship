//! `ship use --launch` — dispatch a single agent session.
//!
//! After the existing compile-and-activate flow runs in `main.rs::run_use`,
//! this module forwards launch-bearing invocations to the daemon's
//! `POST /api/v1/agent_sessions/dispatch` endpoint.

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

/// Inputs forwarded from the CLI `Use` variant for the launch path.
pub struct LaunchOpts {
    pub agent: String,
    pub path: Option<PathBuf>,
    pub launch: bool,
    pub auto: bool,
    pub resume: bool,
    pub prompt: Option<String>,
    pub spec: Option<PathBuf>,
    pub harness: Option<String>,
    pub model: Option<String>,
}

/// True when any flag in the launch family is set. Implements the
/// implicit-launch contract: `--auto`, `--resume`, `--prompt`, `--spec`
/// each turn `ship use` into a dispatch.
pub fn should_launch(opts: &LaunchOpts) -> bool {
    opts.launch
        || opts.auto
        || opts.resume
        || opts.prompt.is_some()
        || opts.spec.is_some()
}

/// Build the JSON body for `POST /api/v1/agent_sessions/dispatch`.
///
/// `effective_path` is the directory the user is launching into — used to
/// derive the workspace label from its basename.
pub fn build_dispatch_body(
    opts: &LaunchOpts,
    effective_path: &Path,
) -> Result<serde_json::Value> {
    if opts.prompt.is_some() && opts.spec.is_some() {
        bail!("--prompt and --spec are mutually exclusive");
    }

    let spec_content = match opts.spec.as_ref() {
        Some(p) => Some(
            std::fs::read_to_string(p)
                .map_err(|e| anyhow::anyhow!("cannot read spec file {}: {e}", p.display()))?,
        ),
        None => None,
    };

    let source_spec_path = opts.spec.as_ref().and_then(|p| {
        std::fs::canonicalize(p)
            .ok()
            .and_then(|abs| abs.to_str().map(String::from))
            .or_else(|| Some(p.display().to_string()))
    });

    let workspace = workspace_label_from_path(effective_path);

    // Translate the legacy `--auto` CLI flag to the new orthogonal-axes
    // shape the daemon now requires (the inversion plan in
    // .artifacts/agents-arc/discussions/ explains why `auto` was wrong).
    // Phase 7 of the inversion replaces this CLI surface with explicit
    // -i/-t/-d flags; this is the minimum shim to keep `ship use
    // [--auto]` dispatching against the new endpoint until then.
    let (open_tab, keepalive) = if opts.auto {
        // Old `--auto` semantic: no UI tab, batch invocation.
        // New equivalent that preserves subscription billing: PTY
        // one-shot (no keepalive). API-billed --print stays opt-in via
        // the new CLI; never silently selected here.
        (false, false)
    } else {
        (true, true)
    };
    let mut body = serde_json::json!({
        "agent": opts.agent,
        "open_tab": open_tab,
        "invocation": "pty",
        "keepalive": keepalive,
        "resume": opts.resume,
        "workspace": workspace,
    });
    let obj = body.as_object_mut().expect("body is object");
    if let Some(p) = &opts.prompt {
        obj.insert("prompt".into(), serde_json::Value::String(p.clone()));
    }
    if let Some(s) = spec_content {
        obj.insert("spec".into(), serde_json::Value::String(s));
    }
    if let Some(h) = &opts.harness {
        obj.insert("harness".into(), serde_json::Value::String(h.clone()));
    }
    if let Some(m) = &opts.model {
        obj.insert("model".into(), serde_json::Value::String(m.clone()));
    }
    if let Some(sp) = source_spec_path {
        obj.insert("source_spec_path".into(), serde_json::Value::String(sp));
    }
    Ok(body)
}

/// Derive the workspace label from the effective launch path. Falls back
/// to `"workspace"` when the path has no usable final component (e.g. `/`).
fn workspace_label_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(String::from)
        .unwrap_or_else(|| "workspace".to_string())
}

/// Resolve the effective directory for the launch — `--path` if set,
/// otherwise the current working directory.
pub fn effective_path(opts: &LaunchOpts) -> Result<PathBuf> {
    match opts.path.as_ref() {
        Some(p) => std::fs::canonicalize(p)
            .map_err(|e| anyhow::anyhow!("--path {}: {e}", p.display())),
        None => std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("cannot read current directory: {e}")),
    }
}

/// Orchestrate the launch: build body, POST to the daemon, surface result.
pub fn run_launch(opts: LaunchOpts) -> Result<()> {
    let path = effective_path(&opts)?;
    let body = build_dispatch_body(&opts, &path)?;
    let mode = if opts.auto { "auto" } else { "interactive" };
    let resp = post_dispatch(&body)?;

    let ok = resp.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    if !ok {
        let err = resp
            .get("data")
            .and_then(|d| d.as_str())
            .unwrap_or("unknown error");
        bail!("dispatch rejected: {err}");
    }

    let slug = resp
        .get("data")
        .and_then(|d| d.get("slug"))
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    println!("→ launched {slug} ({mode}) — tail with: ship procs --slug {slug}");
    Ok(())
}

/// POST the dispatch body to the running daemon, auto-starting if needed.
/// Translates connection failures into a clear "shipd not running" error.
fn post_dispatch(body: &serde_json::Value) -> Result<serde_json::Value> {
    let (port, token) = crate::daemon::ensure_daemon()
        .map_err(|e| anyhow::anyhow!("shipd not running: {e}\nTry: ship daemon start"))?;
    let url = format!("http://127.0.0.1:{port}/api/v1/agent_sessions/dispatch");
    let resp: serde_json::Value = ureq::post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        .send_json(body)
        .map_err(|e| anyhow::anyhow!("dispatch POST failed: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| anyhow::anyhow!("dispatch response was not JSON: {e}"))?;
    Ok(resp)
}

#[cfg(test)]
#[path = "use_command_tests.rs"]
mod tests;
