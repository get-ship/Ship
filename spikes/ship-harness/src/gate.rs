//! Tool gate — the runtime side of the spec's `ToolPolicy`, fused with the
//! blast-radius ceiling and the human-approval seam.
//!
//! Every harness-owned tool passes its permit decision through `Gate::permit`
//! (called by `spine::run`). Three checks, in order:
//!   1. policy stance  (allow / ask / deny)  ← per-agent, from AgentSpec
//!   2. approval        (for Ask)            ← human-in-the-loop seam
//!   3. blast ceiling   (tier <= ceiling)    ← session authority
//!
//! Ops (Rail A) are NOT gated here — they're gated in shipd at `command::call`.
//! This gate governs the harness-owned tools (file / shell / script).

use std::sync::Arc;

use crate::spec::{Stance, ToolPolicy};
use crate::spine::{Authority, Blast};

/// Resolves an `Ask` stance. Interactive → prompt the human (returns true/false);
/// headless step → there is no human, so Ask must resolve deterministically.
pub trait Approver: Send + Sync {
    fn approve(&self, tool: &str, tier: Blast) -> bool;
}

/// Headless default: no human present, so `Ask` becomes deny. A pipeline step
/// must not block on approval it can never receive.
pub struct DenyOnAsk;
impl Approver for DenyOnAsk {
    fn approve(&self, _tool: &str, _tier: Blast) -> bool {
        false
    }
}

/// Dev/interactive convenience: auto-approve. (Real interactive approver emits
/// an `ApprovalRequested` event and awaits `UserInput::Approve`.)
pub struct AutoApprove;
impl Approver for AutoApprove {
    fn approve(&self, _tool: &str, _tier: Blast) -> bool {
        true
    }
}

pub struct Gate {
    pub authority: Authority,
    pub policy: ToolPolicy,
    pub approver: Arc<dyn Approver>,
}

impl Gate {
    pub fn new(authority: Authority, policy: ToolPolicy, approver: Arc<dyn Approver>) -> Self {
        Self {
            authority,
            policy,
            approver,
        }
    }

    /// The permit decision for `tool` at `tier`. `Ok(())` = run it.
    pub fn permit(&self, tool: &str, tier: Blast) -> Result<(), String> {
        match self.policy.classify(tool) {
            Stance::Deny => return Err(format!("tool `{tool}` denied by agent policy")),
            Stance::Ask => {
                if !self.approver.approve(tool, tier) {
                    return Err(format!("tool `{tool}` requires approval — not granted"));
                }
            }
            Stance::Allow => {}
        }
        if tier > self.authority.ceiling {
            return Err(format!(
                "tool `{tool}` needs {tier:?} but session ceiling is {:?}",
                self.authority.ceiling
            ));
        }
        Ok(())
    }
}
