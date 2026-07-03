//! ReactiveRegistry — fanout dedup keyed by (space_id, scope, query_hash).
//!
//! N subscribers sharing the same `WatchKey` share ONE resolver and ONE kernel
//! subscription. The resolver is refcounted via a `Weak<Sender>`; it tears down
//! automatically when the last subscriber drops.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use tokio::sync::broadcast;

use ship_substrate::events::EventFilter;
use ship_substrate::ports::KernelLog;

use crate::patch::PatchFrame;
use crate::resolver::{Resolver, ResolverSpec};

/// Cache key that identifies a live query slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WatchKey {
    pub space_id: String,
    pub scope: String,
    pub query_hash: u64,
}

/// Registry that deduplicates resolvers by `WatchKey`.
pub struct ReactiveRegistry {
    kernel: Arc<KernelLog>,
    slots: Mutex<HashMap<WatchKey, Weak<broadcast::Sender<PatchFrame>>>>,
}

impl ReactiveRegistry {
    pub fn new(kernel: Arc<KernelLog>) -> Self {
        Self { kernel, slots: Mutex::new(HashMap::new()) }
    }

    /// Subscribe to a live query. If an active resolver already exists for
    /// `key`, the new subscriber joins its broadcast without spawning a second
    /// resolver. Otherwise a new resolver is spawned from `spec`.
    pub fn watch(
        &self,
        key: WatchKey,
        filter: EventFilter,
        spec: ResolverSpec,
    ) -> broadcast::Receiver<PatchFrame> {
        let mut slots = self.slots.lock().unwrap();

        if let Some(weak) = slots.get(&key) {
            if let Some(sender) = weak.upgrade() {
                return sender.subscribe();
            }
        }

        let resolver = Resolver::spawn(self.kernel.clone(), filter, spec);
        let rx = resolver.subscribe();
        slots.insert(key, resolver.weak_sender());
        rx
    }
}
