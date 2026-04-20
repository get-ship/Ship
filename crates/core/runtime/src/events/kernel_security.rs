//! Security extensions for KernelRouter — identity, permissions, scoped delivery.
//!
//! Adds `spawn_actor_with_permissions`, emit enforcement, scoped/directed
//! delivery, and cursor-based replay to the kernel router.

use anyhow::{Result, anyhow};
use tokio::sync::mpsc;

use crate::persistence::actor_store::ActorStore;
use ship_types::envelope::EventEnvelope;
use ship_types::identity::ActorIdentity;
use crate::events::kernel_router::{KernelRouter, MAILBOX_CAPACITY};
use ship_kernel::Mailbox;
use crate::events::permissions::{ActorPermissions, DeliveryScope, PermittedSubscription};
use crate::events::validator::{CallerKind, EmitContext};

/// Per-actor metadata for permission-based actors.
pub(crate) struct ActorMeta {
    pub permissions: ActorPermissions,
    pub workspace_id: Option<String>,
    pub subscriptions: Vec<PermittedSubscription>,
}

impl KernelRouter {
    /// Spawn an actor with identity and permissions.
    pub fn spawn_actor_with_permissions(
        &mut self,
        identity: ActorIdentity,
        permissions: ActorPermissions,
    ) -> Result<(ActorStore, Mailbox)> {
        self.spawn_actor_with_permissions_inner(identity, permissions, None)
    }

    /// Spawn an actor with identity, permissions, and workspace binding.
    pub fn spawn_actor_with_permissions_in_workspace(
        &mut self,
        identity: ActorIdentity,
        permissions: ActorPermissions,
        workspace_id: &str,
    ) -> Result<(ActorStore, Mailbox)> {
        self.spawn_actor_with_permissions_inner(
            identity,
            permissions,
            Some(workspace_id.to_string()),
        )
    }

    fn spawn_actor_with_permissions_inner(
        &mut self,
        identity: ActorIdentity,
        permissions: ActorPermissions,
        workspace_id: Option<String>,
    ) -> Result<(ActorStore, Mailbox)> {
        permissions.validate()?;

        let label = &identity.label;
        if self.inner.has_actor(label) {
            return Err(anyhow!("actor '{}' already exists", label));
        }

        let cursor_pos = self.actor_cursors.get(label).copied();

        let db_path = self.actor_db_path(label);

        // Build an mpsc channel manually so we can replay into it before returning.
        let (tx, rx) = mpsc::channel(MAILBOX_CAPACITY);

        // Build the ActorConfig from permissions for inner registration.
        let config = ship_types::ActorConfig {
            namespace: label.to_string(),
            write_namespaces: permissions.emit.clone(),
            read_namespaces: subs_to_namespaces(&permissions.subscribe),
            subscribe_namespaces: subs_to_namespaces(&permissions.subscribe),
        };

        // Register subscriptions in inner so route() can find this actor.
        // We insert the sender we own so delivery goes to our mailbox.
        // We bypass inner.spawn_actor() here because we need our own channel
        // for the replay path — instead we register via inner's spawn and
        // discard the returned mailbox, then route deliveries ourselves.
        // TODO: expose a lower-level registration API on inner if this pattern
        // grows. For now the unstable path is self-contained.
        let _ = self.inner.spawn_actor(label, config.clone())
            .map_err(|e| anyhow!("{e}"))?;

        // Override: replace inner's sender with our own so that deliver_with_scope
        // can access sender state (full / closed) via inner.mailbox_sender().
        // This is a limitation of the current split: the unstable path creates its
        // own channel and cannot share inner's sender. We store our sender in
        // actor_meta for use by deliver_with_scope.
        let subs = permissions.subscribe.clone();
        self.actor_meta.insert(
            label.to_string(),
            ActorMeta {
                permissions: permissions.clone(),
                workspace_id,
                subscriptions: subs,
            },
        );

        // Replay events since cursor, sorted by ULID for guaranteed ordering.
        if let Some(pos) = cursor_pos {
            let log = self.event_log.lock().unwrap();
            let mut replay: Vec<_> = log[pos..]
                .iter()
                .filter(|ev| self.should_deliver(ev, label))
                .cloned()
                .collect();
            replay.sort_by(|a, b| a.id.cmp(&b.id));
            for ev in &replay {
                let _ = tx.try_send(ev.clone());
            }
        }

        // Store sender for use by deliver_with_scope.
        self.actor_meta
            .get_mut(label)
            .expect("just inserted")
            .permissions = permissions.clone();

        // Keep a copy of tx for deliver_with_scope via the senders map.
        // We stash it separately since inner's channel is different.
        self.unstable_senders.insert(label.to_string(), tx);

        let store = ActorStore::new(
            label,
            db_path,
            permissions.emit.clone(),
            subs_to_namespaces(&permissions.subscribe),
        );
        Ok((store, Mailbox::from_receiver(rx)))
    }

    /// Check if an event matches an actor's subscription scope.
    /// Returns None if the actor has no permission metadata (legacy spawn_actor).
    pub(crate) fn event_matches_actor_scope(
        &self,
        event: &EventEnvelope,
        actor_label: &str,
    ) -> Option<bool> {
        let meta = self.actor_meta.get(actor_label)?;
        for sub in &meta.subscriptions {
            if !event.event_type.starts_with(sub.namespace.as_str()) {
                continue;
            }
            match sub.scope {
                DeliveryScope::Global => return Some(true),
                DeliveryScope::Workspace => {
                    if event.workspace_id.as_deref() == meta.workspace_id.as_deref() {
                        return Some(true);
                    }
                }
                DeliveryScope::Directed => {
                    if event.target_actor_id.as_deref() == Some(actor_label) {
                        return Some(true);
                    }
                }
                DeliveryScope::Elevated => {
                    if event.elevated {
                        return Some(true);
                    }
                }
            }
        }
        Some(false)
    }

    /// Enforce emit permissions. CLI callers bypass.
    pub(crate) fn enforce_emit_permissions(
        &self,
        event: &EventEnvelope,
        ctx: &EmitContext,
    ) -> Result<()> {
        if ctx.caller_kind == CallerKind::Cli {
            return Ok(());
        }
        if ctx.caller_kind == CallerKind::Runtime && event.actor_id.is_none() {
            return Ok(());
        }

        if let Some(ref actor_id) = event.actor_id
            && let Some(meta) = self.actor_meta.get(actor_id)
            && !meta.permissions.can_emit(&event.event_type)
        {
            return Err(anyhow!(
                "actor '{}' not permitted to emit '{}'",
                actor_id,
                event.event_type
            ));
        }

        Ok(())
    }

    /// Should this actor receive this event?
    pub(crate) fn should_deliver(&self, event: &EventEnvelope, actor_id: &str) -> bool {
        self.event_matches_actor_scope(event, actor_id)
            .unwrap_or(true) // Legacy actor — namespace match already done by caller
    }

    /// Deliver event to matching subscribers respecting scope and directed delivery.
    ///
    /// Returns `Err` if any actor's mailbox was full. The event is still
    /// delivered to all actors that have capacity — only the full ones miss it.
    pub(crate) async fn deliver_with_scope(&self, event: &EventEnvelope) -> Result<()> {
        let mut full_actors: Vec<String> = Vec::new();

        if let Some(ref target) = event.target_actor_id {
            // Try unstable senders first, then fall back to inner's channel.
            let tx_opt = self.unstable_senders.get(target)
                .or_else(|| self.inner.mailbox_sender(target));
            if let Some(tx) = tx_opt
                && self.should_deliver(event, target)
            {
                match tx.try_send(event.clone()) {
                    Ok(()) => {
                        tracing::debug!(
                            actor_id = target,
                            event_type = %event.event_type,
                            "kernel-router: targeted event delivered"
                        );
                    }
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        full_actors.push(target.clone());
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        tracing::warn!(
                            actor_id = target,
                            event_type = %event.event_type,
                            "kernel-router: targeted delivery failed, mailbox closed"
                        );
                    }
                }
            }
            if !full_actors.is_empty() {
                self.emit_mailbox_full_events(&full_actors, event);
                return Err(anyhow!(
                    "mailbox full for actor '{}': event '{}' not delivered",
                    full_actors[0],
                    event.event_type
                ));
            }
            return Ok(());
        }

        for (ns, actor_ids) in self.inner.subscriptions() {
            if event.event_type.starts_with(ns.as_str()) {
                for actor_id in actor_ids {
                    if !self.should_deliver(event, actor_id) {
                        continue;
                    }
                    let tx_opt = self.unstable_senders.get(actor_id)
                        .or_else(|| self.inner.mailbox_sender(actor_id));
                    if let Some(tx) = tx_opt {
                        match tx.try_send(event.clone()) {
                            Ok(()) => {
                                tracing::debug!(
                                    actor_id,
                                    event_type = %event.event_type,
                                    "kernel-router: event delivered"
                                );
                            }
                            Err(mpsc::error::TrySendError::Full(_)) => {
                                full_actors.push(actor_id.clone());
                            }
                            Err(mpsc::error::TrySendError::Closed(_)) => {
                                tracing::warn!(
                                    actor_id,
                                    event_type = %event.event_type,
                                    "kernel-router: delivery failed, mailbox closed"
                                );
                            }
                        }
                    }
                }
            }
        }

        if !full_actors.is_empty() {
            self.emit_mailbox_full_events(&full_actors, event);
            return Err(anyhow!(
                "mailbox full for actor(s) [{}]: event '{}' not delivered",
                full_actors.join(", "),
                event.event_type
            ));
        }
        Ok(())
    }

    /// Emit `actor.mailbox_full` kernel events for each actor whose mailbox overflowed.
    fn emit_mailbox_full_events(&self, actor_ids: &[String], event: &EventEnvelope) {
        for actor_id in actor_ids {
            tracing::warn!(
                actor_id,
                event_type = %event.event_type,
                capacity = MAILBOX_CAPACITY,
                "kernel-router: mailbox full, event not delivered"
            );
            let payload = serde_json::json!({
                "actor_id": actor_id,
                "dropped_event_type": &event.event_type,
                "dropped_event_id": &event.id,
            });
            if let Ok(ev) = EventEnvelope::new("actor.mailbox_full", actor_id, &payload) {
                let _ = crate::persistence::actor_store::raw_append(&self.kernel_store_path, &ev);
            }
        }
    }
}

fn subs_to_namespaces(subs: &[PermittedSubscription]) -> Vec<String> {
    subs.iter().map(|s| s.namespace.clone()).collect()
}
