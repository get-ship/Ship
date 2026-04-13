use anyhow::{Result, bail};
use ship_jobs::pipeline::normalize_pipeline_yaml;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Options parsed from CLI flags.
pub struct RunOptions {
    pub pipeline_name: String,
    pub spec_text: Option<String>,
    pub spec_file: Option<PathBuf>,
    pub workspace: Option<String>,
    pub vars: Vec<(String, String)>,
    pub dry_run: bool,
}

pub fn run(opts: RunOptions) -> Result<()> {
    let (pipeline_yaml, pipeline_path) = load_pipeline_yaml(&opts.pipeline_name)?;
    let (normalized, meta) = normalize_pipeline_yaml(&pipeline_yaml)
        .map_err(|e| anyhow::anyhow!("pipeline parse error: {e}"))?;

    let steps = normalized
        .get("steps")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("pipeline has no steps"))?;

    // Merge vars: pipeline defaults < CLI flags
    let mut vars: HashMap<String, String> = meta
        .vars
        .clone()
        .unwrap_or_default();
    for (k, v) in &opts.vars {
        vars.insert(k.clone(), v.clone());
    }

    // Resolve spec content
    let spec_content = resolve_spec(&opts)?;

    // Determine first agent for the job
    let first_agent = steps
        .first()
        .and_then(|s| s.get("agent"))
        .and_then(|a| a.as_str())
        .unwrap_or("default");

    // Interpolate agent name for display
    let first_agent_display = interpolate_simple(first_agent, &vars);

    let pipeline_name = meta.name.as_deref().unwrap_or(&opts.pipeline_name);
    let workspace_name = opts
        .workspace
        .as_deref()
        .unwrap_or(pipeline_name);

    if opts.dry_run {
        print_dry_run(pipeline_name, &pipeline_path, steps, &vars, &spec_content, workspace_name);
        return Ok(());
    }

    // Post to daemon
    let slug = format!("{}-{}", workspace_name, &runtime::gen_ulid()[..8].to_lowercase());
    let branch = format!("job/{slug}");
    let spec_path = ".artifacts/job-spec.md".to_string();

    let body = serde_json::json!({
        "slug": slug,
        "agent": first_agent_display,
        "branch": branch,
        "spec_path": spec_path,
        "pipeline": pipeline_name,
        "spec_content": if spec_content.is_empty() { None } else { Some(&spec_content) },
        "vars": if vars.is_empty() { None } else { Some(&vars) },
    });

    match post_to_daemon("/api/runtime/jobs", &body) {
        Ok(resp) => {
            eprintln!("Pipeline: {pipeline_name}");
            eprintln!("Steps:    {}", steps.len());
            eprintln!("Slug:     {slug}");
            eprintln!("Branch:   {branch}");
            if let Some(ok) = resp.get("ok").and_then(|v| v.as_bool()) {
                if ok {
                    eprintln!("Status:   dispatched");
                } else {
                    let err = resp
                        .get("data")
                        .and_then(|d| d.as_str())
                        .unwrap_or("unknown error");
                    bail!("daemon rejected job: {err}");
                }
            }
        }
        Err(e) => {
            bail!("failed to reach daemon: {e}\nTry: ship daemon start");
        }
    }

    Ok(())
}

/// Load pipeline YAML from .ship/pipelines/<name>.yaml.
fn load_pipeline_yaml(name: &str) -> Result<(String, PathBuf)> {
    let mut dir = std::env::current_dir()?;
    loop {
        let ship_dir = dir.join(".ship");
        if ship_dir.is_dir() {
            let path = ship_dir.join("pipelines").join(format!("{name}.yaml"));
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                return Ok((content, path));
            }
            bail!(
                "pipeline '{name}' not found at {}\nAvailable: {}",
                path.display(),
                list_pipelines(&ship_dir)
            );
        }
        if !dir.pop() {
            bail!("no .ship/ directory found. Run: ship init");
        }
    }
}

/// List available pipeline names for error messages.
fn list_pipelines(ship_dir: &Path) -> String {
    let pipelines_dir = ship_dir.join("pipelines");
    std::fs::read_dir(pipelines_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name.strip_suffix(".yaml").map(str::to_string)
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

/// Resolve spec content from inline text or file.
fn resolve_spec(opts: &RunOptions) -> Result<String> {
    if let Some(ref path) = opts.spec_file {
        return std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("cannot read spec file {}: {e}", path.display()));
    }
    if let Some(ref text) = opts.spec_text {
        return Ok(text.clone());
    }
    Ok(String::new())
}

/// Simple `{{ var }}` interpolation for display purposes.
fn interpolate_simple(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find("{{") {
        result.push_str(&rest[..open]);
        rest = &rest[open + 2..];
        if let Some(close) = rest.find("}}") {
            let var_name = rest[..close].trim();
            match vars.get(var_name) {
                Some(value) => result.push_str(value),
                None => {
                    result.push_str("{{");
                    result.push_str(&rest[..close]);
                    result.push_str("}}");
                }
            }
            rest = &rest[close + 2..];
        } else {
            result.push_str("{{");
        }
    }
    result.push_str(rest);
    result
}

fn print_dry_run(
    name: &str,
    path: &Path,
    steps: &[serde_json::Value],
    vars: &HashMap<String, String>,
    spec: &str,
    workspace: &str,
) {
    println!("=== Dry Run ===");
    println!("Pipeline:  {name}");
    println!("File:      {}", path.display());
    println!("Workspace: {workspace}");
    if !vars.is_empty() {
        println!("Vars:");
        for (k, v) in vars {
            println!("  {k} = {v}");
        }
    }
    println!("Steps ({}):", steps.len());
    for (i, step) in steps.iter().enumerate() {
        let step_type = step
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("agent");
        let label = match step_type {
            "agent" | "gate" => {
                let agent = step
                    .get("agent")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let resolved = interpolate_simple(agent, vars);
                format!("[{step_type}] {resolved}")
            }
            "run" => {
                let cmd = step
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                format!("[run] {cmd}")
            }
            "human" => {
                let goal = step
                    .get("goal")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                format!("[human] {goal}")
            }
            _ => format!("[{step_type}]"),
        };
        let after = step
            .get("after")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let dep = if after.is_empty() {
            String::new()
        } else {
            format!(" (after: {after})")
        };
        println!("  {}. {label}{dep}", i + 1);
    }
    if !spec.is_empty() {
        println!("Spec:");
        // Show first 5 lines
        for line in spec.lines().take(5) {
            println!("  {line}");
        }
        let total = spec.lines().count();
        if total > 5 {
            println!("  ... ({} more lines)", total - 5);
        }
    }
}

/// POST JSON to the daemon API, auto-starting the daemon if needed.
fn post_to_daemon(path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
    let (port, token) = crate::daemon::ensure_daemon()?;

    let url = format!("http://127.0.0.1:{port}{path}");
    let resp: serde_json::Value = ureq::post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        .send_json(body)?
        .body_mut()
        .read_json()?;
    Ok(resp)
}
