//! Event registrations for the ARTIFACT domain.
//!
//! Registers the 4 artifact.* metadata events with the global event registry
//! using the hoisted `register_event!` macro from `ship_types`.
//!
//! Keystone A (2026-06-10): the ARTIFACT domain owns its events at L1.
//! These registrations moved from `ship_substrate::events::payload_registry` (now `ship_substrate::events::payload_registry` does not exist; `payload_registry` lives in the facade `ship_substrate::events::payload_registry`)
//! into this crate so that payload types and registrations co-locate with
//! their domain author.

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

register_event!("artifact.created", [EventScope::Space], Retention::Durable, Stability::Stable, "ArtifactCreated", crate::events::ArtifactCreated);
register_event!("artifact.deleted", [EventScope::Space], Retention::Durable, Stability::Stable, "ArtifactDeleted", crate::events::ArtifactDeleted);
register_event!("artifact.moved", [EventScope::Space], Retention::Durable, Stability::Stable, "ArtifactMoved", crate::events::ArtifactMoved);
register_event!("artifact.updated", [EventScope::Space], Retention::Durable, Stability::Stable, "ArtifactUpdated", crate::events::ArtifactUpdated);
