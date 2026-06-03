//! Shell-quoting for `LaunchCommand` flattening (tmux send-keys).
//!
//! Split out of `dispatch_ports.rs` to keep that file under the size cap; the
//! `LaunchCommand` type itself stays in the parent as a core port type, this
//! submodule carries its `to_shell_string` impl plus the quoting helper.

use super::LaunchCommand;

impl LaunchCommand {
    /// Flatten to a single shell command string (for tmux send-keys).
    /// Arguments and env values containing shell-significant characters
    /// are single-quoted so prompts like `"Read spec.md and execute it."`
    /// survive intact rather than being word-split by the shell.
    pub fn to_shell_string(&self) -> String {
        let mut parts = Vec::new();
        for (k, v) in &self.env {
            parts.push(format!("{k}={}", shell_quote(v)));
        }
        parts.push(shell_quote(&self.bin));
        parts.extend(self.args.iter().map(|a| shell_quote(a)));
        parts.join(" ")
    }
}

/// Quote a string for safe POSIX-sh consumption. Strings containing only
/// shell-safe characters pass through unchanged; everything else is
/// wrapped in single quotes with inner single quotes escaped as `'\''`.
fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':' | '@' | '=' | '+' | ',')
        })
    {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

#[cfg(test)]
mod shell_quote_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn safe_strings_pass_through_unquoted() {
        assert_eq!(shell_quote("codex"), "codex");
        assert_eq!(shell_quote("--full-auto"), "--full-auto");
        assert_eq!(shell_quote("/tmp/foo.txt"), "/tmp/foo.txt");
        assert_eq!(shell_quote("KEY=value"), "KEY=value");
    }

    #[test]
    fn empty_string_is_quoted() {
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn strings_with_spaces_are_single_quoted() {
        assert_eq!(
            shell_quote("Read spec.md and execute it."),
            "'Read spec.md and execute it.'",
        );
    }

    #[test]
    fn inner_single_quotes_are_escaped() {
        assert_eq!(shell_quote("it's working"), "'it'\\''s working'");
    }

    #[test]
    fn shell_metachars_are_quoted() {
        assert_eq!(shell_quote("a && b"), "'a && b'");
        assert_eq!(shell_quote("$HOME"), "'$HOME'");
        assert_eq!(shell_quote("foo | bar"), "'foo | bar'");
        assert_eq!(shell_quote("`whoami`"), "'`whoami`'");
    }

    #[test]
    fn to_shell_string_quotes_multiword_prompt() {
        let cmd = LaunchCommand {
            bin: "codex".into(),
            args: vec![
                "exec".into(),
                "--full-auto".into(),
                "Read .artifacts/spec.md and execute it.".into(),
            ],
            env: HashMap::new(),
        };
        assert_eq!(
            cmd.to_shell_string(),
            "codex exec --full-auto 'Read .artifacts/spec.md and execute it.'",
        );
    }

    #[test]
    fn to_shell_string_quotes_env_values_with_spaces() {
        let mut env = HashMap::new();
        env.insert("MSG".to_string(), "hello world".to_string());
        let cmd = LaunchCommand {
            bin: "claude".into(),
            args: vec!["-p".into()],
            env,
        };
        assert!(cmd.to_shell_string().starts_with("MSG='hello world' "));
    }
}
