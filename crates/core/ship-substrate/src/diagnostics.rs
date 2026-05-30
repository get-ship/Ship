//! DiagnosticSnapshot — per-subsystem observability trait + registry.
//!
//! Every major subsystem implements [`DiagnosticSnapshot`]. The registry
//! aggregates them into a single JSON tree, read via the `diagnostics.snapshot`
//! op. Designed for O(1)/O(log n) snapshots so callers can poll every few
//! seconds during a live leak hunt without locking the system.
//!
//! # Usage
//!
//! ```rust,ignore
//! let mut reg = DiagnosticRegistry::new();
//! reg.register(Arc::new(my_subsystem.diagnostic_view()));
//! let snapshot = reg.snapshot();
//! ```

use std::sync::{Arc, Mutex};
use serde_json::{Value, json};

/// A subsystem that can report its current in-process state.
///
/// Implementations must be `Send + Sync` (held in an `Arc`) and must
/// return from `snapshot()` in O(1) or O(log n) time. No allocations
/// proportional to stored event history — just count/size fields.
pub trait DiagnosticSnapshot: Send + Sync {
    /// Stable identifier in the aggregated tree.
    /// Examples: `"kernel_router"`, `"supervisor"`, `"session_registry"`.
    fn name(&self) -> &'static str;

    /// Snapshot the subsystem's current state. Returns a JSON object with
    /// at minimum a `bytes_estimated` field (best-effort allocation size)
    /// and `count_*` fields for every meaningful container.
    fn snapshot(&self) -> Value;
}

/// Central registry holding all registered subsystem snapshots.
///
/// One instance is held on the kernel / app facade and registered at startup.
/// Thread-safe — `register` may be called from any thread.
#[derive(Default, Clone)]
pub struct DiagnosticRegistry {
    subsystems: Arc<Mutex<Vec<Arc<dyn DiagnosticSnapshot>>>>,
}

impl DiagnosticRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a subsystem snapshot provider. Idempotent by name — a second
    /// registration for the same name replaces the first.
    pub fn register(&self, provider: Arc<dyn DiagnosticSnapshot>) {
        let mut guard = self.subsystems.lock().unwrap_or_else(|e| e.into_inner());
        let name = provider.name();
        if let Some(idx) = guard.iter().position(|p| p.name() == name) {
            guard[idx] = provider;
        } else {
            guard.push(provider);
        }
    }

    /// Aggregate all registered subsystem snapshots into a single JSON object.
    /// Each subsystem's result is keyed by its name.
    pub fn snapshot_all(&self) -> Value {
        let guard = self.subsystems.lock().unwrap_or_else(|e| e.into_inner());
        let mut subsystems = serde_json::Map::new();
        for provider in guard.iter() {
            subsystems.insert(provider.name().to_string(), provider.snapshot());
        }
        json!({ "subsystems": Value::Object(subsystems) })
    }

    /// Number of registered providers.
    pub fn len(&self) -> usize {
        self.subsystems.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Read process RSS from `/proc/self/status` on Linux. Returns `None` on other
/// platforms or if the file cannot be parsed.
pub fn process_rss_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let data = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in data.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
                return Some(kb * 1024);
            }
        }
        None
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Counted {
        count: usize,
    }

    impl DiagnosticSnapshot for Counted {
        fn name(&self) -> &'static str { "test_counted" }
        fn snapshot(&self) -> Value {
            json!({ "bytes_estimated": 0, "count_items": self.count })
        }
    }

    #[test]
    fn registry_aggregates_snapshots() {
        let reg = DiagnosticRegistry::new();
        reg.register(Arc::new(Counted { count: 7 }));
        let snap = reg.snapshot_all();
        assert_eq!(snap["subsystems"]["test_counted"]["count_items"], 7);
    }

    #[test]
    fn registry_replace_by_name() {
        let reg = DiagnosticRegistry::new();
        reg.register(Arc::new(Counted { count: 1 }));
        reg.register(Arc::new(Counted { count: 2 }));
        let snap = reg.snapshot_all();
        // Second registration with same name replaces first.
        assert_eq!(snap["subsystems"]["test_counted"]["count_items"], 2);
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn process_rss_bytes_is_reasonable() {
        if let Some(rss) = process_rss_bytes() {
            // Any running process should have at least 1 MB RSS.
            assert!(rss > 1024 * 1024, "rss {rss} suspiciously small");
        }
        // On non-Linux platforms None is acceptable.
    }
}
