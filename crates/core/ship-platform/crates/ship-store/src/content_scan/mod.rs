//! Content security scanner for skill and prompt files.
//!
//! Detects hidden Unicode characters that are invisible to humans but
//! tokenized by LLMs — a prompt injection vector. Scans run automatically
//! on `ship install`. Folded into ship-store (sole install-path consumer).

mod ranges;
mod scan;

pub use ranges::Severity;
pub use scan::{Finding, has_critical, scan_dir, scan_file, scan_text, summarize};
