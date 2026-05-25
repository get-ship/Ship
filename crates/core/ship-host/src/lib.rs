//! Ship's host-OS adapter layer (L0).
//!
//! Ship is becoming its own runtime, with its own notions of processes
//! (agents/actors), identity, capabilities, and isolation. This crate is the
//! **adapter to the underlying operating system** — the narrow set of host
//! primitives that runtime needs, behind one trait, with a per-OS
//! implementation. Ship's own model lives *above* this; here is the only place
//! that touches raw OS process state.
//!
//! Leaf crate: it depends only on the OS, never on other Ship crates, so every
//! layer (substrate, platform, supervisor, isolation) can build on it without a
//! dependency cycle.
//!
//! First primitive: the **process container** — the kernel-tracked grouping a
//! process belongs to (Linux cgroup, macOS process-group, Windows job object).
//! Peer credentials, spawn-into-container, and signals land behind the same
//! trait as the runtime grows.

/// The host OS, abstracted to the process primitives Ship needs. One impl per
/// platform; [`host`] returns the current one.
pub trait HostOs: Send + Sync {
    /// The kernel-tracked **process container** of `pid`, in OS-native string
    /// form (Linux: the cgroup path; macOS: the process-group; Windows: the job
    /// object). Higher layers parse Ship semantics (e.g. an agent id) from it.
    /// `None` when the OS can't report it — a dead process, or a platform whose
    /// backend isn't built yet (callers must fail closed, never assume trust).
    fn process_container(&self, pid: u32) -> Option<String>;
}

/// Kernel-verified credentials of the process on the other end of a connected
/// stream socket — read from the OS via peer credentials, never trusted from
/// the wire. This is the unspoofable anchor for per-agent identity: the daemon
/// reads it off the accepted connection and feeds the `pid` to the resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerCred {
    pub pid: u32,
    pub uid: u32,
    pub gid: u32,
}

/// Read the peer credentials off a connected unix-socket fd.
///
/// - **Linux:** `SO_PEERCRED` → full `pid`/`uid`/`gid`.
/// - **macOS:** pending (`LOCAL_PEERPID` + `getpeereid`) — returns `None` so
///   callers fail closed until the backend lands on real hardware.
///
/// Windows named-pipe peer identity (`GetNamedPipeClientProcessId`) is a
/// separate path — a pipe handle, not a unix fd — so it isn't this function.
#[cfg(unix)]
pub fn peer_cred(fd: std::os::unix::io::RawFd) -> Option<PeerCred> {
    #[cfg(target_os = "linux")]
    {
        let mut cred = libc::ucred { pid: 0, uid: 0, gid: 0 };
        let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        // SAFETY: getsockopt writes the kernel's view of the peer into `cred`;
        // `len` is the buffer size in/out. `fd` must be a connected socket —
        // the caller's responsibility (it just accepted it).
        let rc = unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                std::ptr::addr_of_mut!(cred) as *mut libc::c_void,
                &mut len,
            )
        };
        if rc == 0 && cred.pid > 0 {
            return Some(PeerCred {
                pid: cred.pid as u32,
                uid: cred.uid,
                gid: cred.gid,
            });
        }
        None
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = fd;
        None
    }
}

/// The host adapter for the current platform.
pub fn host() -> &'static dyn HostOs {
    #[cfg(target_os = "linux")]
    {
        &Linux
    }
    #[cfg(not(target_os = "linux"))]
    {
        &Unsupported
    }
}

// ── Linux ─────────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
struct Linux;

#[cfg(target_os = "linux")]
impl HostOs for Linux {
    fn process_container(&self, pid: u32) -> Option<String> {
        let content = std::fs::read_to_string(format!("/proc/{pid}/cgroup")).ok()?;
        parse_cgroup_path(&content)
    }
}

/// Parse `/proc/<pid>/cgroup` to the process's cgroup path. cgroup v2 is the
/// single `0::<path>` line; v1 is `<n>:<controllers>:<path>` per line (a scope's
/// path is shared across controllers). Prefer the v2 unified line, else the
/// first line; the path is the substring after the last `:`. Pure — the
/// unit-tested core of the Linux backend.
#[cfg(target_os = "linux")]
fn parse_cgroup_path(content: &str) -> Option<String> {
    let line = content
        .lines()
        .find(|l| l.starts_with("0::"))
        .or_else(|| content.lines().next())?;
    line.rsplit_once(':')
        .map(|(_, p)| p.to_string())
        .filter(|p| !p.is_empty())
}

// ── Other platforms (stubbed) ───────────────────────────────────────────────────

/// macOS (process-group) + Windows (job object) backends land here. Until then
/// `process_container` is `None` so callers fail closed.
#[cfg(not(target_os = "linux"))]
struct Unsupported;

#[cfg(not(target_os = "linux"))]
impl HostOs for Unsupported {
    fn process_container(&self, _pid: u32) -> Option<String> {
        None
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn cgroup_v2_single_line() {
        let c = "0::/ship.slice/ship-agent-rust-lane.scope\n";
        assert_eq!(parse_cgroup_path(c).as_deref(), Some("/ship.slice/ship-agent-rust-lane.scope"));
    }

    #[test]
    fn cgroup_v1_first_line_path() {
        let c = "12:pids:/ship.slice/ship-agent-y.scope\n\
                 11:memory:/ship.slice/ship-agent-y.scope\n";
        assert_eq!(parse_cgroup_path(c).as_deref(), Some("/ship.slice/ship-agent-y.scope"));
    }

    #[test]
    fn hybrid_prefers_the_v2_unified_line() {
        let c = "12:pids:/legacy\n0::/unified\n";
        assert_eq!(parse_cgroup_path(c).as_deref(), Some("/unified"));
    }

    #[test]
    fn empty_is_none() {
        assert_eq!(parse_cgroup_path(""), None);
    }

    #[test]
    fn self_process_has_a_container() {
        // Smoke: our own PID is in *some* cgroup on any Linux test host.
        let c = host().process_container(std::process::id());
        assert!(c.is_some_and(|p| p.starts_with('/')), "self cgroup path should resolve");
    }

    #[test]
    fn peer_cred_of_socketpair_reports_self() {
        // Both ends of an in-process pair are us → the kernel reports our PID.
        use std::os::unix::io::AsRawFd;
        let (a, _b) = std::os::unix::net::UnixStream::pair().unwrap();
        let cred = peer_cred(a.as_raw_fd()).expect("peer cred on a connected pair");
        assert_eq!(cred.pid, std::process::id(), "peer of an in-process pair is self");
    }
}
