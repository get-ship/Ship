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
/// `ship-host` and decides per [`decide`].
pub fn resolve(peer_pid: u32, peer_uid: u32) -> PeerIdentity {
    let host = ship_host::host();
    decide(
        host.process_container(peer_pid).as_deref(),
        host.containers_supported(),
        peer_uid,
        ship_host::current_uid(),
    )
}

/// The pure resolution policy (unit-tested independent of the OS):
///
/// - **agent container** (`ship-agent-<id>`) → [`PeerIdentity::Agent`].
/// - **container present but not an agent** → the operator's own process tree,
///   confirmed by same-owner uid.
/// - **no container, backend NOT supported** (macOS/Windows today) → a
///   same-owner peer is the operator; no agent could have been containerized.
/// - **no container, backend IS supported** (Linux) → [`PeerIdentity::Unresolved`]:
///   fail closed. An unreadable cgroup could be a transient race on a real
///   agent — never silently promote it to operator.
/// - **different owner uid** anywhere → [`PeerIdentity::Unresolved`] (defense in
///   depth; the `0600` socket already restricts to our uid).
fn decide(
    container: Option<&str>,
    containers_supported: bool,
    peer_uid: u32,
    host_uid: u32,
) -> PeerIdentity {
    if let Some(c) = container {
        if let Some(id) = agent_from_container(c) {
            return PeerIdentity::Agent(id);
        }
        return owner_or_unresolved(peer_uid, host_uid);
    }
    if containers_supported {
        return PeerIdentity::Unresolved;
    }
    owner_or_unresolved(peer_uid, host_uid)
}

/// A peer running as our own uid is the operator; anyone else is unresolved.
fn owner_or_unresolved(peer_uid: u32, host_uid: u32) -> PeerIdentity {
    if peer_uid == host_uid {
        PeerIdentity::Operator
    } else {
        PeerIdentity::Unresolved
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

    const ME: u32 = 1000;
    const OTHER: u32 = 1001;

    #[test]
    fn agent_container_resolves_regardless_of_uid() {
        // The container name is authoritative for an agent; uid isn't consulted.
        let id = decide(Some("/ship.slice/ship-agent-x.scope"), true, OTHER, ME);
        assert_eq!(id, PeerIdentity::Agent("x".into()));
    }

    #[test]
    fn non_agent_container_same_owner_is_operator() {
        let id = decide(Some("/user.slice/session-2.scope"), true, ME, ME);
        assert_eq!(id, PeerIdentity::Operator);
    }

    #[test]
    fn linux_no_container_fails_closed() {
        // Backend supported (Linux) but no container read → never escalate.
        assert_eq!(decide(None, true, ME, ME), PeerIdentity::Unresolved);
    }

    #[test]
    fn no_backend_same_owner_is_operator() {
        // macOS/Windows today: no named container, same-uid peer is the operator.
        assert_eq!(decide(None, false, ME, ME), PeerIdentity::Operator);
    }

    #[test]
    fn foreign_uid_never_resolves() {
        // Defense in depth beyond the 0600 socket: a different-owner peer is
        // never the operator, with or without a backend.
        assert_eq!(decide(None, false, OTHER, ME), PeerIdentity::Unresolved);
        assert_eq!(decide(Some("/user.slice/x.scope"), true, OTHER, ME), PeerIdentity::Unresolved);
    }
}
