//! Sessions, conversation history, and compaction-as-projection.
//!
//! Why: `run_step` is single-shot today. Multi-turn sessions need durable
//! conversation history and lossless compaction — the model sees a compact
//! *view* while the full log is always preserved and re-expandable.
//!
//! Seam: `InMemoryHistoryStore` is the stub. The durable event-sourced store
//! (append-only log, replay, crash recovery) is a later slice — swap at that
//! seam without touching `Session` or `CompactionPolicy`.

use std::collections::HashMap;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Core turn type
// ---------------------------------------------------------------------------

/// A single turn in a conversation: a role (e.g. "user", "assistant") and
/// the textual content of that turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    pub role: String,
    pub content: String,
}

impl Turn {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Turn { role: role.into(), content: content.into() }
    }
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

/// An in-memory multi-turn session keyed by `id`.
/// `history` is the full, authoritative log — it is never mutated by
/// compaction; compaction produces a *view* via `CompactionPolicy::view`.
pub struct Session {
    pub id: String,
    pub history: Vec<Turn>,
}

impl Session {
    pub fn new(id: impl Into<String>) -> Self {
        Session { id: id.into(), history: Vec::new() }
    }

    /// Append a turn to the full history log.
    pub fn record(&mut self, turn: Turn) {
        self.history.push(turn);
    }
}

// ---------------------------------------------------------------------------
// HistoryStore
// ---------------------------------------------------------------------------

/// Durable storage seam for conversation histories.
///
/// Stub today (`InMemoryHistoryStore`). The real implementation is an
/// append-only event-sourced log (crash-safe, replay-capable) — a later slice.
pub trait HistoryStore {
    fn load(&self, session: &str) -> Vec<Turn>;
    fn save(&self, session: &str, history: &[Turn]);
}

/// In-process, in-memory history store (no persistence across restarts).
pub struct InMemoryHistoryStore {
    inner: Mutex<HashMap<String, Vec<Turn>>>,
}

impl InMemoryHistoryStore {
    pub fn new() -> Self {
        InMemoryHistoryStore { inner: Mutex::new(HashMap::new()) }
    }
}

impl Default for InMemoryHistoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryStore for InMemoryHistoryStore {
    fn load(&self, session: &str) -> Vec<Turn> {
        self.inner.lock().unwrap().get(session).cloned().unwrap_or_default()
    }

    fn save(&self, session: &str, history: &[Turn]) {
        self.inner.lock().unwrap().insert(session.to_string(), history.to_vec());
    }
}

// ---------------------------------------------------------------------------
// CompactionPolicy — compaction as a pure projection
// ---------------------------------------------------------------------------

/// A compaction policy produces a *view* of the full history log for the
/// model to consume. The full log is **never mutated** — `view` is a pure
/// function. Re-running `view` on the same full log always returns the same
/// result; the full log can always be recovered.
pub trait CompactionPolicy {
    fn view(&self, full: &[Turn]) -> Vec<Turn>;
}

/// Keep the most-recent `keep` turns; when `full.len() > trigger_at`, prepend
/// a synthesized summary-placeholder turn so the model knows history exists.
///
/// Below `trigger_at`, the view equals the full log (identity projection).
pub struct KeepRecent {
    /// Number of recent turns to retain in the compact view.
    pub keep: usize,
    /// Compaction activates only when the log grows past this length.
    pub trigger_at: usize,
}

impl CompactionPolicy for KeepRecent {
    fn view(&self, full: &[Turn]) -> Vec<Turn> {
        if full.len() <= self.trigger_at {
            return full.to_vec();
        }
        // Synthesise a placeholder turn that marks omitted history.
        let omitted = full.len().saturating_sub(self.keep);
        let summary = Turn::new(
            "system",
            format!("[compacted: {} earlier turn(s) omitted]", omitted),
        );
        let mut view = vec![summary];
        let tail = if self.keep >= full.len() { full } else { &full[full.len() - self.keep..] };
        view.extend_from_slice(tail);
        view
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // A session accumulates turns across multiple `record` calls.
    #[test]
    fn session_grows_with_each_record() {
        let mut s = Session::new("s1");
        assert!(s.history.is_empty());
        s.record(Turn::new("user", "hello"));
        s.record(Turn::new("assistant", "hi there"));
        s.record(Turn::new("user", "how are you?"));
        assert!(s.history.len() >= 3, "history must grow with each record");
    }

    // InMemoryHistoryStore round-trips: save then load returns the same turns.
    #[test]
    fn in_memory_store_round_trips() {
        let store = InMemoryHistoryStore::new();
        let turns = vec![
            Turn::new("user", "q1"),
            Turn::new("assistant", "a1"),
            Turn::new("user", "q2"),
        ];
        store.save("sess-a", &turns);
        let loaded = store.load("sess-a");
        assert_eq!(loaded, turns, "load must return exactly what was saved");
    }

    // Loading a session that was never saved returns an empty vec.
    #[test]
    fn in_memory_store_returns_empty_for_unknown_session() {
        let store = InMemoryHistoryStore::new();
        let loaded = store.load("nonexistent");
        assert!(loaded.is_empty());
    }

    // Below trigger_at, view(full) == full (identity projection, no compaction).
    #[test]
    fn keep_recent_below_trigger_is_identity() {
        let policy = KeepRecent { keep: 3, trigger_at: 5 };
        let full: Vec<Turn> = (0..4).map(|i| Turn::new("user", format!("m{i}"))).collect();
        let view = policy.view(&full);
        assert_eq!(view, full, "below trigger_at the view must equal the full log");
    }

    // At exactly trigger_at (not strictly above), still identity.
    #[test]
    fn keep_recent_at_trigger_boundary_is_identity() {
        let policy = KeepRecent { keep: 2, trigger_at: 5 };
        let full: Vec<Turn> = (0..5).map(|i| Turn::new("user", format!("m{i}"))).collect();
        let view = policy.view(&full);
        assert_eq!(view, full);
    }

    // Above trigger_at: view is shorter than full, full is unchanged, and the
    // view ends with the most-recent turns (continuity preserved).
    #[test]
    fn keep_recent_above_trigger_compacts_losslessly() {
        let policy = KeepRecent { keep: 2, trigger_at: 3 };
        let full: Vec<Turn> = (0..6).map(|i| Turn::new("user", format!("m{i}"))).collect();
        let full_before = full.clone();

        let view = policy.view(&full);

        // Lossless: full log is unchanged.
        assert_eq!(full, full_before, "full log must never be mutated");

        // View is shorter than the full log.
        assert!(view.len() < full.len(), "compacted view must be shorter than the full log");

        // The view ends with the most-recent turns (continuity).
        let tail = &full[full.len() - 2..];
        assert_eq!(&view[view.len() - 2..], tail, "view must end with the most-recent turns");
    }

    // Compaction is a pure function of the input (same full log → same view).
    #[test]
    fn keep_recent_is_pure() {
        let policy = KeepRecent { keep: 2, trigger_at: 3 };
        let full: Vec<Turn> = (0..5).map(|i| Turn::new("assistant", format!("r{i}"))).collect();
        let v1 = policy.view(&full);
        let v2 = policy.view(&full);
        assert_eq!(v1, v2, "compaction must be deterministic / pure");
    }

    // Multiple sessions in the same store are isolated.
    #[test]
    fn in_memory_store_isolates_sessions() {
        let store = InMemoryHistoryStore::new();
        let a = vec![Turn::new("user", "session-a")];
        let b = vec![Turn::new("user", "session-b"), Turn::new("assistant", "reply-b")];
        store.save("a", &a);
        store.save("b", &b);
        assert_eq!(store.load("a"), a);
        assert_eq!(store.load("b"), b);
    }
}
