/// Integration tests — tool_gate contextual flows.
///
/// The unit tests in `tool_gate_tests.rs` already cover every individual rule
/// in isolation. These integration tests add:
///   - Context variation: daemon-present vs daemon-absent
///   - Home-directory resolution: tilde expansion with non-default home
///   - Multi-rule interaction: redirect rule fires independent of verb
///   - Deep nesting: shell wrapper beyond MAX_DEPTH is denied
///   - Gate allows unknown tool names unconditionally
///   - Gate allows Bash with no command field (missing input key)
use std::path::PathBuf;

use crate::hooks::tool_gate::{check, Context, Decision};

fn ctx_with_home(home: &str) -> Context {
    Context {
        home_dir: PathBuf::from(home),
        daemon_port: Some(7777),
    }
}

fn ctx_no_daemon(home: &str) -> Context {
    Context {
        home_dir: PathBuf::from(home),
        daemon_port: None,
    }
}

fn write_payload(path: &str) -> String {
    serde_json::json!({
        "tool_name": "Write",
        "tool_input": { "file_path": path },
    })
    .to_string()
}

fn bash_payload(cmd: &str) -> String {
    serde_json::json!({
        "tool_name": "Bash",
        "tool_input": { "command": cmd },
    })
    .to_string()
}

// ── Context variation: daemon port ────────────────────────────────────────────

#[test]
fn hooks_tool_gate_daemon_present_blocks_curl_to_loopback() {
    let ctx = ctx_with_home("/home/u");
    let d = check(&bash_payload("curl http://localhost:7777/api"), &ctx);
    assert!(matches!(d, Decision::Deny(_)),
        "curl to daemon port must be denied when daemon is present; got {d:?}");
}

#[test]
fn hooks_tool_gate_daemon_absent_skips_http_rule() {
    let ctx = ctx_no_daemon("/home/u");
    // Same URL — allowed because daemon_port is None.
    let d = check(&bash_payload("curl http://localhost:7777/api"), &ctx);
    assert_eq!(d, Decision::Allow,
        "curl to daemon port must be allowed when no daemon is running");
}

#[test]
fn hooks_tool_gate_daemon_present_allows_external_curl() {
    let ctx = ctx_with_home("/home/u");
    let d = check(&bash_payload("curl https://api.example.com/v1/data"), &ctx);
    assert_eq!(d, Decision::Allow,
        "curl to external URL is allowed even when daemon is present");
}

// ── Home directory resolution ─────────────────────────────────────────────────

#[test]
fn hooks_tool_gate_tilde_resolved_against_context_home() {
    // Home is /srv/agent — tilde should expand to /srv/agent/.ship/data/...
    let ctx = ctx_with_home("/srv/agent");
    let d = check(&write_payload("~/.ship/data/events.db"), &ctx);
    assert!(matches!(d, Decision::Deny(_)),
        "tilde must expand to context home_dir for protected path detection; got {d:?}");
}

#[test]
fn hooks_tool_gate_tilde_only_home_is_safe() {
    // Just "~" alone should not match any protected segment.
    let ctx = ctx_with_home("/home/u");
    let d = check(&write_payload("~"), &ctx);
    // "~" itself is not a protected path (no segment match); allow.
    assert_eq!(d, Decision::Allow,
        "bare ~ with no protected segment must be allowed");
}

// ── Redirect rule (Rule 2): fires regardless of verb ─────────────────────────

#[test]
fn hooks_tool_gate_redirect_to_ship_data_denied_any_verb() {
    let ctx = ctx_with_home("/home/u");
    // "ls" is not in SHIP_TOUCHING_VERBS, but the redirect > still hits Rule 2.
    let d = check(&bash_payload("ls foo > ~/.ship/data/out.txt"), &ctx);
    assert!(matches!(d, Decision::Deny(_)),
        "redirect to protected path must be denied regardless of the leading verb; got {d:?}");
}

#[test]
fn hooks_tool_gate_append_redirect_to_ship_machine_denied() {
    let ctx = ctx_with_home("/home/u");
    let d = check(&bash_payload("echo token >> ~/.ship/machine/daemon/token"), &ctx);
    assert!(matches!(d, Decision::Deny(_)),
        ">> append redirect to protected path must also be denied; got {d:?}");
}

// ── Shell wrapper nesting ─────────────────────────────────────────────────────

#[test]
fn hooks_tool_gate_deeply_nested_shell_wrapper_denied_fail_closed() {
    let ctx = ctx_with_home("/home/u");
    // 5 levels of nesting exceeds MAX_DEPTH (4); must fail-closed.
    let inner = r#"cp ~/.ship/machine/daemon/token /tmp/t"#;
    let l1 = format!(r#"bash -c "bash -c \"bash -c 'bash -c \"{inner}\"'\""#);
    let d = check(&bash_payload(&l1), &ctx);
    assert!(matches!(d, Decision::Deny(_)),
        "deeply nested shell wrappers must be denied fail-closed; got {d:?}");
}

// ── Write tool variants ───────────────────────────────────────────────────────

#[test]
fn hooks_tool_gate_edit_tool_protected_path_denied() {
    let ctx = ctx_with_home("/home/u");
    let payload = serde_json::json!({
        "tool_name": "Edit",
        "tool_input": { "file_path": "~/.ship/data/events.db" },
    })
    .to_string();
    let d = check(&payload, &ctx);
    assert!(matches!(d, Decision::Deny(_)),
        "Edit tool on protected path must be denied; got {d:?}");
}

#[test]
fn hooks_tool_gate_multiedit_tool_protected_path_denied() {
    let ctx = ctx_with_home("/home/u");
    let payload = serde_json::json!({
        "tool_name": "MultiEdit",
        "tool_input": { "file_path": "~/.ship/projections/foo" },
    })
    .to_string();
    let d = check(&payload, &ctx);
    assert!(matches!(d, Decision::Deny(_)),
        "MultiEdit tool on protected path must be denied; got {d:?}");
}

// ── Unknown tool and missing input ────────────────────────────────────────────

#[test]
fn hooks_tool_gate_read_tool_always_allowed() {
    let ctx = ctx_with_home("/home/u");
    let payload = serde_json::json!({
        "tool_name": "Read",
        "tool_input": { "file_path": "~/.ship/machine/daemon/token" },
    })
    .to_string();
    let d = check(&payload, &ctx);
    assert_eq!(d, Decision::Allow,
        "Read tool is not in WRITE_TOOLS and must always be allowed");
}

#[test]
fn hooks_tool_gate_bash_missing_tool_input_allowed() {
    let ctx = ctx_with_home("/home/u");
    let payload = serde_json::json!({
        "tool_name": "Bash",
    })
    .to_string();
    let d = check(&payload, &ctx);
    assert_eq!(d, Decision::Allow,
        "Bash with no tool_input must be allowed (nothing to inspect)");
}

#[test]
fn hooks_tool_gate_write_missing_tool_input_allowed() {
    let ctx = ctx_with_home("/home/u");
    let payload = serde_json::json!({
        "tool_name": "Write",
    })
    .to_string();
    let d = check(&payload, &ctx);
    assert_eq!(d, Decision::Allow,
        "Write with no tool_input must be allowed (no path to check)");
}

// ── .claude/ path boundary ────────────────────────────────────────────────────

#[test]
fn hooks_tool_gate_claude_settings_local_denied() {
    let ctx = ctx_with_home("/home/u");
    let d = check(&write_payload(".claude/settings.local.json"), &ctx);
    assert!(matches!(d, Decision::Deny(_)),
        "settings.local.json under .claude/ must be denied; got {d:?}");
}

#[test]
fn hooks_tool_gate_claude_skills_allowed() {
    let ctx = ctx_with_home("/home/u");
    let d = check(&write_payload(".claude/skills/my-skill/SKILL.md"), &ctx);
    assert_eq!(d, Decision::Allow,
        "skills under .claude/ are author-editable and must be allowed");
}

#[test]
fn hooks_tool_gate_claude_commands_allowed() {
    let ctx = ctx_with_home("/home/u");
    let d = check(&write_payload(".claude/commands/my-cmd.md"), &ctx);
    assert_eq!(d, Decision::Allow,
        "commands under .claude/ are author-editable and must be allowed");
}
