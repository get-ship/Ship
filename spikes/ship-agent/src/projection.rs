//! Context projection (S16.e) — the model's context is a *view* of the journal,
//! not its accumulator.
//!
//! A typical agent appends to a flat message array forever, then truncates
//! destructively. Here the **journal is the lossless truth** and the context the
//! model sees is a **disposable, re-derivable projection** of it. Compaction
//! stops being destruction and becomes *rendering*: you can project the same
//! journal any number of times, at any budget, with any strategy, and never lose
//! the underlying detail (re-project larger to bring it back).
//!
//! The runtime owns projection (swappable strategies are experimental settings);
//! the agent author never sees the journal, only the rendered context. Sub-runs
//! get their own journals — so orchestrating via sub-agents compartmentalizes
//! context for free (the parent sees a child's result, not its 50 inner steps).

use serde_json::{json, Value};

use crate::engine::{Journal, JournalEntry, OpRecord};

/// One rendered item the model would see — a view of a journal slice.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextItem {
    /// A journaled op rendered verbatim: the op and its (logged) result.
    Op { op: String, result: Value },
    /// `count` earlier ops collapsed away. The detail still lives in the journal;
    /// re-project at a larger budget to bring it back.
    Elided { count: usize },
}

/// Render one journal entry as a context item (a suspended op shows as pending).
fn entry_item(e: &JournalEntry) -> ContextItem {
    let result = match &e.record {
        OpRecord::Completed(v) => v.clone(),
        OpRecord::Suspended { reason } => json!({ "awaiting": reason }),
    };
    ContextItem::Op { op: e.op.clone(), result }
}

/// Renders the model's context from the journal. The journal is NEVER mutated —
/// the projection is a disposable view. Swap the impl to change the strategy.
pub trait ContextProjector: Send + Sync {
    fn project(&self, journal: &Journal) -> Vec<ContextItem>;
}

/// Everything, verbatim — the lossless render (the baseline; bounded only by the
/// journal length).
pub struct Verbatim;

impl ContextProjector for Verbatim {
    fn project(&self, journal: &Journal) -> Vec<ContextItem> {
        journal.entries().iter().map(entry_item).collect()
    }
}

/// Keep the most recent `keep` ops verbatim; collapse everything older into one
/// `Elided` marker. The classic budget strategy — but non-destructive: the
/// elided ops are still in the journal.
pub struct KeepRecent {
    pub keep: usize,
}

impl ContextProjector for KeepRecent {
    fn project(&self, journal: &Journal) -> Vec<ContextItem> {
        let n = journal.entries().len();
        if n <= self.keep {
            return journal.entries().iter().map(entry_item).collect();
        }
        let elided = n - self.keep;
        let mut out = Vec::with_capacity(self.keep + 1);
        out.push(ContextItem::Elided { count: elided });
        out.extend(journal.entries().iter().skip(elided).map(entry_item));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{execute, ExecCtx, EngineError, Program, RunStatus};
    use crate::auth::AuthContext;
    use crate::gateway::InProcessGateway;
    use async_trait::async_trait;
    use std::sync::Arc;

    fn agent(grants: &[&str]) -> AuthContext {
        AuthContext::in_process("t", grants.iter().map(|s| s.to_string()).collect())
    }

    /// A program that journals `n` reads, to build a journal to project from.
    struct NReads(usize);
    #[async_trait]
    impl Program for NReads {
        async fn run(&self, ctx: &mut ExecCtx) -> Result<Value, EngineError> {
            for _ in 0..self.0 {
                ctx.op("collections.read", json!({})).await?;
            }
            Ok(json!("done"))
        }
    }

    async fn journal_of(n: usize) -> Journal {
        let gw = Arc::new(InProcessGateway::default());
        let (status, journal) = execute(Journal::new(), gw, agent(&["collections"]), &NReads(n)).await;
        assert!(matches!(status, RunStatus::Completed(_)));
        journal
    }

    // Verbatim renders every journaled op.
    #[tokio::test]
    async fn verbatim_renders_every_op() {
        let journal = journal_of(3).await;
        let ctx = Verbatim.project(&journal);
        assert_eq!(ctx.len(), 3);
        assert!(ctx.iter().all(|i| matches!(i, ContextItem::Op { .. })));
    }

    // KeepRecent compacts to budget — but the journal is LOSSLESS, so the same
    // journal re-projects at a larger budget without ever having lost detail.
    #[tokio::test]
    async fn keep_recent_compacts_but_journal_is_lossless() {
        let journal = journal_of(5).await;

        // Budget 2: one Elided marker + the 2 most-recent ops.
        let tight = KeepRecent { keep: 2 }.project(&journal);
        assert_eq!(tight.len(), 3, "1 elided marker + 2 recent ops");
        assert_eq!(tight[0], ContextItem::Elided { count: 3 });
        assert!(matches!(tight[1], ContextItem::Op { .. }));

        // The journal is untouched — re-project the SAME journal larger and the
        // elided detail comes back. Compaction is rendering, not destruction.
        let wide = KeepRecent { keep: 4 }.project(&journal);
        assert_eq!(wide.len(), 5, "1 elided + 4 recent — detail recovered from the same journal");
        assert_eq!(wide[0], ContextItem::Elided { count: 1 });
        assert_eq!(journal.entries().len(), 5, "the journal never lost anything");
    }

    // The projector is a trait object — strategies are swappable at runtime.
    #[tokio::test]
    async fn projector_is_swappable() {
        let journal = journal_of(4).await;
        let strategies: Vec<Box<dyn ContextProjector>> =
            vec![Box::new(Verbatim), Box::new(KeepRecent { keep: 1 })];
        let lens: Vec<usize> = strategies.iter().map(|s| s.project(&journal).len()).collect();
        assert_eq!(lens, vec![4, 2], "same journal, different views (4 verbatim vs 1 elided + 1 recent)");
    }
}
