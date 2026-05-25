//! Peer identity — who is on the other end of a daemon connection, resolved
//! from a **kernel-verified** process attribute, never a self-declared string.
//!
//! Each dispatched agent is spawned into its own kernel-tracked process
//! container (Linux **cgroup**, macOS **process-group**, Windows **job
//! object**). A connection's peer PID — reported by the kernel via the socket's
//! peer credentials (`SO_PEERCRED` / `LOCAL_PEERPID` / `GetNamedPipeClient­ProcessId`)
//! — maps to that container, and the container names the agent. The agent sends
//! no identity claim; the kernel reports who is calling and the daemon owns the
//! map. Unspoofable: a process can't fake its PID to the kernel, nor join
//! another agent's container without privilege.
//!
//! This module is the single daemon-side resolver seam: `peer_pid → identity`.
//! The gate then composes `identity → agent_id → resolve_grants` (#85). Spawning
//! agents INTO their containers + binding the peer-credential socket are separate
//! wiring steps; this is the platform-abstracted resolver they plug into.

/// Who a connection's peer process belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerIdentity {
    /// A dispatched agent — the peer is inside that agent's process container.
    /// Carries the agent id parsed from the container name.
    Agent(String),
    /// Not inside any agent container — the operator's own process (the human
    /// session that owns the daemon).
    Operator,
    /// The platform backend can't yet determine identity (e.g. a stubbed OS).
    /// The gate MUST treat this as least-privilege (no restricted grants) — fail
    /// closed; never silently promote an unknown peer to operator.
    Unresolved,
}

/// Resolve the identity of the process at `peer_pid` (kernel-verified, taken
/// from the connection's peer credentials). Platform-specific.
pub trait IdentityResolver: Send + Sync {
    fn resolve(&self, peer_pid: u32) -> PeerIdentity;
}

/// The resolver for the current platform. Linux uses cgroups; other platforms
/// are stubbed (`Unresolved`) until their process-container backend lands.
pub fn default_resolver() -> Box<dyn IdentityResolver> {
    #[cfg(target_os = "linux")]
    {
        Box::new(CgroupResolver)
    }
    #[cfg(not(target_os = "linux"))]
    {
        Box::new(UnsupportedResolver)
    }
}

// ── Linux: cgroup backend ─────────────────────────────────────────────────────

/// Linux resolver: read the peer's cgroup from `/proc/<pid>/cgroup` and extract
/// the agent id from the Ship container segment. A PID not inside a `ship-agent-*`
/// container is the operator.
#[cfg(target_os = "linux")]
pub struct CgroupResolver;

#[cfg(target_os = "linux")]
impl IdentityResolver for CgroupResolver {
    fn resolve(&self, peer_pid: u32) -> PeerIdentity {
        match read_cgroup(peer_pid) {
            Some(content) => match agent_from_cgroup(&content) {
                Some(id) => PeerIdentity::Agent(id),
                None => PeerIdentity::Operator,
            },
            // No cgroup readable (dead process / race) → fail closed.
            None => PeerIdentity::Unresolved,
        }
    }
}

#[cfg(target_os = "linux")]
fn read_cgroup(pid: u32) -> Option<String> {
    std::fs::read_to_string(format!("/proc/{pid}/cgroup")).ok()
}

/// Non-Linux stub — until the process-group (macOS) / job-object (Windows)
/// backends land, identity is `Unresolved` (the gate fails closed). Never
/// returns `Operator`: an unbuilt backend must not hand out operator authority.
#[cfg(not(target_os = "linux"))]
pub struct UnsupportedResolver;

#[cfg(not(target_os = "linux"))]
impl IdentityResolver for UnsupportedResolver {
    fn resolve(&self, _peer_pid: u32) -> PeerIdentity {
        PeerIdentity::Unresolved
    }
}

/// Extract the Ship agent id from a `/proc/<pid>/cgroup` file. The spawner
/// places each agent in a cgroup whose leaf names it: `ship-agent-<id>` (with an
/// optional `.scope`/`.slice` suffix). Handles both cgroup v2 (`0::<path>`) and
/// v1 (`<n>:<controllers>:<path>`) line shapes. `None` = not in a Ship agent
/// container (operator or some other process).
///
/// Pure + total — the unit-tested core; `resolve` is just this over `/proc`.
fn agent_from_cgroup(content: &str) -> Option<String> {
    for line in content.lines() {
        // The path is the substring after the last ':' (v2: "0::/p"; v1: "n:c:/p").
        let path = line.rsplit_once(':').map(|(_, p)| p).unwrap_or(line);
        for seg in path.split('/') {
            let seg = seg.strip_suffix(".scope").unwrap_or(seg);
            let seg = seg.strip_suffix(".slice").unwrap_or(seg);
            if let Some(id) = seg.strip_prefix("ship-agent-")
                && !id.is_empty()
            {
                return Some(id.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cgroup_v2_agent_container_resolves_the_agent() {
        // cgroup v2: one "0::<path>" line.
        let c = "0::/ship.slice/ship-agent-rust-lane-01.scope\n";
        assert_eq!(agent_from_cgroup(c).as_deref(), Some("rust-lane-01"));
    }

    #[test]
    fn cgroup_v1_multiline_agent_container_resolves() {
        let c = "12:pids:/ship.slice/ship-agent-reviewer.scope\n\
                 0::/ship.slice/ship-agent-reviewer.scope\n";
        assert_eq!(agent_from_cgroup(c).as_deref(), Some("reviewer"));
    }

    #[test]
    fn operator_session_cgroup_is_not_an_agent() {
        // A normal user session — no ship-agent container.
        let c = "0::/user.slice/user-1000.slice/session-2.scope\n";
        assert_eq!(agent_from_cgroup(c), None);
    }

    #[test]
    fn agent_id_without_scope_suffix_resolves() {
        assert_eq!(agent_from_cgroup("0::/ship-agent-bare").as_deref(), Some("bare"));
    }

    #[test]
    fn empty_or_garbage_is_not_an_agent() {
        assert_eq!(agent_from_cgroup(""), None);
        assert_eq!(agent_from_cgroup("not a cgroup file"), None);
        // The prefix with no id is not a valid agent container.
        assert_eq!(agent_from_cgroup("0::/ship.slice/ship-agent-.scope"), None);
    }
}
