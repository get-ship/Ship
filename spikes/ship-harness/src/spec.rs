//! AgentSpec — the programmable configuration of a Ship agent.
//!
//! The Ship agent is programmable by users and is the DEFAULT runner for
//! pipeline steps that involve an LLM. So a spec must be enough to:
//!   - run headless as a pipeline step (no UI), and
//!   - constrain what the agent may do (tool policy + blast ceiling), and
//!   - declare what it returns (output contract, so a pipeline can consume it).
//!
//! In production this is projected from the `.ship/agents/<id>.jsonc` profile
//! (tools_allow / tools_deny / tools_ask, model, rules). Here it's the same
//! shape, hand-built.

use std::collections::HashSet;

use crate::spine::Blast;

/// How a tool is treated for this agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stance {
    Allow,
    /// Requires human approval (interactive) or denial (headless, no approver).
    Ask,
    Deny,
}

/// Per-agent tool policy. Mirrors Ship profile `tools_allow/deny/ask`.
/// Precedence: deny > ask > allow-list. An empty `allow` means "allow all not
/// otherwise denied"; a non-empty `allow` is a strict allowlist.
#[derive(Debug, Clone, Default)]
pub struct ToolPolicy {
    pub allow: HashSet<String>,
    pub deny: HashSet<String>,
    pub ask: HashSet<String>,
}

impl ToolPolicy {
    pub fn classify(&self, tool: &str) -> Stance {
        if self.deny.contains(tool) {
            return Stance::Deny;
        }
        if self.ask.contains(tool) {
            return Stance::Ask;
        }
        if !self.allow.is_empty() && !self.allow.contains(tool) {
            return Stance::Deny; // strict allowlist
        }
        Stance::Allow
    }
}

/// What a run produces — drives how a pipeline step consumes the result.
#[derive(Debug, Clone)]
pub enum OutputContract {
    /// Free text (interactive / summary steps).
    Text,
    /// Structured output validated against a JSON schema (typed pipeline steps).
    Json { schema: serde_json::Value },
}

/// Interactive (streamed to a UI) vs Step (headless, for a pipeline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Interactive,
    Step,
}

/// The full programmable spec. Built from a Ship agent profile.
#[derive(Debug, Clone)]
pub struct AgentSpec {
    pub id: String,
    pub preamble: String,
    pub model: String,
    pub tool_policy: ToolPolicy,
    /// Highest blast radius this agent may reach. C3 is operator-only and an
    /// in-process agent can never hold it.
    pub blast_ceiling: Blast,
    pub max_turns: usize,
    pub output: OutputContract,
}

impl AgentSpec {
    /// A sane default for the base no-bash Ship agent.
    pub fn base(id: impl Into<String>, model: impl Into<String>) -> Self {
        let mut policy = ToolPolicy::default();
        // No sidecar yet → bash is denied for the base agent by policy AND by
        // the host-refuse sandbox. Belt and suspenders.
        policy.deny.insert("bash".into());
        // Edits ask for approval by default (the human-in-the-loop seam).
        policy.ask.insert("edit_file".into());
        Self {
            id: id.into(),
            preamble: String::new(),
            model: model.into(),
            tool_policy: policy,
            blast_ceiling: Blast::C2Elevated,
            max_turns: 20,
            output: OutputContract::Text,
        }
    }
}
