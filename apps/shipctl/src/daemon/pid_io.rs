use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Read a PID from the pid file. Returns None if the file does not exist.
pub fn read_pid(pid_path: &std::path::Path) -> Result<Option<u32>> {
    if !pid_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(pid_path)
        .with_context(|| format!("cannot read {}", pid_path.display()))?;
    let pid: u32 = content
        .trim()
        .parse()
        .with_context(|| format!("invalid PID in {}", pid_path.display()))?;
    Ok(Some(pid))
}

/// Write a PID to the pid file atomically.
///
/// Census §2: a plain `std::fs::write` is non-atomic — an interrupt mid-write
/// can leave a truncated pidfile. The supervisor's pidfile also races with
/// `pidfile::acquire`'s flock-guarded record; the legacy `pid_io` path was
/// the weaker of the two, so it is rewritten to mirror `fs_util::write_atomic`:
/// temp file in the same dir, `write_all` + `sync_all`, then `fs::rename` over
/// the destination. `fs_util` is not a direct dep of this crate, so the
/// primitive is inlined here (one way to do things).
pub fn write_pid(pid_path: &std::path::Path, pid: u32) -> Result<()> {
    if let Some(parent) = pid_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create directory {}", parent.display()))?;
    }
    let dir = pid_path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = pid_path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    let own_pid = std::process::id();
    let bytes = format!("{}\n", pid);

    for attempt in 0..16u8 {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let tmp_path = dir.join(format!(".tmp-{}-{}-{}-{}", file_name, own_pid, nonce, attempt));

        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
        {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(err).with_context(|| {
                    format!("cannot create temp pidfile: {}", tmp_path.display())
                });
            }
        };

        file.write_all(bytes.as_bytes()).with_context(|| {
            format!("cannot write temp pidfile: {}", tmp_path.display())
        })?;
        // Flush to stable storage BEFORE the rename: a crash between write and
        // rename must not surface a truncated/empty pidfile at the destination.
        file.sync_all()
            .with_context(|| format!("cannot sync temp pidfile: {}", tmp_path.display()))?;
        drop(file);

        std::fs::rename(&tmp_path, pid_path).with_context(|| {
            format!(
                "cannot rename {} -> {}",
                tmp_path.display(),
                pid_path.display()
            )
        })?;
        return Ok(());
    }

    Err(anyhow::anyhow!(
        "cannot allocate unique temp pidfile for {}",
        pid_path.display()
    ))
}

/// Check whether a process with the given PID is alive by sending signal 0.
pub(crate) fn is_process_alive(pid: u32) -> bool {
    let status = std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status();
    matches!(status, Ok(s) if s.success())
}
