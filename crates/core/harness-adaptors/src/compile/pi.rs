//! Pi compiler target (pi.dev).
//!
//! Pi is config-light by design: it reads `AGENTS.md` for context and
//! `.agents/skills/<id>/SKILL.md` for skills — both universal outputs Ship
//! already emits via the shared context + skills passes. It has no built-in
//! MCP, no native subagent file format, and no compiled settings file
//! (model/provider/tools are CLI flags + env; tools come from TS extensions;
//! `settings.json`/extensions are user-authored). So this target reuses the
//! universal outputs and emits nothing harness-specific — onboarding Pi is a
//! descriptor entry plus these two no-ops. Source: pi.dev/docs.

use stack_model::types::AgentProfile;

/// Pi has no native subagent file format (`agents_dir = None`), so
/// `compile_agent_profiles` never invokes this; present only to satisfy the
/// descriptor's fn pointer.
pub(super) fn compile_pi_agent(_profile: &AgentProfile) -> String {
    String::new()
}

/// Pi consumes AGENTS.md + `.agents/skills` (emitted by the shared context
/// and skills passes); there are no Pi-specific settings/policy/hook files.
pub(super) fn build_settings(
    _desc: &super::harness::HarnessDescriptor,
    _resolved: &stack_model::resolve::ResolvedConfig,
    _out: &mut super::CompileOutput,
) {
}
