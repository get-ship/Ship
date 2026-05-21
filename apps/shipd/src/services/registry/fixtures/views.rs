//! Fixtures for the small daemon-assembled domains: chat, sessions, events,
//! artifacts, workspaces, settings.

use serde_json::{json, Value};

use crate::services::registry::wire::{
    ArtifactsListResponse, ChatStateResponse, EventsListResponse, SessionsListResponse,
    WorkspaceActiveResponse, WorkspaceRow, WorkspacesListResponse,
};
use crate::services::settings::view_state::{
    AboutInfo, FeatureFlag, ProviderRow, SelectOption, SettingsCtx, TerminalConfig,
};
use crate::views::chat_state::{ChatMessage, Sender};
use crate::views::events_state::{EntityGroup, EventRow};
use ship_platform::artifacts::ArtifactRecord;
use ship_platform::services::sessions::types::{SessionKind, SessionMeta, SessionStatus};

fn fixed_dt() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(1_767_225_600, 0).expect("valid timestamp")
}

fn chat() -> ChatStateResponse {
    ChatStateResponse {
        workspace_id: "ws-default".to_string(),
        messages: vec![ChatMessage {
            id: "evt-1".to_string(),
            sender: Sender::Agent,
            content: "Migration complete.".to_string(),
            detail: None,
            timestamp: "09:04".to_string(),
            event_type: "studio.chat_turn".to_string(),
            workspace_id: "ws-default".to_string(),
        }],
        total: 1,
    }
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

/// Shared `EventRow` builder — reused by the dashboard fixture's events feed.
pub(super) fn event_row() -> EventRow {
    EventRow {
        id: "evt-1".to_string(),
        event_type: "pipeline.run.merged".to_string(),
        entity_id: "typed-ops-a1b2c3d".to_string(),
        actor: "rust-lane".to_string(),
        workspace_id: "ws-default".to_string(),
        session_id: "sess-1".to_string(),
        timestamp: "2026-01-01T09:04:12Z".to_string(),
        payload_pretty: "{\n  \"slug\": \"typed-ops-a1b2c3d\"\n}".to_string(),
    }
}

fn events() -> EventsListResponse {
    EventsListResponse {
        groups: vec![EntityGroup {
            entity_id: "typed-ops-a1b2c3d".to_string(),
            count: 1,
            collapsed: false,
            rows: vec![event_row()],
        }],
        types: vec!["pipeline.run.merged".to_string()],
        actors: vec!["rust-lane".to_string()],
    }
}

fn artifacts() -> ArtifactsListResponse {
    let rec = ArtifactRecord {
        id: "art-1".to_string(),
        name: "report.md".to_string(),
        mime: "text/markdown".to_string(),
        size: 2048,
        path: "artifacts/report.md".to_string(),
        scope: "project".to_string(),
        project_id: Some("ws-default".to_string()),
        run_id: Some("typed-ops-a1b2c3d".to_string()),
        user_id: None,
        content_hash: Some("sha256:abc123".to_string()),
        expires_at: None,
        owner_actor: Some("rust-lane".to_string()),
        collection_id: None,
        item_id: None,
        created_at: "2026-01-01T09:00:00Z".to_string(),
    };
    ArtifactsListResponse { items: vec![rec], count: 1, scope: "project".to_string() }
}

fn workspaces() -> WorkspacesListResponse {
    WorkspacesListResponse {
        items: vec![WorkspaceRow {
            id: "ws-default".to_string(),
            name: "ship".to_string(),
            path: "/home/dev/ship".to_string(),
            created_at: json!("2026-01-01T08:00:00Z"),
            last_seen_at: json!("2026-01-01T09:04:12Z"),
            active: true,
        }],
        active: Some("ws-default".to_string()),
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
        ("chat.state", v(chat())),
        ("sessions.list", v(sessions())),
        ("events.list", v(events())),
        ("artifacts.list", v(artifacts())),
        ("workspaces.list", v(workspaces())),
        ("workspaces.active", v(WorkspaceActiveResponse { id: Some("ws-default".to_string()) })),
        ("settings.state", v(SettingsCtx::example())),
    ]
}
