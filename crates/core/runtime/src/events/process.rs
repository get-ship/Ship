//! Process-lifecycle events. Distinct from `job.*` because a single
//! job can spawn many processes (per step), and a process is an OS
//! resource the supervisor must observe directly.

use serde::{Deserialize, Serialize};
use specta::Type;

pub mod event_types {
    pub const PROCESS_SPAWNED: &str = "process.spawned";
    pub const PROCESS_EXITED: &str = "process.exited";
    pub const PROCESS_SIGNAL_SENT: &str = "process.signal_sent";
    pub const PROCESS_ORPHANED: &str = "process.orphaned";
    pub const PROCESS_KILLED: &str = "process.killed";
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProcessSpawnedPayload {
    /// Slug of the job that owns this process.
    pub slug: String,
    pub job_id: String,
    /// PID of the spawned process. None for tmux-detached processes.
    pub pid: Option<u32>,
    /// Binary name (e.g. "claude").
    pub cmd: String,
    /// Sanitized argv — secrets redacted.
    pub argv: Vec<String>,
    /// Environment variable names only (not values). Filtered to
    /// `SHIP_*` and a small allowlist.
    pub env_keys: Vec<String>,
    pub cwd: String,
    /// "process" | "pty" | "tmux"
    pub executor_kind: String,
    /// None for non-pipeline jobs.
    pub step_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProcessExitedPayload {
    pub slug: String,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    /// "SIGTERM", "SIGKILL", etc; None for clean exit.
    pub signal: Option<String>,
    pub wall_duration_ms: u64,
    pub executor_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProcessSignalSentPayload {
    pub slug: String,
    pub pid: Option<u32>,
    /// "SIGTERM" | "SIGKILL" | "SIGINT"
    pub signum: String,
    /// "shutdown" | "cancellation" | "timeout"
    pub reason: String,
    /// "operator" | "supervisor"
    pub actor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProcessOrphanedPayload {
    pub pid: u32,
    pub cmd: String,
    pub last_known_slug: Option<String>,
    /// "sweep" | "startup_scan"
    pub detected_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProcessKilledPayload {
    /// Slug of the job whose process was killed.
    pub slug: String,
    /// PID at time of kill, if known. None when the registry was
    /// empty and the kill went via tmux session destruction.
    pub pid: Option<u32>,
    /// Signal sent (e.g. "SIGTERM", "SIGKILL", "tmux_kill_session").
    pub signal: String,
    /// Why the process was killed: "cancellation", "shutdown",
    /// "timeout", "force", "supervisor_sweep".
    pub reason: String,
    /// Who initiated: "operator", "supervisor", "policy:<name>".
    pub actor: String,
}

// ── Sanitization helpers ────────────────────────────────────────────────────

/// Regex-free secret pattern matching for argv sanitization.
fn is_secret_key(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower.contains("password")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("apikey")
        || lower.contains("api-key")
        || lower.contains("api_key")
        || lower.contains("key=")
}

/// Redact argv entries that look like secrets.
///
/// Two forms handled:
/// - `--api-key=VALUE` → `--api-key=<redacted>`
/// - `--api-key VALUE` → caller passes each element separately;
///   we flag the key and the NEXT call sees a bare value after a flagged key.
///   For simplicity, we only handle the `=` form inline and strip bare
///   values that match the secret pattern entirely.
pub fn sanitize_argv(argv: &[String]) -> Vec<String> {
    argv.iter()
        .map(|arg| {
            if let Some(eq_pos) = arg.find('=') {
                let key_part = &arg[..eq_pos];
                if is_secret_key(key_part) {
                    return format!("{key_part}=<redacted>");
                }
            }
            if is_secret_key(arg) && !arg.starts_with('-') {
                return "<redacted>".to_string();
            }
            arg.clone()
        })
        .collect()
}

const ENV_ALLOWLIST: &[&str] = &["PATH", "HOME", "USER", "PWD"];

/// Filter env keys to SHIP_* prefix and the small allowlist.
pub fn filter_env_keys(keys: &[String]) -> Vec<String> {
    keys.iter()
        .filter(|k| k.starts_with("SHIP_") || ENV_ALLOWLIST.contains(&k.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_argv_redacts_secrets() {
        let argv = vec![
            "--name".to_string(),
            "job-X".to_string(),
            "--api-key=secret123".to_string(),
            "--token=abc".to_string(),
            "--verbose".to_string(),
        ];
        let result = sanitize_argv(&argv);
        assert_eq!(result[0], "--name");
        assert_eq!(result[1], "job-X");
        assert_eq!(result[2], "--api-key=<redacted>");
        assert_eq!(result[3], "--token=<redacted>");
        assert_eq!(result[4], "--verbose");
    }

    #[test]
    fn sanitize_argv_handles_password_variations() {
        let argv = vec![
            "--password=hunter2".to_string(),
            "--db-secret=x".to_string(),
        ];
        let result = sanitize_argv(&argv);
        assert_eq!(result[0], "--password=<redacted>");
        assert_eq!(result[1], "--db-secret=<redacted>");
    }

    #[test]
    fn filter_env_keys_keeps_allowed() {
        let keys = vec![
            "SHIP_ACTOR_ID".to_string(),
            "SHIP_AGENT_ID".to_string(),
            "PATH".to_string(),
            "HOME".to_string(),
            "AWS_SECRET_ACCESS_KEY".to_string(),
            "DATABASE_URL".to_string(),
        ];
        let result = filter_env_keys(&keys);
        assert_eq!(result, vec!["SHIP_ACTOR_ID", "SHIP_AGENT_ID", "PATH", "HOME"]);
    }
}
