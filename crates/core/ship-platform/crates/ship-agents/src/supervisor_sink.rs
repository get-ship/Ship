//! The ship-side implementation of `process_supervisor`'s event sink:
//! mesh pre-registration via the kernel router + `actor.created` emission
//! through the runtime event store. Also the [`supervisor_context`]
//! composition helper every session-spawner call site uses.
//!
//! Lives in `ship-agents` — session spawners are the only consumers.

use std::sync::Arc;

use anyhow::{Result, anyhow};
use process_supervisor::{
    ActorRegistration, ProcessCreated, SupervisorContext, SupervisorEventSink,
};
use ship_substrate::db;
use ship_substrate::events::{self, KernelRouter};
use ship_substrate::project;
use ship_types::payload::ActorCreated;
use tokio::sync::Mutex;

/// The kernel-backed supervisor sink. Holds an explicit router handle —
/// no process-global lookup.
pub struct KernelSupervisorSink {
    kernel: Arc<Mutex<KernelRouter>>,
}

impl KernelSupervisorSink {
    pub fn new(kernel: Arc<Mutex<KernelRouter>>) -> Self {
        Self { kernel }
    }
}

/// Map a supervisor-owned [`ProcessCreated`] DTO into the ship event payload.
///
/// Translation lives only on the ship side of the port: process-supervisor
/// never imports ship event types. Ship-only fields get fixed defaults
/// (`restart_count: 0`, `model: None`, `system_prompt_hash: None`,
/// empty `capabilities`).
pub(crate) fn actor_created_from_process(notice: ProcessCreated) -> ActorCreated {
    ActorCreated {
        kind: notice.kind,
        environment_type: notice.environment_type,
        parent_actor_id: notice.parent_actor_id,
        restart_count: 0,
        agent_id: notice.agent_id,
        harness: notice.harness,
        model: None,
        transport: notice.transport,
        persistence: notice.persistence,
        system_prompt_hash: None,
        pid: notice.pid,
        capabilities: Vec::new(),
        space_id: notice.space_id,
        cwd: notice.cwd,
    }
}

impl SupervisorEventSink for KernelSupervisorSink {
    fn pre_register_actor(&self, registration: ActorRegistration) -> Result<()> {
        let kr = self.kernel.clone();
        let actor_id = registration.namespace.clone();
        let kernel_config = events::ActorConfig {
            namespace: registration.namespace,
            write_namespaces: registration.write_namespaces,
            read_namespaces: registration.read_namespaces,
            subscribe_namespaces: registration.subscribe_namespaces,
            see_directed_events: false,
        };
        db::block_on_anyhow(async move {
            let mut kernel = kr.lock().await;
            if !kernel.has_actor(&actor_id) {
                kernel
                    .spawn_actor(&actor_id, kernel_config)
                    .map_err(|e| anyhow!("{e}"))?;
            }
            Ok(())
        })
    }

    fn actor_created(&self, actor_id: &str, notice: ProcessCreated) -> Result<()> {
        let payload = actor_created_from_process(notice);
        db::actor_events::emit_actor_created(
            actor_id,
            &payload,
            payload.space_id.as_deref(),
            payload.parent_actor_id.as_deref(),
            Some(&self.kernel),
        )
    }
}

/// The standard spawn context: kernel-backed sink + the ship global dir.
///
/// `kernel` is the daemon's shared router — the sink pre-registers actors
/// and delivers live mailbox notifications through it.
pub fn supervisor_context(kernel: Arc<Mutex<KernelRouter>>) -> Result<SupervisorContext> {
    Ok(SupervisorContext {
        sink: Arc::new(KernelSupervisorSink::new(kernel)),
        global_dir: project::get_global_dir()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_created_maps_field_for_field_with_ship_defaults() {
        let notice = ProcessCreated {
            kind: "agent".into(),
            environment_type: "local".into(),
            parent_actor_id: Some("parent-ulid".into()),
            agent_id: Some("coder".into()),
            harness: Some("claude".into()),
            transport: Some("pty".into()),
            persistence: Some("durable".into()),
            pid: Some(4242),
            space_id: Some("space-1".into()),
            cwd: Some("/tmp/work".into()),
        };

        let payload = actor_created_from_process(notice.clone());

        // DTO fields copied through.
        assert_eq!(payload.kind, notice.kind);
        assert_eq!(payload.environment_type, notice.environment_type);
        assert_eq!(payload.parent_actor_id, notice.parent_actor_id);
        assert_eq!(payload.agent_id, notice.agent_id);
        assert_eq!(payload.harness, notice.harness);
        assert_eq!(payload.transport, notice.transport);
        assert_eq!(payload.persistence, notice.persistence);
        assert_eq!(payload.pid, notice.pid);
        assert_eq!(payload.space_id, notice.space_id);
        assert_eq!(payload.cwd, notice.cwd);

        // Ship-only defaults applied at the port boundary.
        assert_eq!(payload.restart_count, 0);
        assert_eq!(payload.model, None);
        assert_eq!(payload.system_prompt_hash, None);
        assert!(payload.capabilities.is_empty());
    }

    #[test]
    fn process_created_maps_sparse_notice() {
        let notice = ProcessCreated {
            kind: "shell".into(),
            environment_type: "local".into(),
            ..ProcessCreated::default()
        };
        let payload = actor_created_from_process(notice);
        assert_eq!(payload.kind, "shell");
        assert_eq!(payload.environment_type, "local");
        assert_eq!(payload.parent_actor_id, None);
        assert_eq!(payload.agent_id, None);
        assert_eq!(payload.harness, None);
        assert_eq!(payload.transport, None);
        assert_eq!(payload.persistence, None);
        assert_eq!(payload.pid, None);
        assert_eq!(payload.space_id, None);
        assert_eq!(payload.cwd, None);
        assert_eq!(payload.restart_count, 0);
        assert_eq!(payload.model, None);
        assert_eq!(payload.system_prompt_hash, None);
        assert!(payload.capabilities.is_empty());
    }
}
