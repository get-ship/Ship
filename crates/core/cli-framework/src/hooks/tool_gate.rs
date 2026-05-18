//! Runtime Ship-integrity check — defense Layer 2 per `SECURITY.md`.
//!
//! Called from `ship hook before-tool` on every PreToolUse across every
//! supported harness (the compiler injects that command via
//! `inject_ship_lifecycle_hooks`). Pure logic, no I/O. Fail-closed on
//! malformed input. Full rule list + non-defended cases live in
//! `SECURITY.md` Layer 2 — the source of record.

use std::path::{Component, Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny(String),
}

/// Context the gate needs to make a decision. Resolved once per invocation in
/// the caller so the gate itself stays pure.
pub struct Context {
    pub home_dir: PathBuf,
    /// Loopback daemon port — used to detect direct HTTP from the agent
    /// process. `None` when no daemon is running; the daemon-HTTP rule is
    /// then skipped (no target = no attack surface).
    pub daemon_port: Option<u16>,
}

/// Ship-state path segments that must never be modified by an agent. Match is
/// segment-aware: `~/.ship/data/events.db` matches `.ship/data`, but
/// `~/.ship/datasets/` does not.
const PROTECTED_SEGMENTS: &[(&str, &str)] = &[
    (".ship/machine", "Ship credentials / ambient secrets"),
    (".ship/data", "Ship event store (corruption is data loss)"),
    (".ship/projections", "Ship projections"),
];

/// Tool names that the gate inspects for file paths. Anything else falls
/// through to allow — Layer 2 is targeted at write/exec, not read.
const WRITE_TOOLS: &[&str] = &["Write", "Edit", "MultiEdit", "NotebookEdit"];

/// Shell verbs that, paired with a Ship-protected path argument, indicate
/// destructive or exfiltration intent. We don't need an exhaustive list —
/// Layer 1 already covers the obvious surface (`cat ~/.ship/machine/*` etc).
/// These are the constructions Layer 1's literal-glob matcher misses.
const SHIP_TOUCHING_VERBS: &[&str] = &[
    "tar", "cp", "mv", "dd", "rsync", "tee", "chmod", "chown", "ln", "install",
    "base64", "xxd", "od", "hexdump",
    "cat", "head", "tail", "less", "more",
    "rm",
];

/// Commands whose first token, combined with a loopback URL, indicates
/// direct daemon HTTP from the agent process.
const HTTP_CLIENTS: &[&str] = &["curl", "wget", "http", "httpie", "xh"];

/// Inspect a parsed PreToolUse payload and decide whether the call should run.
///
/// `stdin_json` is the raw JSON the harness wrote to the hook's stdin. The
/// shape is provider-specific; we look at the lowest common denominator
/// (`tool_name` + `tool_input`, with camelCase fallbacks for Cursor).
pub fn check(stdin_json: &str, ctx: &Context) -> Decision {
    let value: serde_json::Value = match serde_json::from_str(stdin_json) {
        Ok(v) => v,
        Err(e) => {
            return Decision::Deny(format!(
                "ship hook: malformed input ({e}) — denied (fail-closed)"
            ));
        }
    };
    let tool_name = value
        .get("tool_name")
        .or_else(|| value.get("toolName"))
        .or_else(|| value.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let tool_input = value
        .get("tool_input")
        .or_else(|| value.get("toolInput"))
        .or_else(|| value.get("input"));

    if WRITE_TOOLS.contains(&tool_name) {
        return check_write(tool_input, &ctx.home_dir);
    }
    if tool_name == "Bash" {
        return check_bash(tool_input, ctx);
    }
    Decision::Allow
}

fn check_write(input: Option<&serde_json::Value>, home: &Path) -> Decision {
    let Some(input) = input else { return Decision::Allow };
    let path = input
        .get("file_path")
        .or_else(|| input.get("filePath"))
        .or_else(|| input.get("path"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if path.is_empty() {
        return Decision::Allow;
    }
    classify_path(path, home)
        .map(Decision::Deny)
        .unwrap_or(Decision::Allow)
}

fn check_bash(input: Option<&serde_json::Value>, ctx: &Context) -> Decision {
    let Some(input) = input else { return Decision::Allow };
    let cmd = input.get("command").and_then(|v| v.as_str()).unwrap_or("");
    if cmd.is_empty() {
        return Decision::Allow;
    }
    check_bash_command(cmd, ctx, 0)
}

/// Recursive Bash inspection. `depth` bounds nested shell wrappers so a
/// pathological `bash -c "bash -c '...'"` chain cannot trigger unbounded
/// work inside the hook.
fn check_bash_command(cmd: &str, ctx: &Context, depth: u8) -> Decision {
    const MAX_DEPTH: u8 = 4;
    if depth > MAX_DEPTH {
        return Decision::Deny(
            "ship hook: shell-wrapper nesting exceeded — denied (fail-closed)".into(),
        );
    }
    let tokens = match shell_words::split(cmd) {
        Ok(t) => t,
        Err(_) => {
            return Decision::Deny(
                "ship hook: unparseable Bash command — denied (fail-closed)".into(),
            );
        }
    };
    if tokens.is_empty() {
        return Decision::Allow;
    }

    // Strip a leading `sudo` / `env KEY=v` / `timeout 5s` wrapper so we
    // identify the underlying command. Layer-1 already denies `sudo:*`,
    // but the gate is defense-in-depth, so we still look at what comes after.
    let command_name = strip_prefix_wrappers(&tokens);

    // Rule 0 — recurse into shell wrappers (`bash -c "..."`, `sh -c ...`,
    // `eval "..."`). Without this, an agent could trivially evade the
    // verb-based rules below by inlining the dangerous command in a quoted
    // script string.
    if let Some(inner) = extract_shell_wrapper(&tokens) {
        let d = check_bash_command(inner, ctx, depth + 1);
        if matches!(d, Decision::Deny(_)) {
            return d;
        }
    }

    // Rule 1 — verbs that read/write Ship-owned paths. Substring match on
    // every token (not just args) catches operators like `>` that survive
    // tokenisation as their own token, so `echo X > ~/.ship/data/Y` lands
    // a deny on the third token.
    if SHIP_TOUCHING_VERBS.contains(&command_name) {
        for tok in &tokens {
            if let Some(reason) = classify_path(tok, &ctx.home_dir) {
                return Decision::Deny(reason);
            }
        }
    }

    // Rule 2 — any redirect targeting a protected path, regardless of verb.
    for window in tokens.windows(2) {
        if matches!(window[0].as_str(), ">" | ">>" | "&>" | "&>>")
            && let Some(reason) = classify_path(&window[1], &ctx.home_dir)
        {
            return Decision::Deny(reason);
        }
    }

    // Rule 3 — direct daemon HTTP. The daemon enforces auth on its own, but
    // any HTTP from the agent process is suspicious by default: legitimate
    // Ship operations go through the `ship` CLI or MCP, never raw curl.
    if let Some(port) = ctx.daemon_port
        && HTTP_CLIENTS.contains(&command_name)
    {
        for tok in &tokens[1..] {
            if mentions_daemon(tok, port) {
                return Decision::Deny(format!(
                    "ship hook: blocked direct HTTP to local daemon (:{port}). \
                     Use the `ship` CLI or MCP tools — they authenticate \
                     correctly and audit-log."
                ));
            }
        }
    }

    Decision::Allow
}

/// Returns the inner script string of a `bash -c "..."` / `sh -c "..."` /
/// `eval "..."` wrapper, if the leading tokens match. Used by Rule 0 to
/// recurse into shell-quoted payloads.
fn extract_shell_wrapper(tokens: &[String]) -> Option<&str> {
    let shells = ["bash", "sh", "zsh", "dash", "ash", "ksh"];
    let first = tokens.first()?.as_str();
    if shells.contains(&first) && tokens.get(1).map(String::as_str) == Some("-c") {
        return tokens.get(2).map(String::as_str);
    }
    if first == "eval" {
        return tokens.get(1).map(String::as_str);
    }
    None
}

/// Returns `Some(deny-reason)` when the path lies inside a protected tree.
/// Centralises the substring rules so Write/Edit and Bash share one truth
/// table.
fn classify_path(path: &str, home: &Path) -> Option<String> {
    let expanded = expand_tilde(path, home);
    let lossy = expanded.to_string_lossy();
    for (segment, label) in PROTECTED_SEGMENTS {
        if segment_touches(&lossy, segment) {
            return Some(format!(
                "ship hook: blocked operation on {label} (matched `{segment}` in `{path}`)"
            ));
        }
    }
    if path_has_segment(&expanded, ".git") {
        return Some(format!(
            "ship hook: blocked write inside .git/ (would let an agent rewrite refs) — path `{path}`"
        ));
    }
    if claude_path_is_protected(&expanded) {
        return Some(format!(
            "ship hook: blocked write to .claude/ settings or hooks (would let an agent disable its own hooks) — path `{path}`"
        ));
    }
    None
}

/// True iff a path under `.claude/` targets the agent's own configuration —
/// settings JSON or the hooks directory. Skills, commands, and other content
/// under `.claude/` are author-editable and not gated here.
fn claude_path_is_protected(path: &Path) -> bool {
    let mut components = path.components();
    while let Some(c) = components.next() {
        if matches!(c, Component::Normal(n) if n == ".claude") {
            let rest: Vec<_> = components.collect();
            let first = rest.first().and_then(|c| match c {
                Component::Normal(n) => n.to_str(),
                _ => None,
            });
            return match first {
                Some("hooks") => true,
                Some(name) => name.starts_with("settings") && name.ends_with(".json"),
                None => false,
            };
        }
    }
    false
}

fn strip_prefix_wrappers(tokens: &[String]) -> &str {
    let mut i = 0;
    while i < tokens.len() {
        let t = tokens[i].as_str();
        if t == "sudo" || t == "nohup" || t == "exec" {
            i += 1;
            continue;
        }
        if (t == "env" || t == "timeout") && i + 1 < tokens.len() {
            i += 1;
            // env may carry KEY=value pairs; skip those too.
            while i < tokens.len() && tokens[i].contains('=') {
                i += 1;
            }
            continue;
        }
        break;
    }
    tokens.get(i).map(String::as_str).unwrap_or("")
}

fn segment_touches(haystack: &str, segment: &str) -> bool {
    let with_slash = format!("{segment}/");
    haystack.contains(&with_slash)
        || haystack.ends_with(segment)
        || haystack == segment
}

fn path_has_segment(path: &Path, segment: &str) -> bool {
    path.components()
        .any(|c| matches!(c, Component::Normal(n) if n == segment))
}

fn expand_tilde(input: &str, home: &Path) -> PathBuf {
    if let Some(rest) = input.strip_prefix("~/") {
        home.join(rest)
    } else if input == "~" {
        home.to_path_buf()
    } else {
        PathBuf::from(input)
    }
}

fn mentions_daemon(token: &str, port: u16) -> bool {
    let p = port.to_string();
    let needles = [
        format!("localhost:{p}"),
        format!("127.0.0.1:{p}"),
        format!("[::1]:{p}"),
        format!("0.0.0.0:{p}"),
    ];
    needles.iter().any(|n| token.contains(n))
}

#[cfg(test)]
#[path = "tool_gate_tests.rs"]
mod tests;
