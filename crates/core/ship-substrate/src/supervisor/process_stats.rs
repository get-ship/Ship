//! Per-process resource stats from `/proc`. Linux-only.
//!
//! Read on demand by the mesh snapshot endpoint — cheap, two file reads
//! per pid. Returns `None` when `/proc/{pid}` is gone (process exited)
//! or on non-Linux platforms. Callers treat absence as "no stats
//! available," not as an error.
//!
//! CPU% is computed as a delta against a previous snapshot the caller
//! holds. The first sample for a pid returns `cpu_pct = None`; the
//! second computes (Δticks / Δwall_ms) × 100 / num_cores. This avoids
//! the need for an in-memory tick history at this layer.

#[cfg(target_os = "linux")]
mod imp {
    use std::fs;

    /// One-shot read of a process's resident-set + thread + state +
    /// total CPU ticks (utime + stime). All fields are absolute; the
    /// caller computes deltas for CPU%.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ProcessStats {
        pub pid: u32,
        /// Resident set size in bytes (real memory in use).
        pub rss_bytes: u64,
        /// Virtual memory size in bytes.
        pub vsize_bytes: u64,
        /// Threads in this process.
        pub threads: u32,
        /// Linux process state code: R=running, S=sleeping, D=disk wait,
        /// Z=zombie, T=stopped, t=tracing stop, X=dead, I=idle.
        pub state: char,
        /// Sum of user + system jiffies the process has used since spawn.
        /// Subtract two snapshots and divide by SC_CLK_TCK to get CPU
        /// seconds, then by wall delta for CPU%.
        pub cpu_ticks: u64,
    }

    pub fn read_pid(pid: u32) -> Option<ProcessStats> {
        let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        // /proc/PID/stat is space-separated, but field 2 (comm) is
        // wrapped in parens and may contain spaces. Split on the
        // closing paren to skip past it cleanly.
        let after_comm = stat.rsplit_once(')').map(|(_, rest)| rest.trim())?;
        let fields: Vec<&str> = after_comm.split_ascii_whitespace().collect();
        // Indices are 0-based AFTER comm:
        //   0  state         (field 3 in proc(5))
        //   11 utime         (field 14)
        //   12 stime         (field 15)
        //   17 num_threads   (field 20)
        //   20 vsize bytes   (field 23)
        //   21 rss pages     (field 24)
        if fields.len() < 22 {
            return None;
        }
        let state = fields[0].chars().next()?;
        let utime: u64 = fields[11].parse().ok()?;
        let stime: u64 = fields[12].parse().ok()?;
        let threads: u32 = fields[17].parse().ok()?;
        let vsize: u64 = fields[20].parse().ok()?;
        let rss_pages: u64 = fields[21].parse().ok()?;
        let page_size = page_size_bytes();
        Some(ProcessStats {
            pid,
            rss_bytes: rss_pages.saturating_mul(page_size),
            vsize_bytes: vsize,
            threads,
            state,
            cpu_ticks: utime + stime,
        })
    }

    fn page_size_bytes() -> u64 {
        // SAFETY: sysconf is read-only; standard libc call.
        let v = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        if v <= 0 { 4096 } else { v as u64 }
    }

    pub fn clock_ticks_per_sec() -> u64 {
        // SAFETY: sysconf is read-only; standard libc call.
        let v = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        if v <= 0 { 100 } else { v as u64 }
    }

    pub fn num_cores() -> u32 {
        std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1)
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    #[derive(Debug, Clone, PartialEq)]
    pub struct ProcessStats {
        pub pid: u32,
        pub rss_bytes: u64,
        pub vsize_bytes: u64,
        pub threads: u32,
        pub state: char,
        pub cpu_ticks: u64,
    }
    pub fn read_pid(_pid: u32) -> Option<ProcessStats> { None }
    pub fn clock_ticks_per_sec() -> u64 { 100 }
    pub fn num_cores() -> u32 { 1 }
}

pub use imp::{clock_ticks_per_sec, num_cores, read_pid, ProcessStats};

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn read_self_returns_plausible_stats() {
        // Reading our own pid: should get nonzero rss, at least one
        // thread, a sensible state, and monotonic cpu_ticks.
        let pid = std::process::id();
        let s = read_pid(pid).expect("self read");
        assert!(s.rss_bytes > 0, "rss should be > 0");
        assert!(s.threads >= 1, "should have at least 1 thread");
        assert!(matches!(s.state, 'R' | 'S' | 'D' | 'I'), "state {}", s.state);
    }

    #[test]
    fn read_dead_pid_returns_none() {
        // pid 0 is never a real process; /proc/0 doesn't exist.
        assert!(read_pid(0).is_none());
    }
}

/// CPU% over the interval between two snapshots. Returns `None` when
/// either snapshot is missing or the wall delta is zero.
pub fn cpu_percent(
    prev: Option<&ProcessStats>,
    curr: Option<&ProcessStats>,
    wall_delta_ms: u64,
) -> Option<f32> {
    let (p, c) = (prev?, curr?);
    if wall_delta_ms == 0 {
        return None;
    }
    let dt = c.cpu_ticks.saturating_sub(p.cpu_ticks);
    let ticks_per_sec = clock_ticks_per_sec() as f64;
    let cores = num_cores() as f64;
    let cpu_secs = dt as f64 / ticks_per_sec;
    let wall_secs = wall_delta_ms as f64 / 1000.0;
    Some(((cpu_secs / wall_secs) * 100.0 / cores) as f32)
}
