pub const WORKSPACE_CREATED: &str = "workspace.created";
pub const WORKSPACE_DELETED: &str = "workspace.deleted";
pub const WORKSPACE_STATUS_CHANGED: &str = "workspace.status_changed";
pub const WORKSPACE_ACTIVATED: &str = "workspace.activated";
pub const WORKSPACE_COMPILED: &str = "workspace.compiled";
pub const WORKSPACE_COMPILE_FAILED: &str = "workspace.compile_failed";
pub const WORKSPACE_ARCHIVED: &str = "workspace.archived";
pub const WORKSPACE_AGENT_CHANGED: &str = "workspace.agent_changed";
pub const WORKSPACE_RECONCILED: &str = "workspace.reconciled";
pub const WORKSPACE_TMUX_ASSIGNED: &str = "workspace.tmux_assigned";
pub const WORKSPACE_STARTED: &str = "workspace.started";
pub const WORKSPACE_WORKTREE_ADDED: &str = "workspace.worktree_added";
pub const WORKSPACE_WORKTREE_REMOVED: &str = "workspace.worktree_removed";
pub const WORKSPACE_BRANCH_SWITCHED: &str = "workspace.branch_switched";
pub const WORKSPACE_CONFIG_CHANGED: &str = "workspace.config_changed";
pub const SESSION_STARTED: &str = "session.started";
pub const SESSION_PROGRESS: &str = "session.progress";
pub const SESSION_ENDED: &str = "session.ended";
pub const SESSION_RECORDED: &str = "session.recorded";
pub const SESSION_DRAIN_STARTED: &str = "session.drain_started";
pub const SESSION_DRAIN_COMPLETED: &str = "session.drain_completed";
pub const SESSION_DRAIN_ABORTED: &str = "session.drain_aborted";
pub const SESSION_TOOL_COUNT_INCREMENTED: &str = "session.tool_count_incremented";
pub const SESSION_ATTACHED: &str = "session.attached";
pub const SESSION_HEARTBEAT: &str = "session.heartbeat";
pub const ACTOR_CREATED: &str = "actor.created";
pub const ACTOR_WOKE: &str = "actor.woke";
pub const ACTOR_SLEPT: &str = "actor.slept";
pub const ACTOR_STOPPED: &str = "actor.stopped";
pub const ACTOR_CRASHED: &str = "actor.crashed";
pub const GATE_PASSED: &str = "gate.passed";
pub const GATE_FAILED: &str = "gate.failed";
pub const JOB_CREATED: &str = "job.created";
pub const JOB_CLAIMED: &str = "job.claimed";
pub const JOB_COMPLETED: &str = "job.completed";
pub const JOB_FAILED: &str = "job.failed";
pub const JOB_DISPATCHED: &str = "job.dispatched";
pub const JOB_UPDATE: &str = "job.update";
pub const CONFIG_CHANGED: &str = "config.changed";
pub const PROJECT_LOG: &str = "project.log";
pub const SKILL_STARTED: &str = "skill.started";
pub const SKILL_COMPLETED: &str = "skill.completed";
pub const SKILL_FAILED: &str = "skill.failed";
pub const SUPERVISOR_SHUTTING_DOWN: &str = "supervisor.shutting_down";
pub const SUPERVISOR_SWEEP_STARTED: &str = "supervisor.sweep_started";
pub const SUPERVISOR_SWEEP_COMPLETED: &str = "supervisor.sweep_completed";
pub const SUPERVISOR_DRIFT_DETECTED: &str = "supervisor.drift_detected";
pub const SUPERVISOR_COHERENCE_RESTORED: &str = "supervisor.coherence_restored";
pub const PROCESS_SPAWNED: &str = "process.spawned";
pub const PROCESS_EXITED: &str = "process.exited";
pub const PROCESS_SIGNAL_SENT: &str = "process.signal_sent";
pub const PROCESS_ORPHANED: &str = "process.orphaned";
pub const PROCESS_KILLED: &str = "process.killed";
pub const EXECUTOR_SESSION_CREATED: &str = "executor.session_created";
pub const EXECUTOR_SESSION_DESTROYED: &str = "executor.session_destroyed";
pub const EXECUTOR_WINDOW_CREATED: &str = "executor.window_created";
pub const EXECUTOR_WINDOW_DESTROYED: &str = "executor.window_destroyed";
pub const DAEMON_STARTUP_COMPLETED: &str = "daemon.startup_completed";
pub const DAEMON_SHUTDOWN_COMPLETED: &str = "daemon.shutdown_completed";
pub const DAEMON_CRASHED: &str = "daemon.crashed";
pub const DAEMON_PORT_BIND_FAILED: &str = "daemon.port_bind_failed";
pub const DAEMON_PIDFILE_ACQUIRED: &str = "daemon.pidfile_acquired";
pub const DAEMON_PIDFILE_STALE_TAKEN: &str = "daemon.pidfile_stale_taken";
pub const JOB_STEP_COMPLETED: &str = "job.step_completed";
pub const JOB_STEP_FAILED: &str = "job.step_failed";
pub const JOB_CANCELLED: &str = "job.cancelled";
pub const JOB_MERGED: &str = "job.merged";
pub const JOB_SPAWN_FAILED: &str = "job.spawn_failed";

pub const ALL: &[&str] = &[
    WORKSPACE_CREATED,
    WORKSPACE_DELETED,
    WORKSPACE_STATUS_CHANGED,
    WORKSPACE_ACTIVATED,
    WORKSPACE_COMPILED,
    WORKSPACE_COMPILE_FAILED,
    WORKSPACE_ARCHIVED,
    WORKSPACE_AGENT_CHANGED,
    WORKSPACE_RECONCILED,
    WORKSPACE_TMUX_ASSIGNED,
    WORKSPACE_STARTED,
    WORKSPACE_WORKTREE_ADDED,
    WORKSPACE_WORKTREE_REMOVED,
    WORKSPACE_BRANCH_SWITCHED,
    WORKSPACE_CONFIG_CHANGED,
    SESSION_STARTED,
    SESSION_PROGRESS,
    SESSION_ENDED,
    SESSION_RECORDED,
    SESSION_DRAIN_STARTED,
    SESSION_DRAIN_COMPLETED,
    SESSION_DRAIN_ABORTED,
    SESSION_TOOL_COUNT_INCREMENTED,
    SESSION_ATTACHED,
    SESSION_HEARTBEAT,
    ACTOR_CREATED,
    ACTOR_WOKE,
    ACTOR_SLEPT,
    ACTOR_STOPPED,
    ACTOR_CRASHED,
    GATE_PASSED,
    GATE_FAILED,
    JOB_CREATED,
    JOB_CLAIMED,
    JOB_COMPLETED,
    JOB_FAILED,
    JOB_DISPATCHED,
    JOB_UPDATE,
    CONFIG_CHANGED,
    PROJECT_LOG,
    SKILL_STARTED,
    SKILL_COMPLETED,
    SKILL_FAILED,
    SUPERVISOR_SHUTTING_DOWN,
    SUPERVISOR_SWEEP_STARTED,
    SUPERVISOR_SWEEP_COMPLETED,
    SUPERVISOR_DRIFT_DETECTED,
    SUPERVISOR_COHERENCE_RESTORED,
    PROCESS_SPAWNED,
    PROCESS_EXITED,
    PROCESS_SIGNAL_SENT,
    PROCESS_ORPHANED,
    PROCESS_KILLED,
    EXECUTOR_SESSION_CREATED,
    EXECUTOR_SESSION_DESTROYED,
    EXECUTOR_WINDOW_CREATED,
    EXECUTOR_WINDOW_DESTROYED,
    DAEMON_STARTUP_COMPLETED,
    DAEMON_SHUTDOWN_COMPLETED,
    DAEMON_CRASHED,
    DAEMON_PORT_BIND_FAILED,
    DAEMON_PIDFILE_ACQUIRED,
    DAEMON_PIDFILE_STALE_TAKEN,
    JOB_STEP_COMPLETED,
    JOB_STEP_FAILED,
    JOB_CANCELLED,
    JOB_MERGED,
    JOB_SPAWN_FAILED,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_constants_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for &t in ALL {
            assert!(seen.insert(t), "duplicate event type constant: {t}");
        }
    }
}
