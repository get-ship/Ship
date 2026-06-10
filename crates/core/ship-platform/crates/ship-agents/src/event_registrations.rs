//! Event registrations for the STUDIO domain.
//!
//! Registers the 4 studio.* events with the global event registry using the
//! hoisted `register_event!` macro from `ship_types`.
//!
//! Keystone A (2026-06-10): the STUDIO domain owns its events at L1.
//! These registrations moved from `ship_substrate::events::payload_registry`
//! into this crate so that payload types and registrations co-locate with
//! their domain author (`ship-agents` owns annotation/feedback/chat/reply
//! because it emits all four: annotation.rs, feedback.rs, agent_actor.rs,
//! and the studio.chat_turn emitter at shipd/sessions/api/messages.rs is
//! the single outlier but the family belongs here by majority ownership).

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

// ── studio.* ────────────────────────────────────────────────────────────────
register_event!("studio.annotation.created", [EventScope::Space], Retention::Durable, Stability::Stable, "StudioAnnotationCreated", crate::studio_events::StudioAnnotationCreated);
register_event!("studio.chat_turn", [EventScope::Space], Retention::Durable, Stability::Stable, "StudioChatTurn", crate::studio_events::StudioChatTurn);
register_event!("studio.feedback.created", [EventScope::Space], Retention::Durable, Stability::Stable, "StudioFeedbackCreated", crate::studio_events::StudioFeedbackCreated);
register_event!("studio.reply", [EventScope::Space], Retention::Durable, Stability::Stable, "StudioReply", crate::studio_events::StudioReply);
