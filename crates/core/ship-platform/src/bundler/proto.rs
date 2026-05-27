//! Bundler result types.
//!
//! Formerly the bun-worker JSON-line protocol; with the in-process Rolldown
//! bundler (#163) only the build manifest remains — constructed in Rust, not
//! deserialized from a worker.

use std::path::PathBuf;

/// What one target produced.
#[derive(Debug, Clone)]
pub struct BundleManifest {
    /// Logical target name (e.g. `sui-runtime`, `views/git/view`).
    pub target: String,
    /// Absolute output path.
    pub out_path: String,
    /// Output size in bytes.
    pub bytes: usize,
    /// Source inputs (entry / aggregate sources) — for watch dependency hints.
    pub inputs: Vec<String>,
}

impl BundleManifest {
    /// Output path as a `PathBuf`.
    pub fn out_path(&self) -> PathBuf {
        PathBuf::from(&self.out_path)
    }
}
