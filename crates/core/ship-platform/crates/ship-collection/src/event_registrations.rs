//! Event registrations for the COLLECTION domain.
//!
//! Registers the 5 collection.* CRUD events with the global event registry
//! using the hoisted `register_event!` macro from `ship_types`.
//!
//! Keystone A (2026-06-10): the COLLECTION domain owns its events at L1.
//! These registrations moved from `ship_substrate::events::payload_registry` (now `ship_substrate::events::payload_registry` does not exist; `payload_registry` lives in the facade `ship_substrate::events::payload_registry`)
//! into this crate so that payload types and registrations co-locate with
//! their domain author.

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

// `collection.changed` (ConfigChanged payload, watcher-family) moved to
// ship-config/src/event_registrations.rs (keystone-A Phase A bundle 3,
// 2026-06-10) with the rest of the watcher-ping family — it is a config-domain
// watcher event, not a CRUD/collection-domain event.

register_event!("collection.created", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionCreated", crate::events::CollectionCreated);
register_event!("collection.item_created", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionItemCreated", crate::events::CollectionItemCreated);
register_event!("collection.item_deleted", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionItemDeleted", crate::events::CollectionItemDeleted);
register_event!("collection.item_updated", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionItemUpdated", crate::events::CollectionItemUpdated);
register_event!("collection.schema.changed", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionSchemaChanged", crate::events::CollectionSchemaChanged);
