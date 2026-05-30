pub const STARTUP_COMPLETED: &str = "daemon.startup_completed";
pub const SHUTDOWN_COMPLETED: &str = "daemon.shutdown_completed";
pub const CRASHED: &str = "daemon.crashed";
pub const PORT_BIND_FAILED: &str = "daemon.port_bind_failed";
pub const PIDFILE_ACQUIRED: &str = "daemon.pidfile_acquired";
pub const PIDFILE_STALE_TAKEN: &str = "daemon.pidfile_stale_taken";
/// Daemon-owner binding — a user paired with this daemon. `entity_id`
/// is the user_id. The daemon_owner projection materializes the latest.
pub const OWNER_BOUND: &str = "daemon.owner.bound";
/// Daemon-owner binding — the user unpaired (`ship logout`).
pub const OWNER_UNBOUND: &str = "daemon.owner.unbound";

pub const ALL_DAEMON: &[&str] = &[
    STARTUP_COMPLETED,
    SHUTDOWN_COMPLETED,
    CRASHED,
    PORT_BIND_FAILED,
    PIDFILE_ACQUIRED,
    PIDFILE_STALE_TAKEN,
    OWNER_BOUND,
    OWNER_UNBOUND,
];
