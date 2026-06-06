//! Fixtures for the small daemon-assembled domains: sessions, settings.

use serde_json::Value;

use crate::services::registry::wire::SessionsListResponse;
use crate::services::settings::view_state::{
    AboutInfo, FeatureFlag, ProviderRow, SelectOption, SettingsCtx, TerminalConfig,
};
use ship_agents::services::sessions::types::{SessionKind, SessionMeta, SessionStatus};
fn fixed_dt() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(1_767_225_600, 0).expect("valid timestamp")
}

fn sessions() -> SessionsListResponse {
    SessionsListResponse {
        items: vec![SessionMeta {
            id: "sess-1".to_string(),
            kind: SessionKind::Agent,
            workspace_id: "ws-default".to_string(),
            agent_id: Some("rust-lane".to_string()),
            harness: Some("claude".to_string()),
            pid: Some(4242),
            status: SessionStatus::Idle,
            log_path: "/tmp/ship/sess-1.log".to_string(),
            spawned_at: fixed_dt(),
            last_activity: fixed_dt(),
            exit_code: None,
            harness_session_id: Some("conv-abc".to_string()),
        }],
    }
}


impl SettingsCtx {
    pub fn example() -> Self {
        SettingsCtx {
            harnesses: vec![ProviderRow {
                id: "claude".to_string(),
                label: "Anthropic Claude".to_string(),
                glyph: "\u{2726}".to_string(),
                binary: "claude".to_string(),
                version: Some("1.0.0".to_string()),
                available: true,
                is_default: true,
            }],
            default_harness: "claude".to_string(),
            default_model: "opus".to_string(),
            terminal: TerminalConfig {
                program: "xterm-256color".to_string(),
                worktree_base: "~/dev/ship-worktrees".to_string(),
                options: vec![SelectOption {
                    value: "xterm-256color".to_string(),
                    label: "xterm-256color".to_string(),
                }],
            },
            advanced: vec![FeatureFlag {
                key: "agent_session_auto_resume".to_string(),
                label: "Auto-resume agent sessions".to_string(),
                description: "Continue the last conversation on the next message.".to_string(),
                enabled: true,
            }],
            advanced_registry: "https://registry.ship.dev".to_string(),
            daemon_endpoint: "http://127.0.0.1:4000".to_string(),
            about: AboutInfo {
                ship_version: "0.2.0".to_string(),
                daemon_version: "0.2.0".to_string(),
                cli_version: "0.2.0".to_string(),
                license: "MIT".to_string(),
                homepage: "https://ship.dev".to_string(),
                docs: "https://ship.dev/docs".to_string(),
                source: "https://github.com/ship/ship".to_string(),
            },
            harness_default: "claude".to_string(),
            harness_default_model: "opus".to_string(),
            agent_session_auto_resume: true,
            watcher_debounce_ms: "250".to_string(),
            git_log_limit: "50".to_string(),
            hooks_max_depth: "8".to_string(),
        }
    }
}

pub fn examples() -> Vec<(&'static str, Value)> {
    fn v<T: serde::Serialize>(t: T) -> Value {
        serde_json::to_value(t).expect("example serializes")
    }
    vec![
        ("agents.sessions", v(sessions())),
        ("settings.state", v(SettingsCtx::example())),
    ]
}
