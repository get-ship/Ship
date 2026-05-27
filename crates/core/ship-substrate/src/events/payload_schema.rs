//! Payload schema validation for known event types.
//!
//! Maps event type strings to their expected payload struct. If a known event
//! type has a malformed payload, it is rejected before persistence. Unknown
//! event types (custom skill events) pass through.

use ship_types::envelope::EventEnvelope;
use crate::events::validator::{EmitContext, EventValidator, ValidationError};

/// Validates payloads for known event types at emit time. Unknown event types
/// pass through — custom skill events are the caller's responsibility.
pub struct PayloadSchemaValidator;

impl EventValidator for PayloadSchemaValidator {
    fn validate(
        &self,
        event: &EventEnvelope,
        _ctx: &EmitContext,
    ) -> Result<(), ValidationError> {
        if let Some(validate_fn) = schema_for_event_type(&event.event_type) {
            validate_fn(&event.payload_json).map_err(|e| {
                ValidationError::SchemaViolation {
                    event_type: event.event_type.clone(),
                    error: e.to_string(),
                }
            })?;
        }
        Ok(())
    }
}

/// Validation function that checks a JSON payload string against an expected schema.
type PayloadValidator = fn(&str) -> Result<(), serde_json::Error>;

/// Returns a payload validation function for known event types.
/// The function attempts to deserialize the JSON payload into the expected
/// struct. Returns `None` for unknown/custom event types.
fn schema_for_event_type(
    event_type: &str,
) -> Option<PayloadValidator> {
    use crate::events::types::*;

    fn check<T: serde::de::DeserializeOwned>(json: &str) -> Result<(), serde_json::Error> {
        serde_json::from_str::<T>(json).map(|_| ())
    }

    match event_type {
        // workspace
        "workspace.created" => Some(check::<WorkspaceCreated>),
        "workspace.activated" => Some(check::<WorkspaceActivated>),
        "workspace.archived" => Some(check::<WorkspaceArchived>),
        // session
        "session.started" => Some(check::<SessionStarted>),
        "session.progress" => Some(check::<SessionProgress>),
        "session.ended" => Some(check::<SessionEnded>),
        "session.drain_started" => Some(check::<SessionDrainStarted>),
        "session.drain_completed" => Some(check::<SessionDrainCompleted>),
        "session.drain_aborted" => Some(check::<SessionDrainAborted>),
        "session.tool_count_incremented" => Some(check::<SessionToolCountIncremented>),
        "session.heartbeat" => Some(check::<SessionHeartbeat>),
        // actor
        "actor.created" => Some(check::<ActorCreated>),
        "actor.woke" => Some(check::<ActorWoke>),
        "actor.slept" => Some(check::<ActorSlept>),
        "actor.stopped" => Some(check::<ActorStopped>),
        "actor.crashed" => Some(check::<ActorCrashed>),
        // job — canonical 18-field payloads live in `crate::events::job`
        "pipeline.run.started" => Some(check::<crate::events::pipeline::PipelineRunStartedPayload>),
        "pipeline.run.completed" => Some(check::<crate::events::pipeline::PipelineRunCompletedPayload>),
        "pipeline.run.failed" => Some(check::<crate::events::pipeline::PipelineRunFailedPayload>),
        "pipeline.run.dispatched" => Some(check::<crate::events::pipeline::PipelineRunDispatchedPayload>),
        ship_types::event_types::PIPELINE_STEP_COMPLETED => Some(check::<crate::events::pipeline::PipelineStepCompletedPayload>),
        ship_types::event_types::PIPELINE_STEP_FAILED => Some(check::<crate::events::pipeline::PipelineStepFailedPayload>),
        ship_types::event_types::PIPELINE_RUN_CANCELLED => Some(check::<crate::events::pipeline::PipelineRunCancelledPayload>),
        ship_types::event_types::PIPELINE_RUN_MERGED => Some(check::<crate::events::pipeline::PipelineRunMergedPayload>),
        ship_types::event_types::PIPELINE_STEP_SPAWN_FAILED => Some(check::<crate::events::pipeline::PipelineStepSpawnFailedPayload>),
        // config + declarative-tier domain events (Phase 4)
        "config.changed" => Some(check::<ConfigChanged>),
        "config.file_changed" => Some(check::<ConfigFileChanged>),
        "space.setting.changed" => Some(check::<SpaceSettingChanged>),
        "agent.changed" => Some(check::<ConfigChanged>),
        "skill.changed" => Some(check::<ConfigChanged>),
        "view.changed" => Some(check::<ConfigChanged>),
        "pipeline.changed" => Some(check::<ConfigChanged>),
        "rule.changed" => Some(check::<ConfigChanged>),
        "project.changed" => Some(check::<ConfigChanged>),
        // project
        "project.log" => Some(check::<ProjectLog>),
        // skill
        "skill.started" => Some(check::<SkillStarted>),
        "skill.completed" => Some(check::<SkillCompleted>),
        "skill.failed" => Some(check::<SkillFailed>),
        ship_types::event_types::SKILL_ATTACHED_TO_SESSION => Some(check::<SkillAttachedToSession>),
        // process
        "process.spawned" => Some(check::<crate::events::process::ProcessSpawnedPayload>),
        "process.exited" => Some(check::<crate::events::process::ProcessExitedPayload>),
        "process.signal_sent" => Some(check::<crate::events::process::ProcessSignalSentPayload>),
        "process.orphaned" => Some(check::<crate::events::process::ProcessOrphanedPayload>),
        // executor
        "executor.session_destroyed" => Some(check::<crate::events::executor::ExecutorSessionDestroyedPayload>),
        "executor.window_destroyed" => Some(check::<crate::events::executor::ExecutorWindowDestroyedPayload>),
        // agent session (the dispatch primitive)
        "agent.session.started" => Some(check::<crate::events::agent_session::AgentSessionStartedPayload>),
        "agent.session.dispatched" => Some(check::<crate::events::agent_session::AgentSessionDispatchedPayload>),
        "agent.session.completed" => Some(check::<crate::events::agent_session::AgentSessionCompletedPayload>),
        "agent.session.failed" => Some(check::<crate::events::agent_session::AgentSessionFailedPayload>),
        "agent.session.cancelled" => Some(check::<crate::events::agent_session::AgentSessionCancelledPayload>),
        "agent.session.turn_ended" => Some(check::<crate::events::agent_session::AgentSessionTurnEndedPayload>),
        // supervisor
        "supervisor.shutting_down" => Some(check::<SupervisorShuttingDown>),
        "supervisor.sweep_started" => Some(check::<crate::events::supervisor::SweepStartedPayload>),
        "supervisor.sweep_completed" => Some(check::<crate::events::supervisor::SweepCompletedPayload>),
        "supervisor.drift_detected" => Some(check::<crate::events::supervisor::DriftDetectedPayload>),
        "supervisor.coherence_restored" => Some(check::<crate::events::supervisor::CoherenceRestoredPayload>),
        // daemon
        "daemon.startup_completed" => Some(check::<crate::events::daemon::StartupCompletedPayload>),
        "daemon.shutdown_completed" => Some(check::<crate::events::daemon::ShutdownCompletedPayload>),
        "daemon.crashed" => Some(check::<crate::events::daemon::CrashedPayload>),
        "daemon.port_bind_failed" => Some(check::<crate::events::daemon::PortBindFailedPayload>),
        "daemon.pidfile_acquired" => Some(check::<crate::events::daemon::PidfileAcquiredPayload>),
        "daemon.pidfile_stale_taken" => Some(check::<crate::events::daemon::PidfileStaleTakenPayload>),
        // artifact
        "artifact.created" => Some(check::<ArtifactCreated>),
        "artifact.updated" => Some(check::<ArtifactUpdated>),
        "artifact.moved" => Some(check::<ArtifactMoved>),
        "artifact.deleted" => Some(check::<ArtifactDeleted>),
        // studio / feedback (agent-designed feedback payload from Studio views)
        "studio.feedback.created" => Some(check::<crate::events::feedback::FeedbackEvent>),
        // studio / annotation (Phase 1 pin for the Phase 2 overlay wire format)
        "studio.annotation.created" => Some(check::<crate::events::annotation::AnnotationEvent>),
        // compile
        "compile.completed" => Some(check::<CompileCompleted>),
        // project registry
        "project.registered" => Some(check::<ProjectRegistered>),
        "project.seen" => Some(check::<ProjectSeen>),
        "project.archived" => Some(check::<ProjectArchived>),
        "project.switched" => Some(check::<ProjectSwitched>),
        // auth
        "auth.session.created" => Some(check::<AuthSessionCreated>),
        "auth.session.revoked" => Some(check::<AuthSessionRevoked>),
        "auth.session.expired" => Some(check::<AuthSessionExpired>),
        "daemon.owner.bound" => Some(check::<DaemonOwnerBound>),
        "daemon.owner.unbound" => Some(check::<DaemonOwnerUnbound>),
        // account lifecycle
        "account.login_started" => Some(check::<AccountLoginStarted>),
        "account.login_completed" => Some(check::<AccountLoginCompleted>),
        "account.logout" => Some(check::<AccountLogout>),
        "account.key_refreshed" => Some(check::<AccountKeyRefreshed>),
        // skill / rule / permission CRUD (event-coverage/01)
        "skill.created" => Some(check::<SkillCreated>),
        "skill.updated" => Some(check::<SkillUpdated>),
        "skill.deleted" => Some(check::<SkillDeleted>),
        "rule.created" => Some(check::<RuleCreated>),
        "rule.updated" => Some(check::<RuleUpdated>),
        "rule.deleted" => Some(check::<RuleDeleted>),
        "permissions.updated" => Some(check::<PermissionsUpdated>),
        // capability provisioning
        "agent.grants.provisioned" => Some(check::<AgentGrantsProvisioned>),
        // session lifecycle additions (event-coverage/02)
        "session.status_changed" => Some(check::<SessionStatusChanged>),
        "session.turn_failed" => Some(check::<SessionTurnFailed>),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::validator::CallerKind;

    fn caller_ctx(kind: CallerKind) -> EmitContext {
        EmitContext {
            caller_kind: kind,
            skill_id: None,
            space_id: None,
            session_id: None,
        }
    }

    #[test]
    fn valid_payload_passes() {
        let v = PayloadSchemaValidator;
        let event = EventEnvelope::new(
            "workspace.created",
            "ws-1",
            &serde_json::json!({ "workspace_id": "ws-1" }),
        )
        .unwrap();
        let ctx = caller_ctx(CallerKind::Runtime);
        assert!(v.validate(&event, &ctx).is_ok());
    }

    #[test]
    fn malformed_payload_rejected() {
        let v = PayloadSchemaValidator;
        let event = EventEnvelope::new(
            "workspace.created",
            "ws-1",
            &serde_json::json!({ "garbage": true }),
        )
        .unwrap();
        let ctx = caller_ctx(CallerKind::Runtime);
        let err = v.validate(&event, &ctx).unwrap_err();
        assert!(matches!(err, ValidationError::SchemaViolation { .. }));
    }

    #[test]
    fn unknown_event_type_passes_schema() {
        let v = PayloadSchemaValidator;
        let event = EventEnvelope::new(
            "my-analytics.page_view",
            "page-1",
            &serde_json::json!({ "anything": "goes" }),
        )
        .unwrap();
        let ctx = caller_ctx(CallerKind::Mcp);
        assert!(v.validate(&event, &ctx).is_ok());
    }

    #[test]
    fn all_known_event_types_have_schemas() {
        use crate::events::types::event_types::ALL;
        // job.update has no dedicated payload struct — it's a generic update.
        // process.killed is declared but not yet emitted; deferred to a
        // follow-on slice where we wire the plumbing through the executor
        // signal-send sites. studio.chat_turn carries an ad-hoc {actor_id,
        // text} payload; not yet a typed struct.
        let exceptions = &[
            "pipeline.run.updated",
            "session.attached",
            "process.killed",
            "studio.chat_turn",
            "studio.reply",
        ];
        for &et in ALL {
            if exceptions.contains(&et) {
                continue;
            }
            assert!(
                schema_for_event_type(et).is_some(),
                "missing schema mapping for event type '{et}'"
            );
        }
    }
}
