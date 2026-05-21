//! Named wire shapes for shipd-orchestrated registry ops — the daemon-side
//! counterpart to each domain's ship-platform `wire_types.rs`. Used where a
//! response is assembled in the daemon (joining projections, PTY state, view
//! helpers) rather than returned by a pure ship-platform state fn.
//!
//! These satisfy the typed-op single-source contract: every return is a named
//! `Serialize` type, never an anonymous `json!`, and dispatch is byte-identical.
//! Their `data.ts` handles stay `unknown` until the shapes are specta-registered
//! (incremental and additive — see `gen_sdk::data_ts::MIGRATED_RETURN_TYPES`).

use serde::Serialize;

use crate::views::chat_state::ChatMessage;
use crate::views::events_state::EntityGroup;
use ship_platform::artifacts::ArtifactRecord;
use ship_platform::services::sessions::types::SessionMeta;

/// Wire shape of `ship.chat.state()`.
#[derive(Serialize)]
pub struct ChatStateResponse {
    pub workspace_id: String,
    pub messages: Vec<ChatMessage>,
    pub total: usize,
}

/// Wire shape of `ship.sessions.list()`.
#[derive(Serialize)]
pub struct SessionsListResponse {
    pub items: Vec<SessionMeta>,
}

/// Wire shape of `ship.events.list()`.
#[derive(Serialize)]
pub struct EventsListResponse {
    pub groups: Vec<EntityGroup>,
    pub types: Vec<String>,
    pub actors: Vec<String>,
}

/// Wire shape of `ship.artifacts.list()`.
#[derive(Serialize)]
pub struct ArtifactsListResponse {
    pub items: Vec<ArtifactRecord>,
    pub count: usize,
    pub scope: String,
}
