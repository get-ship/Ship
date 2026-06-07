//! Unit tests for the PreToolUse integrity gate. Lives in a sibling file
//! so `tool_gate.rs` stays under the project file-size cap. Wired in via
//! `#[path = "tool_gate_tests.rs"] mod tests;` at the bottom of
//! `tool_gate.rs`, so this file behaves as the standard
//! `#[cfg(test)] mod tests` block of the parent module.

use super::*;

fn ctx() -> Context {
    Context {
        home_dir: PathBuf::from("/home/u"),
        daemon_port: Some(9876),
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

#[test]
fn write_to_ship_machine_denied() {
    let d = check(&write_payload("~/.ship/machine/daemon/token"), &ctx());
    assert!(matches!(d, Decision::Deny(_)), "got {d:?}");
}

#[test]
fn write_to_ship_data_denied_absolute_path() {
    let d = check(&write_payload("/home/u/.ship/data/events.db"), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn write_to_ship_projections_denied() {
    let d = check(&write_payload("~/.ship/projections/x"), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn write_to_dot_git_denied() {
    let d = check(&write_payload(".git/refs/heads/main"), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn write_to_dot_claude_settings_denied() {
    let d = check(&write_payload(".claude/settings.json"), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn write_to_dot_claude_local_settings_denied() {
    let d = check(&write_payload(".claude/settings.local.json"), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn write_to_dot_claude_hooks_dir_denied() {
    let d = check(&write_payload(".claude/hooks/before.sh"), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn write_to_dot_claude_skill_allowed() {
    let d = check(&write_payload(".claude/skills/ship-sdk/SKILL.md"), &ctx());
    assert_eq!(d, Decision::Allow);
}

#[test]
fn write_to_dot_claude_command_allowed() {
    let d = check(&write_payload(".claude/commands/foo.md"), &ctx());
    assert_eq!(d, Decision::Allow);
}

#[test]
fn write_to_gitignore_allowed() {
    let d = check(&write_payload(".gitignore"), &ctx());
    assert_eq!(d, Decision::Allow);
}

#[test]
fn write_to_ship_library_allowed() {
    let d = check(&write_payload("~/.ship/library/skills/foo.md"), &ctx());
    assert_eq!(d, Decision::Allow);
}

#[test]
fn write_to_ship_datasets_allowed() {
    let d = check(&write_payload("~/.ship/datasets/foo"), &ctx());
    assert_eq!(d, Decision::Allow);
}

#[test]
fn ordinary_write_allowed() {
    let d = check(&write_payload("/tmp/scratch.txt"), &ctx());
    assert_eq!(d, Decision::Allow);
}

#[test]
fn bash_tar_ship_data_denied() {
    let d = check(
        &bash_payload("tar -czf /tmp/leak.tgz ~/.ship/data/"),
        &ctx(),
    );
    assert!(matches!(d, Decision::Deny(_)), "got {d:?}");
}

#[test]
fn bash_cp_ship_machine_denied() {
    let d = check(&bash_payload("cp ~/.ship/machine/daemon/token /tmp/t"), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn bash_redirect_to_ship_data_denied() {
    let d = check(&bash_payload("echo corrupt > ~/.ship/data/events.db"), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn bash_base64_read_machine_denied() {
    let d = check(
        &bash_payload(r#"bash -c "base64 $HOME/.ship/machine/daemon/token""#),
        &ctx(),
    );
    assert!(matches!(d, Decision::Deny(_)), "got {d:?}");
}

#[test]
fn bash_chmod_ship_machine_denied() {
    let d = check(&bash_payload("chmod 0777 ~/.ship/machine/daemon/token"), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn bash_curl_localhost_daemon_denied() {
    let d = check(&bash_payload("curl http://localhost:9876/api/v1/whoami"), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn bash_curl_loopback_ip_denied() {
    let d = check(&bash_payload("curl http://127.0.0.1:9876/api"), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn bash_curl_other_port_allowed() {
    let d = check(&bash_payload("curl http://127.0.0.1:8080/api"), &ctx());
    assert_eq!(d, Decision::Allow);
}

#[test]
fn bash_wget_external_url_allowed() {
    let d = check(&bash_payload("wget https://example.com/file.txt"), &ctx());
    assert_eq!(d, Decision::Allow);
}

#[test]
fn bash_sudo_prefix_does_not_hide_violation() {
    let d = check(&bash_payload("sudo cp ~/.ship/data/events.db /tmp/x"), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn bash_env_prefix_does_not_hide_violation() {
    let d = check(
        &bash_payload("env DEBUG=1 cp ~/.ship/data/events.db /tmp/x"),
        &ctx(),
    );
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn bash_ordinary_command_allowed() {
    let d = check(&bash_payload("cargo build -p compiler"), &ctx());
    assert_eq!(d, Decision::Allow);
}

#[test]
fn bash_grep_in_dot_git_log_allowed() {
    let d = check(&bash_payload("grep -r token src/"), &ctx());
    assert_eq!(d, Decision::Allow);
}

#[test]
fn malformed_json_denied_fail_closed() {
    let d = check("{not json", &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn unparseable_bash_denied_fail_closed() {
    let d = check(&bash_payload(r#"echo "unterminated"#), &ctx());
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn unknown_tool_allowed() {
    let payload = serde_json::json!({
        "tool_name": "Read",
        "tool_input": { "file_path": "~/.ship/machine/x" },
    })
    .to_string();
    assert_eq!(check(&payload, &ctx()), Decision::Allow);
}

#[test]
fn cursor_camelcase_fields_recognized() {
    let payload = serde_json::json!({
        "toolName": "Write",
        "toolInput": { "filePath": "~/.ship/machine/x" },
    })
    .to_string();
    assert!(matches!(check(&payload, &ctx()), Decision::Deny(_)));
}

#[test]
fn no_daemon_port_skips_http_rule() {
    let ctx = Context {
        home_dir: PathBuf::from("/home/u"),
        daemon_port: None,
    };
    let d = check(&bash_payload("curl http://127.0.0.1:9876/api"), &ctx);
    assert_eq!(d, Decision::Allow);
}

#[test]
fn empty_bash_command_allowed() {
    let d = check(&bash_payload(""), &ctx());
    assert_eq!(d, Decision::Allow);
}

#[test]
fn bash_eval_inner_denied() {
    let d = check(
        &bash_payload(r#"eval "cp ~/.ship/machine/daemon/token /tmp/t""#),
        &ctx(),
    );
    assert!(matches!(d, Decision::Deny(_)));
}

#[test]
fn bash_sh_dash_c_inner_denied() {
    let d = check(
        &bash_payload(r#"sh -c "tar -czf /tmp/x ~/.ship/data/""#),
        &ctx(),
    );
    assert!(matches!(d, Decision::Deny(_)));
}
