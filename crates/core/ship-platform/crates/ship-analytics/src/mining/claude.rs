//! Claude data-dir mining — read transcripts from `~/.claude/projects/`.
//!
//! Format (subject to drift):
//! ```text
//! ~/.claude/projects/
//!   <url_encoded_cwd>/                     (one dir per project root)
//!     <session_uuid>.jsonl                 (one file per session)
//! ```
//! Each `.jsonl` file is line-delimited JSON. Each line is a message
//! object — user turn, assistant turn (with text + tool_use blocks),
//! or tool_result. Usage info appears in assistant messages as
//! `message.usage = { input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens }`.
//!
//! Implementation status: stub. The struct shape, scan signature,
//! and watch signature are pinned so the rest of ship-analytics can
//! depend on them. The parsing body lands when we commit to the
//! mining feature for v1 analytics.

use std::path::{Path, PathBuf};

use anyhow::Result;

/// One transcript row mined from a `.jsonl` line. Subset of what's
/// actually there — we only extract what AnalyticsProjection wants
/// (session lifecycle + tool calls + usage). Full transcript text
/// is referenced by file path + line offset; we don't duplicate it.
#[derive(Debug, Clone)]
pub struct MinedRow {
    pub session_uuid: String,
    pub kind: MinedKind,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub enum MinedKind {
    SessionStart {
        model: Option<String>,
        cwd: Option<String>,
    },
    ToolUse {
        tool: String,
    },
    ToolResult {
        tool: String,
        is_error: bool,
    },
    UsageObserved {
        model: String,
        input_tokens: u64,
        output_tokens: u64,
        cache_creation_input_tokens: u64,
        cache_read_input_tokens: u64,
    },
    SessionEnd,
}

/// Default location of Claude's data directory.
pub fn default_data_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".claude").join("projects"))
}

/// One-shot scan of every `.jsonl` under `data_dir`. Returns parsed
/// rows in undefined order (callers should sort by `timestamp` if
/// they need chronological).
///
/// STUB: returns `Ok(vec![])`. Real implementation walks the dir,
/// reads each file, parses lines, extracts the MinedKind variants.
pub fn scan(data_dir: &Path) -> Result<Vec<MinedRow>> {
    if !data_dir.exists() {
        return Ok(Vec::new());
    }
    // TODO: walk dir; for each session file, read lines; for each
    // line, parse JSON; classify into MinedKind; push row.
    Ok(Vec::new())
}

/// Set up a file-watcher that emits MinedRow as new lines appear.
///
/// STUB: not implemented. Will use the `notify` crate (already a
/// dependency elsewhere) with a polling fallback for WSL where
/// inotify is unreliable.
#[allow(dead_code)]
pub fn watch(_data_dir: &Path) -> Result<()> {
    Ok(())
}
