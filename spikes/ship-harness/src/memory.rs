//! Memory — conversation persistence, compaction, and working facts.
//!
//! Three layers, each a seam:
//!   - `Memory`         : load/save conversation history by session id. In prod
//!                        this is Ship's event-sourced store; here in-memory.
//!   - `CompactionPolicy`: when history exceeds budget, fold older turns into a
//!                        priority-tiered snapshot (the context-mode pre-compact
//!                        idea) so the agent doesn't lose active files / tasks /
//!                        decisions on overflow.
//!   - `WorkingMemory`  : durable facts the agent curates and recalls across
//!                        sessions (pi's "hindsight"). Exposed to the model as
//!                        remember/recall tools (wiring is a later increment).

use std::collections::HashMap;
use std::sync::Mutex;

use rig::message::Message;

/// Conversation history persistence.
pub trait Memory: Send + Sync {
    fn load(&self, session: &str) -> Vec<Message>;
    fn save(&self, session: &str, history: &[Message]);
}

#[derive(Default)]
pub struct InMemoryMemory {
    by_session: Mutex<HashMap<String, Vec<Message>>>,
}

impl Memory for InMemoryMemory {
    fn load(&self, session: &str) -> Vec<Message> {
        self.by_session
            .lock()
            .unwrap()
            .get(session)
            .cloned()
            .unwrap_or_default()
    }
    fn save(&self, session: &str, history: &[Message]) {
        self.by_session
            .lock()
            .unwrap()
            .insert(session.to_string(), history.to_vec());
    }
}

/// Decides when/how to compact. Real impls summarize older turns via a cheap LLM
/// call into a ≤budget snapshot, preserving active files / tasks / decisions.
pub trait CompactionPolicy: Send + Sync {
    fn should_compact(&self, history: &[Message]) -> bool;
    /// Returns the compacted history. The seam: a real impl calls the model to
    /// summarize; this default keeps the most recent `keep` messages.
    fn compact(&self, history: &[Message]) -> Vec<Message>;
}

/// Keep the most recent N messages; drop the older tail. Placeholder for the
/// LLM-summary snapshot — the shape is what matters for the bones.
pub struct KeepRecent {
    pub keep: usize,
    pub trigger_at: usize,
}

impl CompactionPolicy for KeepRecent {
    fn should_compact(&self, history: &[Message]) -> bool {
        history.len() > self.trigger_at
    }
    fn compact(&self, history: &[Message]) -> Vec<Message> {
        let n = history.len();
        if n <= self.keep {
            return history.to_vec();
        }
        history[n - self.keep..].to_vec()
    }
}

/// Durable facts the agent curates. Project-scoped recall.
#[derive(Default)]
pub struct WorkingMemory {
    facts: Mutex<Vec<(String, String)>>,
}

impl WorkingMemory {
    pub fn note(&self, key: impl Into<String>, value: impl Into<String>) {
        self.facts.lock().unwrap().push((key.into(), value.into()));
    }
    /// Naive substring recall (real impl: FTS5/BM25, same as ContextStore).
    pub fn recall(&self, query: &str) -> Vec<(String, String)> {
        let q = query.to_lowercase();
        self.facts
            .lock()
            .unwrap()
            .iter()
            .filter(|(k, v)| k.to_lowercase().contains(&q) || v.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }
}
