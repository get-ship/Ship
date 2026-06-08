use anyhow::{Context, Result};

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

/// Write a PID to the pid file.
pub fn write_pid(pid_path: &std::path::Path, pid: u32) -> Result<()> {
    if let Some(parent) = pid_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create directory {}", parent.display()))?;
    }
    std::fs::write(pid_path, format!("{}\n", pid))
        .with_context(|| format!("cannot write {}", pid_path.display()))?;
    Ok(())
}

/// Check whether a process with the given PID is alive by sending signal 0.
pub(crate) fn is_process_alive(pid: u32) -> bool {
    let status = std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status();
    matches!(status, Ok(s) if s.success())
}
