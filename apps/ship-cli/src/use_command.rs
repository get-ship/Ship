//! `ship use --launch` — dispatch a single agent session.
//!
//! After the existing compile-and-activate flow runs in `main.rs::run_use`,
//! this module forwards launch-bearing invocations to the daemon's
//! `POST /api/v1/agent_sessions/dispatch` endpoint.

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

/// Inputs forwarded from the CLI `Use` variant for the launch path.
///
/// The three dispatch axes (detach / transport / keepalive) replace
/// the old single `auto: bool`. See the agent-primitive-inversion
/// plan in `.artifacts/agents-arc/discussions/` for the rationale.
pub struct LaunchOpts {
    pub agent: String,
    pub path: Option<PathBuf>,
    pub launch: bool,
    pub resume: bool,
    pub prompt: Option<String>,
    pub spec: Option<PathBuf>,
    pub harness: Option<String>,
    pub model: Option<String>,

    // Dispatch axes
    /// -d / --detach. Background dispatch — no foreground tab/PTY
    /// attach. Returns a slug + studio_url.
    pub detach: bool,
    /// --transport <pty|print|stream>. None = pty (default).
    pub transport: Option<String>,
    /// -i / --keepalive. Default true for foreground, false for
    /// detached. `no_keepalive` overrides to false.
    pub keepalive: bool,
    pub no_keepalive: bool,
}

/// True when any flag in the launch family is set. Implements the
/// implicit-launch contract: `--detach`, `--resume`, `--prompt`,
/// `--spec`, `--transport`, `--keepalive` each turn `ship use` into
/// a dispatch.
pub fn should_launch(opts: &LaunchOpts) -> bool {
    opts.launch
        || opts.detach
        || opts.resume
        || opts.prompt.is_some()
        || opts.spec.is_some()
        || opts.transport.is_some()
        || opts.keepalive
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

    // The three orthogonal dispatch axes, derived from the new flag
    // shape. Each one defaults to the no-surprise value for the
    // operator's intent (chat in foreground, batch when detached).
    //
    // open_tab: --detach → false, otherwise true
    // invocation: --transport overrides, defaults to "pty"
    // keepalive: --keepalive → true, --no-keepalive → false,
    //            otherwise true when foreground / false when detached
    let open_tab = !opts.detach;
    let invocation = opts
        .transport
        .as_deref()
        .unwrap_or("pty")
        .to_string();
    let keepalive = if opts.keepalive {
        true
    } else if opts.no_keepalive {
        false
    } else {
        // Default: foreground keeps alive (chat); detached doesn't (batch).
        open_tab
    };

    if invocation != "pty" && invocation != "print" && invocation != "stream" {
        bail!(
            "--transport must be one of: pty, print, stream (got `{invocation}`)"
        );
    }

    let mut body = serde_json::json!({
        "agent": opts.agent,
        "open_tab": open_tab,
        "invocation": invocation,
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
    let mode = mode_label(&opts);
    let resp = post_dispatch(&body)?;

    let ok = resp.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    if !ok {
        let err = resp
            .get("data")
            .and_then(|d| d.as_str())
            .unwrap_or("unknown error");
        bail!("dispatch rejected: {err}");
    }

    let data = resp.get("data");
    let slug = data
        .and_then(|d| d.get("slug"))
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let studio_url = data
        .and_then(|d| d.get("studio_url"))
        .and_then(|v| v.as_str());
    let billing_warning = data
        .and_then(|d| d.get("billing_warning"))
        .and_then(|v| v.as_str());

    println!("→ session: {slug} ({mode})");
    if let Some(url) = studio_url {
        println!("→ open in Studio: {url}");
    }
    println!("→ tail: ship procs --slug {slug}");
    if let Some(warn) = billing_warning {
        eprintln!("\n⚠  {warn}");
    }
    Ok(())
}

/// Operator-friendly summary of the dispatch axes — one word that
/// captures the (open_tab × invocation × keepalive) intent.
fn mode_label(opts: &LaunchOpts) -> &'static str {
    let detached = opts.detach;
    let print = opts.transport.as_deref() == Some("print");
    let stream = opts.transport.as_deref() == Some("stream");
    let ka = opts.keepalive || (!opts.no_keepalive && !detached);
    match (detached, print, stream, ka) {
        (false, _, _, _) => "foreground-chat",
        (true, true, _, _) => "background-api",
        (true, _, true, _) => "background-stream",
        (true, _, _, true) => "background-worker",
        (true, _, _, false) => "background-batch",
    }
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
