//! KernelRouter — per-actor mailboxes with sink-based persistence.
//!
//! Pure event routing with no filesystem or database dependencies.
//! Persistence is delegated to the caller-provided `EventSink`.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc;

use ship_types::envelope::EventEnvelope;
pub use ship_types::ActorConfig;

use crate::mailbox::Mailbox;
use crate::sink::EventSink;
use crate::validator::EventValidator;
use crate::KernelError;

pub const MAILBOX_CAPACITY: usize = 256;

/// Kernel-managed event router with per-actor isolation.
pub struct KernelRouter {
    sink: Arc<dyn EventSink>,
    mailboxes: HashMap<String, mpsc::Sender<EventEnvelope>>,
    actor_configs: HashMap<String, ActorConfig>,
    subscriptions: HashMap<String, Vec<String>>,
    validators: Vec<Box<dyn EventValidator>>,
}

impl KernelRouter {
    /// Create a new router backed by the given event sink.
    pub fn new(sink: Arc<dyn EventSink>) -> Self {
        Self {
            sink,
            mailboxes: HashMap::new(),
            actor_configs: HashMap::new(),
            subscriptions: HashMap::new(),
            validators: Vec::new(),
        }
    }

    /// Add a validator to the pipeline. Validators run before persist + delivery.
    pub fn with_validator(mut self, v: Box<dyn EventValidator>) -> Self {
        self.validators.push(v);
        self
    }

    /// Check whether an actor with the given ID is registered.
    pub fn has_actor(&self, actor_id: &str) -> bool {
        self.mailboxes.contains_key(actor_id)
    }

    /// Register an actor and return its mailbox.
    pub fn spawn_actor(
        &mut self,
        actor_id: &str,
        config: ActorConfig,
    ) -> Result<Mailbox, KernelError> {
        if self.mailboxes.contains_key(actor_id) {
            return Err(KernelError::ActorAlreadyExists {
                actor_id: actor_id.to_string(),
            });
        }

        let (tx, rx) = mpsc::channel(MAILBOX_CAPACITY);
        self.mailboxes.insert(actor_id.to_string(), tx);
        self.actor_configs
            .insert(actor_id.to_string(), config.clone());

        for ns in &config.subscribe_namespaces {
            let ids = self.subscriptions.entry(ns.clone()).or_default();
            if !ids.contains(&actor_id.to_string()) {
                ids.push(actor_id.to_string());
            }
        }

        Ok(Mailbox::new(rx))
    }

    /// Remove an actor from the router.
    pub fn stop_actor(&mut self, actor_id: &str) -> Result<(), KernelError> {
        if self.mailboxes.remove(actor_id).is_none() {
            return Err(KernelError::ActorNotFound {
                actor_id: actor_id.to_string(),
            });
        }
        self.actor_configs.remove(actor_id);

        for actor_ids in self.subscriptions.values_mut() {
            actor_ids.retain(|id| id != actor_id);
        }
        Ok(())
    }

    /// Number of registered actors.
    pub fn actor_count(&self) -> usize {
        self.mailboxes.len()
    }

    /// Snapshot of all active actor IDs and their configs.
    pub fn list_actors(&self) -> Vec<(String, ActorConfig)> {
        self.actor_configs
            .iter()
            .map(|(id, cfg)| (id.clone(), cfg.clone()))
            .collect()
    }

    /// Look up the config for a single actor. Returns `None` if not registered.
    pub fn get_actor_config(&self, actor_id: &str) -> Option<&ActorConfig> {
        self.actor_configs.get(actor_id)
    }

    /// Deliver an event directly to a named actor's mailbox. No persistence, no validation.
    ///
    /// Used by runtime-specific delivery paths (e.g. scope-filtered delivery in
    /// the `unstable` feature) that need per-actor control over which events are
    /// forwarded. Returns `true` if the actor exists and the send was attempted,
    /// `false` if the actor is not registered. Silently drops if the mailbox is
    /// full or closed (callers that need overflow handling should check the return
    /// and emit their own diagnostic events).
    pub fn send_to_actor(&self, actor_id: &str, event: EventEnvelope) -> bool {
        if let Some(tx) = self.mailboxes.get(actor_id) {
            let _ = tx.try_send(event);
            true
        } else {
            false
        }
    }

    /// Sender handle for a registered actor's mailbox, or `None` if not found.
    ///
    /// Used by runtime-specific delivery paths that need to check the channel
    /// state (e.g. to detect a full mailbox) before deciding how to proceed.
    pub fn mailbox_sender(&self, actor_id: &str) -> Option<&mpsc::Sender<EventEnvelope>> {
        self.mailboxes.get(actor_id)
    }

    /// Snapshot of all namespace → actor-ID subscription mappings.
    ///
    /// Used by runtime-specific delivery paths (e.g. scope-filtered delivery)
    /// that need to inspect subscription state without going through `route`.
    pub fn subscriptions(&self) -> &HashMap<String, Vec<String>> {
        &self.subscriptions
    }

    /// Validate, persist, and deliver an event.
    pub async fn route(&self, event: EventEnvelope) -> anyhow::Result<()> {
        // Run validators
        for v in &self.validators {
            v.validate(&event)?;
        }

        // Persist via sink
        self.sink.persist(&event)?;

        // Deliver without re-running validators or persisting.
        self.deliver(&event);
        Ok(())
    }

    /// Deliver an event to subscribers without persistence or validation.
    ///
    /// Used by callers that have already persisted the event and just need
    /// to push it to in-memory mailboxes (e.g. the deprecated `route()` path
    /// in the runtime wrapper that skips persistence).
    pub fn deliver(&self, event: &EventEnvelope) {
        // Direct message — deliver only to target
        if let Some(ref target) = event.target_actor_id {
            if let Some(tx) = self.mailboxes.get(target) {
                let _ = tx.try_send(event.clone());
            }
            return;
        }

        // Broadcast — deliver to matching subscribers
        for (ns, actor_ids) in &self.subscriptions {
            if event.event_type.starts_with(ns.as_str()) {
                for actor_id in actor_ids {
                    if let Some(tx) = self.mailboxes.get(actor_id) {
                        let _ = tx.try_send(event.clone());
                    }
                }
            }
        }
    }
}
