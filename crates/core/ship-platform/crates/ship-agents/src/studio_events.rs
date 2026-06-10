// -- Studio event payloads ---------------------------------------------------
//
// These structs moved here from `ship_types::payload::studio` as part of
// the keystone-A de-bucket (2026-06-10): the STUDIO domain now owns its
// events + payload types at L1 in `ship-agents`. The rationale is that
// payload structs belong with the domain that authors and interprets them;
// moving them out of the shared L0 `ship-types` crate follows the same
// pattern as the `collection.*` family migration to `ship-collection` and
// the `session.*` family migration to `agent-def`.

use serde::{Deserialize, Serialize};

/// `studio.annotation.created` — compact projection payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct StudioAnnotationCreated {
    pub annotation_id: String,
    pub session_id: String,
    pub text: String,
    #[serde(default)]
    pub artifact_id: Option<String>,
}

/// `studio.feedback.created` — compact projection payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct StudioFeedbackCreated {
    pub feedback_id: String,
    pub rating: Option<i32>,
    pub comment: Option<String>,
}

/// `studio.chat_turn` — a message posted to a channel-capable session.
#[derive(Debug, Serialize, Deserialize)]
pub struct StudioChatTurn {
    pub session_id: String,
    pub text: String,
    #[serde(default)]
    pub agent_id: Option<String>,
}

/// `studio.reply` — a reply emitted by an agent actor.
#[derive(Debug, Serialize, Deserialize)]
pub struct StudioReply {
    pub actor_id: String,
    pub text: String,
}

/// Every studio.* event name in one slice — the domain-owned closed list
/// (keystone A), mirroring `agent-def`'s `ALL_CANONICAL_*` pattern. Used by
/// the full-linkage vocabulary guard in shipd `tests/event_vocabulary.rs`.
pub const ALL_STUDIO: &[&str] = &[
    "studio.annotation.created",
    "studio.chat_turn",
    "studio.feedback.created",
    "studio.reply",
];
