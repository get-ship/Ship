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

    /// Transform a child spawn so the process lands in agent `id`'s **process
    /// container** — the inverse of [`process_container`]: this *creates* what
    /// that later reads back. Returns the `(program, args)` to actually spawn;
    /// the caller keeps ownership of stdio/env/cwd and spawns normally.
    ///
    /// Linux wraps the command in a **systemd transient user scope**
    /// (`ship-agent-<id>.scope`) — the cgroup the resolver maps to the agent.
    /// Returns the input **unchanged** when no backend is available (the child
    /// still runs; it simply resolves as the operator), so a missing or disabled
    /// systemd never blocks an agent from starting. Identity, not isolation:
    /// resource limits on the scope are a later concern.
    fn containerize(&self, id: &str, program: &str, args: &[String]) -> (String, Vec<String>);

    /// Whether this platform can place an agent in a **named** process container
    /// that [`process_container`] reads back (today: Linux cgroups). This gates
    /// the resolver's fail-closed posture: where containers *are* supported, an
    /// unreadable container must fail closed (it could be a transient `/proc`
    /// race on a real agent — never silently promote to operator); where they
    /// are *not* (macOS/Windows until their backends land), a same-owner peer
    /// is the operator, because no agent could have been containerized anyway.
    fn containers_supported(&self) -> bool;
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
/// - **macOS:** `LOCAL_PEERPID` (pid) + `getpeereid` (uid/gid).
/// - **Other unix:** `None` (callers fail closed).
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
    #[cfg(target_os = "macos")]
    {
        // macOS has no SO_PEERCRED: the peer pid comes from LOCAL_PEERPID, and
        // uid/gid from getpeereid. NOTE: written against the documented libc API;
        // not compile-verified on this Linux dev host — validate on macOS hardware.
        let mut pid: libc::pid_t = 0;
        let mut len = std::mem::size_of::<libc::pid_t>() as libc::socklen_t;
        // SAFETY: getsockopt writes the peer pid into `pid`; `len` is its size.
        let rc = unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_LOCAL,
                libc::LOCAL_PEERPID,
                std::ptr::addr_of_mut!(pid) as *mut libc::c_void,
                &mut len,
            )
        };
        if rc != 0 || pid <= 0 {
            return None;
        }
        let mut uid: libc::uid_t = 0;
        let mut gid: libc::gid_t = 0;
        // SAFETY: getpeereid writes the connected socket's peer euid/egid.
        let rc2 = unsafe { libc::getpeereid(fd, &mut uid, &mut gid) };
        if rc2 != 0 {
            return None;
        }
        return Some(PeerCred {
            pid: pid as u32,
            uid: uid as u32,
            gid: gid as u32,
        });
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = fd;
        None
    }
}

/// The uid Ship is running as — the operator's uid. Used to recognize a
/// same-owner peer as the operator on platforms without a named container
/// backend. Unix only; on other platforms there is no host socket to resolve.
#[cfg(unix)]
pub fn current_uid() -> u32 {
    // SAFETY: getuid is always successful and has no preconditions.
    unsafe { libc::getuid() }
}

/// Non-unix stub: no host socket exists on these platforms, so this is never
/// consulted for a connection; a sentinel that won't match any real peer uid.
#[cfg(not(unix))]
pub fn current_uid() -> u32 {
    u32::MAX
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

    fn containerize(&self, id: &str, program: &str, args: &[String]) -> (String, Vec<String>) {
        if is_safe_unit_id(id) && systemd_user_available() {
            systemd_scope_argv(id, program, args)
        } else {
            (program.to_string(), args.to_vec())
        }
    }

    fn containers_supported(&self) -> bool {
        true
    }
}

/// systemd-run argv that runs `program args` in a transient user scope named
/// `ship-agent-<id>.scope` — the cgroup [`Linux::process_container`] reads back.
/// `--scope` runs synchronously in the foreground, **inheriting our stdio/tty**
/// (so a PTY-attached harness still sees a tty); `--collect` frees the unit on
/// exit; `--quiet` drops the "Running as unit" banner. Pure — the unit-tested
/// core of the backend.
#[cfg(target_os = "linux")]
fn systemd_scope_argv(id: &str, program: &str, args: &[String]) -> (String, Vec<String>) {
    let mut out = vec![
        "--user".to_string(),
        "--scope".to_string(),
        format!("--unit=ship-agent-{id}"),
        "--quiet".to_string(),
        "--collect".to_string(),
        "--".to_string(),
        program.to_string(),
    ];
    out.extend(args.iter().cloned());
    ("systemd-run".to_string(), out)
}

/// A systemd unit name accepts `[A-Za-z0-9:_.-]`. Reject anything else so an
/// unusual agent id can't produce a malformed `--unit` (which would fail the
/// spawn); such ids fall back to a bare, uncontainerized spawn.
#[cfg(target_os = "linux")]
fn is_safe_unit_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '.' | '-'))
}

/// Is a systemd **user** manager live and `systemd-run` on PATH? Cached once.
/// `SHIP_NO_CONTAINERIZE=1` is an escape hatch (forces the bare-spawn fallback).
/// The private socket under `$XDG_RUNTIME_DIR/systemd/` is the cheap liveness
/// signal — no subprocess, no D-Bus round-trip.
#[cfg(target_os = "linux")]
fn systemd_user_available() -> bool {
    use std::sync::OnceLock;
    static AVAIL: OnceLock<bool> = OnceLock::new();
    *AVAIL.get_or_init(|| {
        if std::env::var_os("SHIP_NO_CONTAINERIZE").is_some() {
            return false;
        }
        let Some(xdg) = std::env::var_os("XDG_RUNTIME_DIR") else {
            return false;
        };
        let live = std::path::Path::new(&xdg).join("systemd/private").exists();
        live && systemd_run_on_path()
    })
}

#[cfg(target_os = "linux")]
fn systemd_run_on_path() -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join("systemd-run").is_file())
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

    fn containerize(&self, _id: &str, program: &str, args: &[String]) -> (String, Vec<String>) {
        // No container backend yet (macOS process-group / Windows job object):
        // run the command as-is. The agent resolves as the operator until the
        // platform backend lands.
        (program.to_string(), args.to_vec())
    }

    fn containers_supported(&self) -> bool {
        false
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
    fn systemd_scope_argv_wraps_program_and_names_the_unit() {
        let (prog, args) =
            systemd_scope_argv("rust-lane-01", "claude", &["--print".to_string(), "hi".to_string()]);
        assert_eq!(prog, "systemd-run");
        assert!(args.contains(&"--unit=ship-agent-rust-lane-01".to_string()));
        assert!(args.contains(&"--scope".to_string()));
        // Everything after `--` is the original command, in order.
        let dd = args.iter().position(|a| a == "--").unwrap();
        assert_eq!(&args[dd + 1..], &["claude", "--print", "hi"]);
    }

    #[test]
    fn linux_supports_named_containers() {
        assert!(host().containers_supported(), "linux cgroups are a named container backend");
    }

    #[test]
    fn current_uid_is_our_uid() {
        // Matches getuid via the shell — just assert it's a plausible uid and
        // stable across calls (it reads the process's real uid).
        assert_eq!(current_uid(), current_uid());
    }

    #[test]
    fn unit_id_safety() {
        assert!(is_safe_unit_id("rust-lane-01"));
        assert!(is_safe_unit_id("commander_2.x"));
        assert!(!is_safe_unit_id(""));
        assert!(!is_safe_unit_id("bad id")); // space
        assert!(!is_safe_unit_id("../escape")); // slash
    }

    #[test]
    fn containerize_places_child_in_agent_scope_or_noops() {
        // Real e2e where a systemd user manager is live (this WSL+systemd host):
        // the wrapped `cat` runs inside `ship-agent-<id>.scope`, so its own
        // cgroup names the agent. Degrades to a bare spawn elsewhere.
        let (prog, args) =
            host().containerize("probe-itest", "cat", &["/proc/self/cgroup".to_string()]);
        let out = std::process::Command::new(&prog)
            .args(&args)
            .output()
            .expect("spawn containerized probe");
        let stdout = String::from_utf8_lossy(&out.stdout);
        if prog == "systemd-run" {
            assert!(
                stdout.contains("ship-agent-probe-itest"),
                "child should be in its agent scope, got cgroup: {stdout}"
            );
        } else {
            assert_eq!(prog, "cat", "no backend → command runs unchanged");
        }
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
