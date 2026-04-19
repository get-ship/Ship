//! Daemon pidfile + advisory lock. The OS-enforced singleton.
//!
//! The supervisor (`ship daemon supervise`) acquires an exclusive `flock(2)`
//! on `$SHIP_GLOBAL_DIR/shipd.pid` and writes a JSON record identifying
//! itself. Subsequent supervise invocations find the lock held and exit
//! cleanly. If the supervisor dies (even via SIGKILL), the kernel releases
//! the lock — the next startup takes over.
//!
//! Readers (`ship daemon status`) can check whether the lock is held
//! without acquiring it.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

const PIDFILE_NAME: &str = "shipd.pid";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PidfileRecord {
    pub pid: u32,
    pub port: u16,
    pub version: String,
    pub commit_sha: String,
    pub started_at: String,
    pub pidfile_path: String,
    pub socket_path: String,
    pub role: PidfileRole,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PidfileRole {
    Supervisor,
    Child,
}

/// Held by the running daemon. Drop releases the lock (by closing the fd)
/// and unlinks the file (best-effort). On crash the kernel releases the
/// lock; the file remains until the next startup overwrites it.
pub struct PidfileGuard {
    file: Option<File>,
    path: PathBuf,
}

impl Drop for PidfileGuard {
    fn drop(&mut self) {
        // Closing the fd releases the flock. Do this first so a subsequent
        // acquirer can proceed.
        self.file.take();
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Full path to the pidfile inside `global_dir`.
pub fn pidfile_path(global_dir: &Path) -> PathBuf {
    global_dir.join(PIDFILE_NAME)
}

/// Outcome of [`acquire`]. `Fresh` and `StaleTaken` both yield a held lock;
/// `Held` means another live process holds it.
pub enum AcquireResult {
    /// Lock acquired and the pidfile had no prior record.
    Fresh(PidfileGuard),
    /// Lock acquired by taking over a prior pidfile whose owner had died
    /// without releasing the lock cleanly. Slice 2 emits this for forensics.
    StaleTaken {
        guard: PidfileGuard,
        prior: PidfileRecord,
    },
    /// Another live process holds the lock — caller should print the
    /// existing record (via [`read`]) and exit cleanly.
    Held,
}

/// Try to acquire the singleton lock.
///
/// Returns [`AcquireResult`] so callers can distinguish a fresh lock from
/// a stale-pidfile takeover and emit the right `daemon.pidfile_*` event.
pub fn acquire(global_dir: &Path, mut record: PidfileRecord) -> Result<AcquireResult> {
    std::fs::create_dir_all(global_dir)
        .with_context(|| format!("cannot create directory {}", global_dir.display()))?;
    let path = pidfile_path(global_dir);
    record.pidfile_path = path.display().to_string();

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .with_context(|| format!("cannot open pidfile {}", path.display()))?;

    // Snapshot prior contents BEFORE taking the lock so we can tell a
    // fresh acquisition apart from a stale takeover. A parse failure is
    // treated as no prior record — the next write overwrites garbage.
    let mut prior_buf = String::new();
    let _ = file.read_to_string(&mut prior_buf);
    let prior: Option<PidfileRecord> =
        serde_json::from_str(prior_buf.trim()).ok();

    // SAFETY: flock(2) is a well-defined system call on the file descriptor
    // owned by `file`. The fd outlives this call.
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc != 0 {
        let err = std::io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(libc::EWOULDBLOCK) => Ok(AcquireResult::Held),
            _ => Err(err).with_context(|| format!("flock {}", path.display())),
        };
    }

    // Got the lock. Replace any stale contents with our record.
    file.set_len(0)
        .with_context(|| format!("truncate pidfile {}", path.display()))?;
    file.seek(SeekFrom::Start(0))
        .with_context(|| format!("seek pidfile {}", path.display()))?;
    let json = serde_json::to_vec_pretty(&record).context("serialize pidfile record")?;
    file.write_all(&json)
        .with_context(|| format!("write pidfile {}", path.display()))?;
    let _ = file.write_all(b"\n");
    let _ = file.sync_all();

    let guard = PidfileGuard {
        file: Some(file),
        path,
    };
    Ok(match prior {
        Some(prior) => AcquireResult::StaleTaken { guard, prior },
        None => AcquireResult::Fresh(guard),
    })
}

/// Read the pidfile without acquiring the lock.
///
/// Returns the parsed record plus a flag indicating whether the exclusive
/// lock is currently held by some process. `Ok(None)` when the pidfile does
/// not exist.
pub fn read(global_dir: &Path) -> Result<Option<(PidfileRecord, bool)>> {
    let path = pidfile_path(global_dir);
    if !path.exists() {
        return Ok(None);
    }
    let mut file = File::open(&path)
        .with_context(|| format!("cannot open pidfile {}", path.display()))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .with_context(|| format!("cannot read pidfile {}", path.display()))?;
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let record: PidfileRecord = serde_json::from_str(trimmed)
        .with_context(|| format!("cannot parse pidfile {}", path.display()))?;

    Ok(Some((record, probe_lock_held(&file))))
}

/// Probe whether the flock is currently held, without leaving a lasting
/// lock. Uses a non-blocking exclusive lock attempt on a second fd;
/// `EWOULDBLOCK` → held, success → released immediately.
fn probe_lock_held(file: &File) -> bool {
    // SAFETY: `file` outlives the syscall; flock takes a raw fd.
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc == 0 {
        let _ = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_UN) };
        false
    } else {
        true
    }
}

#[cfg(test)]
#[path = "pidfile_tests.rs"]
mod tests;
