//! Pipeline variable interpolation — resolves `{{ var }}` references in step fields.
//!
//! Resolution order: job vars (highest priority) → step vars → Err (unresolved).
//! Pure logic — no fs / process / network. Same input + var map yields the
//! same output across the daemon, the cloud worker, and the browser embed.

use std::collections::HashMap;

use crate::types::PipelineStep;

/// Replace all `{{ var }}` references in `template` using `vars`.
/// Returns `Err(var_name)` for the first unresolved reference.
pub fn interpolate(template: &str, vars: &HashMap<String, String>) -> Result<String, String> {
    let mut result = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find("{{") {
        result.push_str(&rest[..open]);
        rest = &rest[open + 2..];
        let close = rest
            .find("}}")
            .ok_or_else(|| "malformed template".to_string())?;
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
        harness: step.harness.as_deref().map(interp).transpose()?,
        command: step.command.as_deref().map(interp).transpose()?,
        max_retries: step.max_retries,
        gate: step.gate.clone(),
        produces: step.produces.clone(),
        consumes: step.consumes.clone(),
        promotes: step.promotes.clone(),
        fanout: step.fanout.clone(),
        // invocation is a closed-set enum string; no interpolation.
        invocation: step.invocation.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::StepType;

    fn step_with_harness(template: &str) -> PipelineStep {
        PipelineStep {
            id: "implement".into(),
            step_type: StepType::Agent,
            agent: Some("rust-lane".into()),
            goal: None,
            after: vec![],
            model: None,
            harness: Some(template.into()),
            command: None,
            max_retries: None,
            gate: None,
            produces: vec![],
            consumes: vec![],
            promotes: vec![],
            fanout: None,
            invocation: None,
        }
    }

    #[test]
    fn harness_is_interpolated_from_job_vars() {
        let step = step_with_harness("{{ harness }}");
        let vars: HashMap<String, String> =
            [("harness".into(), "codex".into())].into_iter().collect();
        let out = interpolate_step(&step, &vars, &Default::default()).unwrap();
        assert_eq!(out.harness.as_deref(), Some("codex"));
    }

    #[test]
    fn harness_missing_var_errors() {
        let step = step_with_harness("{{ harness }}");
        let err = interpolate_step(&step, &Default::default(), &Default::default()).unwrap_err();
        assert_eq!(err, "harness");
    }

    #[test]
    fn harness_literal_passes_through() {
        let step = step_with_harness("claude");
        let out = interpolate_step(&step, &Default::default(), &Default::default()).unwrap();
        assert_eq!(out.harness.as_deref(), Some("claude"));
    }

    #[test]
    fn job_vars_override_step_vars_for_same_key() {
        let step = step_with_harness("{{ harness }}");
        let job: HashMap<String, String> =
            [("harness".into(), "claude".into())].into_iter().collect();
        let step_vars: HashMap<String, String> =
            [("harness".into(), "codex".into())].into_iter().collect();
        let out = interpolate_step(&step, &job, &step_vars).unwrap();
        assert_eq!(out.harness.as_deref(), Some("claude"));
    }

    #[test]
    fn malformed_template_errors() {
        let vars = HashMap::new();
        assert!(interpolate("{{ unclosed", &vars).is_err());
    }

    #[test]
    fn pure_string_no_braces_passes_through() {
        let vars = HashMap::new();
        assert_eq!(interpolate("hello world", &vars).unwrap(), "hello world");
    }
}
