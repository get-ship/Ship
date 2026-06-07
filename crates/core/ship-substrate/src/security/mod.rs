//! Content security scanner for skill and prompt files.
//!
//! Detects hidden Unicode characters that are invisible to humans but
//! tokenized by LLMs — a prompt injection vector. Scans run automatically
//! on `ship install` and are available via `ship audit`.
//!
//! Based on APM's ContentScanner (MIT licensed, Microsoft).
//!
//! Split across two files to stay under the file cap: `ranges` owns the
//! classification table, `scan` owns the walk/report logic. The public surface
//! is re-exported here so callers continue to use `config::security::*`.

mod ranges;
mod scan;

pub use ranges::Severity;
pub use scan::{has_critical, scan_dir, scan_file, scan_text, summarize, Finding};
