//! Listener binding with port-collision retry, plus the direct-to-store
//! daemon.* event emit used where the kernel may not be live.

use anyhow::{Result, anyhow};
use ship_substrate::events::daemon::{PortBindFailedPayload, event_types as daemon_events};
use ship_substrate::events::{EventEnvelope, SqliteEventStore};

/// Bind the listener with collision retry. On EADDRINUSE we increment
/// the port up to 9 times before giving up (unless the caller pinned a
/// port, in which case one attempt). The actually-bound port is what the
/// caller writes to `<daemon_dir>/port` — the derived port is just a hint.
pub(super) async fn bind_with_retry(
    host: &str,
    desired_port: u16,
    user_pinned: bool,
    global_dir: &std::path::Path,
) -> Result<(tokio::net::TcpListener, u16)> {
    let max_retries = if user_pinned { 1 } else { 10 };
    let mut bound_port = desired_port;
    let listener = loop {
        let addr = format!("{host}:{bound_port}");
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => break l,
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse && !user_pinned => {
                tracing::warn!(
                    port = bound_port,
                    "shipd: port busy, trying next"
                );
                bound_port += 1;
                if bound_port - desired_port >= max_retries {
                    let already_bound_by_pid = pidfile_pid_for_port(global_dir, desired_port);
                    emit_daemon_event(
                        global_dir,
                        daemon_events::DAEMON_PORT_BIND_FAILED,
                        "daemon",
                        &PortBindFailedPayload {
                            port: desired_port,
                            already_bound_by_pid,
                            error: e.to_string(),
                        },
                    );
                    return Err(anyhow!(
                        "bind {host}:{desired_port}..{}: {} (giving up after {max_retries} retries)",
                        bound_port - 1,
                        e,
                    ));
                }
            }
            Err(e) => {
                let already_bound_by_pid = pidfile_pid_for_port(global_dir, bound_port);
                emit_daemon_event(
                    global_dir,
                    daemon_events::DAEMON_PORT_BIND_FAILED,
                    "daemon",
                    &PortBindFailedPayload {
                        port: bound_port,
                        already_bound_by_pid,
                        error: e.to_string(),
                    },
                );
                return Err(anyhow!("bind {host}:{bound_port}: {e}"));
            }
        }
    };
    Ok((listener, bound_port))
}

/// Append a daemon.* event directly to the runtime event store.
///
/// Used in paths where the kernel may not be live (port bind failure) or where
/// the runtime is already torn down (post-serve shutdown). Fire-and-forget:
/// telemetry must never block lifecycle decisions.
pub(super) fn emit_daemon_event<P: serde::Serialize>(
    global_dir: &std::path::Path,
    event_type: &str,
    entity_id: &str,
    payload: &P,
) {
    let envelope = match EventEnvelope::new(event_type, entity_id, payload) {
        Ok(e) => e.with_actor_id("daemon"),
        Err(e) => {
            tracing::warn!("shipd: cannot build {event_type} envelope: {e}");
            return;
        }
    };
    // daemon.* is Runtime-scoped → write runtime.db (not the dead platform.db).
    let store = match SqliteEventStore::new_for_scope(
        global_dir,
        ship_substrate::events::EventScope::Runtime,
        None,
    ) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("shipd: cannot open event store for {event_type}: {e}");
            return;
        }
    };
    // Enforced write: daemon.* is Runtime-scoped and this path may run with no
    // live kernel/session, so stamp + enforce from the daemon-internal (Runtime)
    // binding before persist.
    let binding = ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk);
    if let Err(e) = store.append_enforced(&envelope, &binding) {
        tracing::warn!("shipd: cannot append {event_type}: {e}");
    }
}

/// Best-effort lookup of the pid currently holding `port`, sourced from the
/// supervisor's lock-based pidfile. Returns None if the file is missing,
/// unparseable, or records a different port.
fn pidfile_pid_for_port(global_dir: &std::path::Path, port: u16) -> Option<u32> {
    let path = ship_substrate::ShipHome::at(global_dir.to_path_buf())
        .machine_dir()
        .join("daemon")
        .join("shipd.pid");
    let data = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(data.trim()).ok()?;
    let recorded_port = v.get("port")?.as_u64()? as u16;
    if recorded_port != port {
        return None;
    }
    Some(v.get("pid")?.as_u64()? as u32)
}
