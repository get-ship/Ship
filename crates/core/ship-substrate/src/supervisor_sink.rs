//! The ship-side implementation of `process_supervisor`'s event sink:
//! mesh pre-registration via the kernel router + `actor.created` emission
//! through the runtime event store. Also the [`supervisor_context`]
//! composition helper every spawn call site uses.

use std::sync::Arc;

use anyhow::{Result, anyhow};
use process_supervisor::{
    ActorCreatedNotice, ActorRegistration, SupervisorContext, SupervisorEventSink,
};

use crate::{db, events, project};

/// The kernel-backed supervisor sink. Constructed once per call site via
/// [`supervisor_context`].
pub struct KernelSupervisorSink;

impl SupervisorEventSink for KernelSupervisorSink {
    fn pre_register_actor(&self, registration: ActorRegistration) -> Result<()> {
        // Kernel not initialized (tests, pre-boot) — skip, matching the
        // historical `if let Some(kr)` behavior.
        let Some(kr) = events::kernel_router() else {
            return Ok(());
        };
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

    fn actor_created(&self, actor_id: &str, notice: ActorCreatedNotice) -> Result<()> {
        let payload = events::types::ActorCreated {
            kind: notice.kind,
            environment_type: notice.environment_type,
            parent_actor_id: notice.parent_actor_id.clone(),
            agent_id: notice.agent_id,
            harness: notice.harness,
            transport: notice.transport,
            persistence: notice.persistence,
            pid: notice.pid,
            space_id: notice.space_id.clone(),
            cwd: notice.cwd,
            ..Default::default()
        };
        db::actor_events::emit_actor_created(
            actor_id,
            &payload,
            notice.space_id.as_deref(),
            notice.parent_actor_id.as_deref(),
        )
    }
}

/// The standard spawn context: kernel-backed sink + the ship global dir.
pub fn supervisor_context() -> Result<SupervisorContext> {
    Ok(SupervisorContext {
        sink: Arc::new(KernelSupervisorSink),
        global_dir: project::get_global_dir()?,
    })
}
