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
            see_directed_events: false,
        };

        // Register with inner for subscription state and actor config lookup.
        // inner.spawn_actor() creates its own channel — we discard that Mailbox
        // and use our own channel (tx/rx) so cursor-based replay can inject
        // events before returning. deliver_with_scope() uses unstable_senders
        // for actual delivery to these actors, not inner's mailboxes.
        let _ = self.inner.spawn_actor(label, config.clone())
            .map_err(|e| anyhow!("{e}"))?;

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
            let log = self.event_log.lock().unwrap_or_else(|e| e.into_inner());
            let mut replay: Vec<_> = log
                .since(pos)
                .filter(|ev| self.should_deliver(ev, label))
                .cloned()
                .collect();
            replay.sort_by(|a, b| a.id.cmp(&b.id));
            for ev in &replay {
                let _ = tx.try_send(ev.clone());
            }
        }

        // Stash the sender so deliver_with_scope can reach this actor's mailbox.
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
                    // Space-scoped delivery: the actor receives the event only
                    // when the event's SPACE matches the space this actor is
                    // bound to. `event.space_id()` reads the `(scope, scope_key)`
                    // tuple — the scope_key when the event is Space-scoped, else
                    // None — carrying the daemon's path-hash space id (never a
                    // branch slug — see spec `116-event-partition.md`). A daemon
                    // serves exactly one space, so this equality is effectively
                    // "belongs to this daemon's space". The opaque match key is
                    // the actor's bound scope (legacy field name `workspace_id`;
                    // rename deferred to a later slice).
                    if event.space_id() == meta.workspace_id.as_deref() {
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
    /// Returns `Err` if any permissions-based actor's mailbox was full. Actors
    /// spawned via `spawn_actor()` (legacy path) silently drop on full — only
    /// actors with `actor_meta` (spawned via `spawn_actor_with_permissions*`)
    /// trigger error propagation and `actor.mailbox_full` events.
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
                        // Only propagate full errors for permissions-based actors.
                        if self.actor_meta.contains_key(target) {
                            full_actors.push(target.clone());
                        } else {
                            tracing::warn!(
                                actor_id = target,
                                event_type = %event.event_type,
                                "kernel-router: mailbox full, event dropped"
                            );
                        }
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
                                // Only propagate full errors for permissions-based actors.
                                if self.actor_meta.contains_key(actor_id) {
                                    full_actors.push(actor_id.clone());
                                } else {
                                    tracing::warn!(
                                        actor_id,
                                        event_type = %event.event_type,
                                        "kernel-router: mailbox full, event dropped"
                                    );
                                }
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
