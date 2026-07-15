use stack_model::resolve::ResolvedConfig;

use super::harness::{ContextFile, HarnessDescriptor};

pub(super) fn build_context_content(
    desc: &HarnessDescriptor,
    resolved: &ResolvedConfig,
) -> Option<String> {
    if desc.context_file == ContextFile::None {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();

    // Inline agent prompts are harvested into the same rule pool as shared
    // policy. Keep shared rules, but emit only the selected agent's prompt.
    let active_agent_prompt = resolved
        .active_agent
        .as_deref()
        .map(|active| format!("{active}.md"));
    for rule in &resolved.rules {
        if let Some(active_prompt) = active_agent_prompt.as_deref()
            && resolved
                .agent_profiles
                .iter()
                .any(|profile| format!("{}.md", profile.agent.id) == rule.file_name)
            && rule.file_name != active_prompt
        {
            continue;
        }
        let trimmed = rule.content.trim().to_string();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }

    // Mode notice
    if let Some(mode) = &resolved.active_agent {
        parts.push(format!("<!-- ship: active mode = {} -->", mode));
    }

    if parts.is_empty() {
        return None;
    }

    Some(parts.join("\n\n"))
}
