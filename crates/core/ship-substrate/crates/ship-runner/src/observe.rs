//! Domain-free observe utilities — git evidence, HEAD SHA, agent-log tail.
//!
//! Pure scanning over a `work_dir` (+ a `slug` for the per-session log path).
//! No agent / pipeline / config domain type appears in any signature, so both
//! the bare-session observe path and the pipeline step-monitor share one
//! implementation rather than each carrying a copy. Mechanism only: it shells
//! out to `git` and reads files; it never emits events or touches the kernel.

/// Scan git state in a worktree, counting commits since `start_sha`.
///
/// Returns `(commits_since_start, uncommitted_files)`. Falls back to
/// `(0, 0)` if git commands fail — never crashes.
pub fn scan_git_evidence(
    work_dir: &std::path::Path,
    start_sha: Option<&str>,
) -> (u32, u32) {
    let commits = match start_sha {
        Some(sha) => std::process::Command::new("git")
            .args(["rev-list", "--count", &format!("{sha}..HEAD")])
            .current_dir(work_dir)
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().ok())
            .unwrap_or(0),
        None => std::process::Command::new("git")
            .args(["rev-list", "--count", "HEAD", "--not", "--remotes"])
            .current_dir(work_dir)
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().ok())
            .unwrap_or(0),
    };

    let uncommitted = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(work_dir)
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .count() as u32
        })
        .unwrap_or(0);

    (commits, uncommitted)
}

/// Read the tail of the dispatched agent's log at
/// `<work_dir>/.artifacts/agent-<slug>.log`. Returns at most ~2 KiB of the
/// last ~30 non-empty lines, with ANSI escape sequences stripped. None
/// when the log doesn't exist or is empty.
pub fn scan_agent_log_tail(
    work_dir: &std::path::Path,
    slug: &str,
) -> Option<String> {
    const MAX_BYTES: usize = 2048;
    const MAX_LINES: usize = 30;

    let path = work_dir
        .join(".artifacts")
        .join(format!("agent-{slug}.log"));
    let raw = std::fs::read_to_string(&path).ok()?;
    if raw.trim().is_empty() {
        return None;
    }

    // Last MAX_BYTES. Slice on a char boundary to avoid mid-codepoint cuts.
    let start = raw.len().saturating_sub(MAX_BYTES);
    let mut cut = start;
    while cut < raw.len() && !raw.is_char_boundary(cut) {
        cut += 1;
    }
    let tail = &raw[cut..];

    // Strip ANSI escape sequences (CSI: ESC [ ... cmd) and keep last N lines.
    let cleaned = strip_ansi(tail);
    let mut lines: Vec<&str> = cleaned.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() > MAX_LINES {
        lines = lines.split_off(lines.len() - MAX_LINES);
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Strip ANSI escape sequences (CSI sequences and single-char escapes) from a
/// string. Used to clean harness log tails before they enter the event stream.
pub fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // CSI: ESC [ ... final byte in 0x40..=0x7E
            if chars.peek() == Some(&'[') {
                chars.next();
                for nc in chars.by_ref() {
                    if matches!(nc, '\x40'..='\x7E') {
                        break;
                    }
                }
                continue;
            }
            // Other escape sequences: skip the next character.
            chars.next();
            continue;
        }
        out.push(c);
    }
    out
}

/// Resolve HEAD SHA in a worktree. Returns `None` if git fails.
pub fn resolve_head_sha(work_dir: &std::path::Path) -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(work_dir)
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_csi_color_codes() {
        // "\x1b[31mred\x1b[0m" → "red"
        let input = "\x1b[31mred\x1b[0m";
        assert_eq!(strip_ansi(input), "red");
    }

    #[test]
    fn strip_ansi_passes_plain_text_through() {
        assert_eq!(strip_ansi("plain text\nline 2"), "plain text\nline 2");
    }

    #[test]
    fn strip_ansi_handles_non_csi_escape() {
        // ESC followed by a single byte (not '[') skips that byte.
        let input = "a\x1bXb";
        assert_eq!(strip_ansi(input), "ab");
    }

    #[test]
    fn scan_git_evidence_falls_back_when_not_a_repo() {
        // A non-repo dir: git commands fail → (0, 0), never panics.
        let tmp = std::env::temp_dir().join(format!("ship-runner-observe-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let (commits, uncommitted) = scan_git_evidence(&tmp, None);
        assert_eq!((commits, uncommitted), (0, 0));
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn resolve_head_sha_none_when_not_a_repo() {
        let tmp = std::env::temp_dir().join(format!("ship-runner-observe-sha-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        assert!(resolve_head_sha(&tmp).is_none());
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn scan_agent_log_tail_none_when_missing() {
        let tmp = std::env::temp_dir().join(format!("ship-runner-observe-log-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        assert!(scan_agent_log_tail(&tmp, "nope").is_none());
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn scan_agent_log_tail_reads_and_strips() {
        let tmp = std::env::temp_dir().join(format!("ship-runner-observe-read-{}", std::process::id()));
        let artifacts = tmp.join(".artifacts");
        std::fs::create_dir_all(&artifacts).unwrap();
        std::fs::write(
            artifacts.join("agent-demo.log"),
            "\x1b[31mfirst\x1b[0m\n\nsecond line\n",
        )
        .unwrap();
        let tail = scan_agent_log_tail(&tmp, "demo").expect("tail present");
        assert_eq!(tail, "first\nsecond line");
        std::fs::remove_dir_all(&tmp).ok();
    }
}
