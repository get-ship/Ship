//! Bounded line accumulator for harness stdout / PTY byte streams.
//!
//! Harness adapters parse newline-delimited stream-json. The reader loops
//! that feed them (`process` executor, agent-actor reader middleware,
//! per-turn headless reader) accumulate raw bytes until a `\n` appears,
//! then hand each complete line to `AgentDriver::parse_output`.
//!
//! On a real PTY — which the process executor uses so claude/codex see
//! `isatty()` and stay in structured-output mode — an *interactive* harness
//! renders a full-screen TUI in the alternate-screen buffer: a torrent of
//! cursor-move escape sequences and almost never a bare `\n`. A plain
//! `String` accumulator then never drains and grows at the full PTY output
//! rate. That is the shipd-leak-193 RSS leak: 80–430 MB/min while fanout
//! lanes author, OOMing the daemon (~7.5 GB after 30 min on one run).
//!
//! [`LineAccumulator`] bounds it: a single un-terminated run past `cap`
//! bytes is dropped. Real stream-json events are newline-delimited and far
//! under the cap, so adapter parsing is unaffected; only the un-parseable
//! TUI-render noise (or a pathological oversized line) is discarded, and
//! the dropped byte count is tracked for observability.

/// Max bytes held without a newline before the partial line is dropped.
/// Generous against any real single stream-json event (a large `tool_use`
/// input or assistant message), tiny against the unbounded TUI-render
/// growth it caps.
pub const DEFAULT_LINE_CAP: usize = 1 << 20; // 1 MiB

/// Accumulates byte-stream chunks and yields complete, trimmed lines while
/// holding at most `cap` bytes of an un-terminated trailing line.
pub struct LineAccumulator {
    buf: String,
    cap: usize,
    dropped_bytes: u64,
}

impl Default for LineAccumulator {
    fn default() -> Self {
        Self::with_cap(DEFAULT_LINE_CAP)
    }
}

impl LineAccumulator {
    /// New accumulator with the default 1 MiB cap.
    pub fn new() -> Self {
        Self::default()
    }

    /// New accumulator with an explicit cap (bytes). A `cap` of 0 still
    /// admits any single complete line that arrives within one `push`,
    /// but drops any trailing partial after each call.
    pub fn with_cap(cap: usize) -> Self {
        Self {
            buf: String::new(),
            cap,
            dropped_bytes: 0,
        }
    }

    /// Append `chunk` and return every complete line it completes, with
    /// trailing `\r`/`\n` trimmed and empty lines skipped (the callers all
    /// skip empties before parsing).
    ///
    /// After draining complete lines, if the un-terminated remainder
    /// exceeds `cap` it is discarded — this is the bound that stops an
    /// interactive TUI's newline-less render stream from growing without
    /// limit. Dropped bytes accrue into [`dropped_bytes`](Self::dropped_bytes).
    pub fn push(&mut self, chunk: &str) -> Vec<String> {
        self.buf.push_str(chunk);
        let mut lines = Vec::new();
        while let Some(nl) = self.buf.find('\n') {
            let line: String = self.buf.drain(..=nl).collect();
            let trimmed = line.trim_end_matches(['\n', '\r']);
            if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
            }
        }
        if self.buf.len() > self.cap {
            self.dropped_bytes += self.buf.len() as u64;
            self.buf.clear();
        }
        lines
    }

    /// Total bytes dropped to the cap so far. Lets a reader log when a
    /// stream is producing newline-less output (an interactive TUI on a
    /// PTY) instead of the expected stream-json.
    pub fn dropped_bytes(&self) -> u64 {
        self.dropped_bytes
    }

    /// Bytes currently buffered (an in-flight partial line). Test/observability.
    pub fn buffered_len(&self) -> usize {
        self.buf.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_newline_delimited_lines() {
        let mut acc = LineAccumulator::new();
        let lines = acc.push("{\"a\":1}\n{\"b\":2}\n");
        assert_eq!(lines, vec!["{\"a\":1}".to_string(), "{\"b\":2}".to_string()]);
        assert_eq!(acc.buffered_len(), 0);
        assert_eq!(acc.dropped_bytes(), 0);
    }

    #[test]
    fn reassembles_line_split_across_chunks() {
        let mut acc = LineAccumulator::new();
        assert!(acc.push("{\"a\":").is_empty(), "partial line yields nothing yet");
        let lines = acc.push("1}\n");
        assert_eq!(lines, vec!["{\"a\":1}".to_string()]);
    }

    #[test]
    fn trims_crlf_and_skips_empty_lines() {
        let mut acc = LineAccumulator::new();
        let lines = acc.push("event\r\n\r\nnext\n");
        assert_eq!(lines, vec!["event".to_string(), "next".to_string()]);
    }

    // The regression test for the leak: a long newline-less byte stream
    // (an interactive TUI rendering with cursor-move escapes) must not grow
    // the buffer without bound. Same bounded-buffer contract as the kernel
    // event_log `replay_log_tests`.
    #[test]
    fn drops_unterminated_stream_past_cap() {
        let cap = 4096;
        let mut acc = LineAccumulator::with_cap(cap);
        // Feed far more than `cap` bytes, never a newline — what a TUI on a
        // PTY produces in alternate-screen mode.
        let chunk = "\x1b[2J\x1b[H".repeat(512); // ANSI clear+home, no '\n'
        let total = 50;
        for _ in 0..total {
            let lines = acc.push(&chunk);
            assert!(lines.is_empty(), "no lines without a newline");
            assert!(
                acc.buffered_len() <= cap + chunk.len(),
                "buffer stays bounded near cap, got {}",
                acc.buffered_len()
            );
        }
        assert!(acc.dropped_bytes() > 0, "overflow was dropped, not retained");
    }

    #[test]
    fn parsing_survives_after_a_dropped_oversized_line() {
        let cap = 1024;
        let mut acc = LineAccumulator::with_cap(cap);
        // Oversized newline-less garbage gets dropped...
        acc.push(&"x".repeat(cap * 4));
        assert!(acc.dropped_bytes() > 0);
        // ...and a subsequent clean stream-json line still parses out.
        let lines = acc.push("\n{\"type\":\"result\"}\n");
        assert!(
            lines.iter().any(|l| l == "{\"type\":\"result\"}"),
            "real line recovered after drop: {lines:?}"
        );
    }
}
