//! MCP bridge for the `@ship/sui-lsp` TypeScript analyzers.
//!
//! Tools — `sui_diagnostics`, `sui_hover`, `sui_dependencies` — spawn `bun`
//! against the one-shot CLI at `packages/sui-lsp/src/cli.ts` and parse its
//! JSON output. Agents that can't speak LSP get the same feedback the VS
//! Code extension surfaces without embedding an LSP client.

use anyhow::{Result, anyhow};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Locate `packages/sui-lsp/src/cli.ts` relative to the project workspace.
///
/// Lookup order:
///   1. `SHIP_WORKSPACE` env var.
///   2. Walk upward from CWD looking for `pnpm-workspace.yaml`.
pub fn resolve_cli_path() -> Result<PathBuf> {
    if let Ok(ws) = std::env::var("SHIP_WORKSPACE") {
        let candidate = PathBuf::from(ws).join("packages/sui-lsp/src/cli.ts");
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    let cwd = std::env::current_dir().map_err(|e| anyhow!("cwd unavailable: {e}"))?;
    if let Some(root) = find_workspace_root(&cwd) {
        let candidate = root.join("packages/sui-lsp/src/cli.ts");
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(anyhow!(
        "sui-lsp CLI not found; set SHIP_WORKSPACE to the repo root, or run from a directory inside the workspace"
    ))
}

fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    loop {
        if cur.join("pnpm-workspace.yaml").is_file() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

/// Resolve a user-supplied path against the workspace root if it isn't absolute.
fn resolve_file_arg(file: &str, workspace_root: &Path) -> PathBuf {
    let p = PathBuf::from(file);
    if p.is_absolute() { p } else { workspace_root.join(p) }
}

/// Run the CLI and return parsed JSON stdout. Stderr/non-zero exits surface
/// as `Err` with a clear message.
async fn run_cli(args: &[&str]) -> Result<Value> {
    let cli = resolve_cli_path()?;
    let workspace_root = cli
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow!("workspace root unresolvable from cli path"))?
        .to_path_buf();
    let mut cmd = Command::new("bun");
    cmd.arg(cli.as_os_str());
    cmd.args(args);
    cmd.current_dir(&workspace_root);
    let output = cmd
        .output()
        .await
        .map_err(|e| anyhow!("failed to spawn bun: {e}. Is bun installed and on PATH?"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow!(
            "sui-lsp CLI exited with status {}: stderr={}, stdout={}",
            output.status,
            stderr.trim(),
            stdout.trim(),
        ));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| anyhow!("CLI stdout was not UTF-8: {e}"))?;
    let parsed: Value = serde_json::from_str(stdout.trim())
        .map_err(|e| anyhow!("CLI produced invalid JSON: {e}; raw={}", stdout.trim()))?;
    Ok(parsed)
}

/// Pre-flight: validate file existence inside the workspace and return its
/// resolved absolute path. The CLI will also return its own file-not-found
/// error, but catching it here gives a cleaner message and avoids spawning
/// bun for an obvious mistake.
fn validate_file(file: &str) -> Result<(PathBuf, PathBuf)> {
    let cli = resolve_cli_path()?;
    let workspace_root = cli
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow!("workspace root unresolvable from cli path"))?
        .to_path_buf();
    let abs = resolve_file_arg(file, &workspace_root);
    if !abs.exists() {
        return Err(anyhow!("file not found: {} (resolved from {file})", abs.display()));
    }
    Ok((abs, workspace_root))
}

pub async fn sui_diagnostics(file: &str) -> String {
    match diagnostics_inner(file).await {
        Ok(s) => s,
        Err(e) => format!("Error: {e}"),
    }
}

async fn diagnostics_inner(file: &str) -> Result<String> {
    let (abs, _) = validate_file(file)?;
    let abs_str = abs.to_string_lossy().to_string();
    let value = run_cli(&["diagnostics", &abs_str]).await?;
    Ok(serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()))
}

pub async fn sui_hover(file: &str, line: u32, col: u32) -> String {
    match hover_inner(file, line, col).await {
        Ok(s) => s,
        Err(e) => format!("Error: {e}"),
    }
}

async fn hover_inner(file: &str, line: u32, col: u32) -> Result<String> {
    let (abs, _) = validate_file(file)?;
    let abs_str = abs.to_string_lossy().to_string();
    let line_str = line.to_string();
    let col_str = col.to_string();
    let value = run_cli(&[
        "hover",
        &abs_str,
        "--line",
        &line_str,
        "--col",
        &col_str,
    ])
    .await?;
    Ok(serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()))
}

/// Report the surface a `.sui` view depends on: custom-element tags,
/// signal/scope references, on:* events, bind:* targets, and the resolved
/// paired-controller path.
pub async fn sui_dependencies(file: &str) -> String {
    match dependencies_inner(file).await {
        Ok(s) => s,
        Err(e) => format!("Error: {e}"),
    }
}

async fn dependencies_inner(file: &str) -> Result<String> {
    let (abs, _) = validate_file(file)?;
    let abs_str = abs.to_string_lossy().to_string();
    let value = run_cli(&["dependencies", &abs_str]).await?;
    Ok(serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_cli_path_finds_workspace() {
        // The test crate is inside the workspace; resolve should succeed.
        let path = resolve_cli_path().expect("cli path resolves");
        assert!(path.ends_with("packages/sui-lsp/src/cli.ts"));
        assert!(path.exists(), "resolved path should exist: {}", path.display());
    }

    #[test]
    fn resolve_file_arg_keeps_absolute_paths() {
        let ws = PathBuf::from("/tmp/ws");
        let abs = PathBuf::from("/etc/hosts");
        assert_eq!(resolve_file_arg("/etc/hosts", &ws), abs);
    }

    #[test]
    fn resolve_file_arg_joins_relative_to_workspace() {
        let ws = PathBuf::from("/tmp/ws");
        assert_eq!(
            resolve_file_arg("packages/x/y.sui", &ws),
            PathBuf::from("/tmp/ws/packages/x/y.sui"),
        );
    }

    #[test]
    fn validate_file_errors_clearly_on_missing() {
        let err = validate_file("definitely/does/not/exist.sui").expect_err("missing file errors");
        let msg = format!("{err}");
        assert!(msg.contains("file not found"), "got: {msg}");
    }

    #[tokio::test]
    async fn diagnostics_happy_path_returns_json() {
        // sui-button.sui is checked into the workspace and ships with
        // known diagnostics under the current ruleset.
        let out = sui_diagnostics("packages/ui-components/src/sui-button/sui-button.sui").await;
        assert!(!out.starts_with("Error:"), "expected JSON output, got: {out}");
        let v: Value = serde_json::from_str(&out).expect("output parses as JSON");
        assert!(v.get("diagnostics").is_some(), "missing diagnostics key in {out}");
    }

    #[tokio::test]
    async fn dependencies_returns_tags_and_events() {
        // views/agents/page.sui depends on many sui-* tags + handles events.
        let out = sui_dependencies("views/agents/page.sui").await;
        assert!(!out.starts_with("Error:"), "expected JSON output, got: {out}");
        let v: Value = serde_json::from_str(&out).expect("output parses as JSON");
        let tags = v.get("tags").and_then(|t| t.as_array()).expect("tags array");
        assert!(
            tags.iter().any(|t| t.as_str() == Some("sui-button")),
            "expected sui-button in tags, got: {out}"
        );
        let events = v.get("events").and_then(|e| e.as_array()).expect("events array");
        assert!(
            events.iter().any(|e| e.as_str() == Some("click")),
            "expected click in events, got: {out}"
        );
    }
}
