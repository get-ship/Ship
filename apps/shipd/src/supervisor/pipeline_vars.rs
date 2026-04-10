//! Pipeline variable interpolation — resolves `{{ var }}` references in step fields.
//!
//! Resolution order: job vars (highest priority) → step vars → Err (unresolved).

use runtime::events::job::PipelineStep;
use std::collections::HashMap;

/// Replace all `{{ var }}` references in `template` using `vars`.
/// Returns `Err(var_name)` for the first unresolved reference.
fn interpolate(template: &str, vars: &HashMap<String, String>) -> Result<String, String> {
    let mut result = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find("{{") {
        result.push_str(&rest[..open]);
        rest = &rest[open + 2..];
        let close = rest.find("}}").ok_or_else(|| "malformed template".to_string())?;
        let var_name = rest[..close].trim();
        match vars.get(var_name) {
            Some(value) => result.push_str(value),
            None => return Err(var_name.to_string()),
        }
        rest = &rest[close + 2..];
    }
    result.push_str(rest);
    Ok(result)
}

/// Interpolate all template fields in `step`.
///
/// `job_vars` take priority over `step_vars` for the same key.
/// Returns `Err(missing_var_name)` if any `{{ var }}` reference is unresolved.
pub fn interpolate_step(
    step: &PipelineStep,
    job_vars: &HashMap<String, String>,
    step_vars: &HashMap<String, String>,
) -> Result<PipelineStep, String> {
    let mut merged: HashMap<String, String> = step_vars.clone();
    for (k, v) in job_vars {
        merged.insert(k.clone(), v.clone());
    }

    let interp = |s: &str| interpolate(s, &merged);

    Ok(PipelineStep {
        id: step.id.clone(),
        step_type: step.step_type.clone(),
        agent: step.agent.as_deref().map(interp).transpose()?,
        goal: step.goal.as_deref().map(interp).transpose()?,
        after: step.after.clone(),
        model: step.model.as_deref().map(interp).transpose()?,
        provider: step.provider.clone(),
        command: step.command.as_deref().map(interp).transpose()?,
        max_retries: step.max_retries,
    })
}
