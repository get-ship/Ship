//! Peer identity — who is on the other end of a daemon connection, resolved
//! from a **kernel-verified** process attribute, never a self-declared string.
//!
//! Each dispatched agent is spawned into its own kernel-tracked process
//! container (`ship-host` abstracts this per OS: Linux cgroup, macOS
//! process-group, Windows job object). A connection's peer PID — reported by
//! the kernel via the socket's peer credentials — maps to that container, and
//! the container names the agent. The agent sends no identity claim; the kernel
//! reports who is calling and the daemon owns the map. Unspoofable: a process
//! can't fake its PID to the kernel, nor join another agent's container without
//! privilege.
//!
//! Layer split: the OS read (`pid → container`) is L0 (`ship-host`); this
//! module is the L1 seam that parses Ship's `ship-agent-<id>` convention out of
//! the container and composes `agent_id → resolve_grants` (#85).

/// Who a connection's peer process belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerIdentity {
    /// A dispatched agent — the peer is inside that agent's process container.
    /// Carries the agent id parsed from the container name.
    Agent(String),
    /// Not inside any agent container — the operator's own process (the human
    /// session that owns the daemon).
    Operator,
    /// The host backend can't report the peer's container (dead process, or a
    /// platform whose `ship-host` backend isn't built yet). The gate MUST treat
    /// this as least-privilege — fail closed; never promote an unknown peer to
    /// operator.
    Unresolved,
}

/// Resolve the identity of the process at `peer_pid` (kernel-verified, taken
/// from the connection's peer credentials). Reads the OS process container via
/// `ship-host`; an agent container names the agent, anything else is the
/// operator, an unreadable container fails closed.
pub fn resolve(peer_pid: u32) -> PeerIdentity {
    match ship_host::host().process_container(peer_pid) {
        Some(container) => match agent_from_container(&container) {
            Some(id) => PeerIdentity::Agent(id),
            None => PeerIdentity::Operator,
        },
        None => PeerIdentity::Unresolved,
    }
}

/// Extract the Ship agent id from a process-container path. The spawner places
/// each agent in a container whose leaf names it — `ship-agent-<id>` (with an
/// optional `.scope`/`.slice` suffix). `None` = not a Ship agent container
/// (operator or some other process). Pure — the unit-tested core; OS-agnostic
/// (the OS read lives in `ship-host`).
fn agent_from_container(path: &str) -> Option<String> {
    path.split('/').find_map(|seg| {
        let seg = seg.strip_suffix(".scope").unwrap_or(seg);
        let seg = seg.strip_suffix(".slice").unwrap_or(seg);
        seg.strip_prefix("ship-agent-")
            .filter(|id| !id.is_empty())
            .map(str::to_string)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_container_names_the_agent() {
        assert_eq!(
            agent_from_container("/ship.slice/ship-agent-rust-lane-01.scope").as_deref(),
            Some("rust-lane-01")
        );
    }

    #[test]
    fn operator_session_container_is_not_an_agent() {
        assert_eq!(agent_from_container("/user.slice/user-1000.slice/session-2.scope"), None);
    }

    #[test]
    fn agent_without_scope_suffix_resolves() {
        assert_eq!(agent_from_container("/ship-agent-bare").as_deref(), Some("bare"));
    }

    #[test]
    fn empty_or_prefix_without_id_is_not_an_agent() {
        assert_eq!(agent_from_container(""), None);
        assert_eq!(agent_from_container("/ship.slice/ship-agent-.scope"), None);
    }
}
