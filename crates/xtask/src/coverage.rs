//! Static coverage report: which ops and events have passing tests?
//!
//! Algorithm:
//! 1. Optionally run `cargo test --workspace --no-fail-fast --message-format=json`,
//!    capturing stdout as JSONL.
//! 2. Parse each line for test events (`type == "test"`, `event == "ok"|"failed"|"ignored"`).
//! 3. Build op surface from `read_policies()` (the proto FDS — single SSOT).
//! 4. Build event surface from the payload registry source file (static parse).
//! 5. For each op/event, match test names by convention; emit red/green markdown.
//!
//! Usage:
//!   cargo xtask coverage                     # run tests + write .artifacts/coverage/report.md
//!   cargo xtask coverage --stdout            # print to stdout instead of file
//!   cargo xtask coverage --report-only       # skip test run, use existing JSON on stdin
//!   cargo xtask coverage --format json       # JSON output (default: markdown)
//!   cargo xtask coverage --filter <service>  # scope to one service prefix

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use ship_ops_rpc::policy::{is_non_domain_service, op_name, read_policies};

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestStatus {
    Passed,
    Failed,
    Ignored,
}

#[derive(Debug, Clone)]
struct TestResult {
    name: String,
    status: TestStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CovStatus {
    Covered,  // at least one passing test matched
    Failing,  // matched tests but all failed/ignored
    Untested, // no matching test found
}

impl CovStatus {
    fn symbol(&self) -> &'static str {
        match self {
            Self::Covered => "✓",
            Self::Failing => "✗",
            Self::Untested => "?",
        }
    }
}

#[derive(Debug, Clone)]
struct OpCov {
    op: String,          // e.g. "agents.create"
    service: String,     // e.g. "AgentsService"
    status: CovStatus,
    matching: Vec<String>,
}

#[derive(Debug, Clone)]
struct EventCov {
    event: String,       // e.g. "actor.created"
    namespace: String,   // first segment, e.g. "actor"
    status: CovStatus,
    matching: Vec<String>,
}

// ── Args ──────────────────────────────────────────────────────────────────────

pub struct CoverageArgs {
    pub report_only: bool,
    pub stdout: bool,
    pub format: OutputFormat,
    pub filter: Option<String>,
}

pub enum OutputFormat {
    Markdown,
    Json,
}

impl CoverageArgs {
    pub fn parse(args: &[&str]) -> Self {
        let mut report_only = false;
        let mut stdout = false;
        let mut format = OutputFormat::Markdown;
        let mut filter = None;
        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "--report-only" => report_only = true,
                "--stdout" => stdout = true,
                "--format" => {
                    i += 1;
                    if i < args.len() && args[i] == "json" {
                        format = OutputFormat::Json;
                    }
                }
                "--filter" => {
                    i += 1;
                    if i < args.len() {
                        filter = Some(args[i].to_string());
                    }
                }
                _ => {}
            }
            i += 1;
        }
        Self { report_only, stdout, format, filter }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(args: &[&str]) -> Result<()> {
    let cfg = CoverageArgs::parse(args);

    let test_results = if cfg.report_only {
        parse_jsonl_from_stdin()?
    } else {
        run_cargo_test()?
    };

    let op_covs = compute_op_coverage(&test_results, cfg.filter.as_deref());
    let event_covs = compute_event_coverage(&test_results, cfg.filter.as_deref());

    let output = match cfg.format {
        OutputFormat::Markdown => render_markdown(&op_covs, &event_covs),
        OutputFormat::Json => render_json(&op_covs, &event_covs)?,
    };

    if cfg.stdout {
        print!("{output}");
    } else {
        let repo_root = crate::cli::repo_root();
        write_report(&repo_root, &output)?;
    }

    Ok(())
}

// ── Test runner ───────────────────────────────────────────────────────────────

fn run_cargo_test() -> Result<Vec<TestResult>> {
    eprintln!("Running cargo test --workspace --no-fail-fast --message-format=json ...");
    eprintln!("(This may take a minute. Use --report-only to skip re-running.)");

    let mut child = Command::new("cargo")
        .args(["test", "--workspace", "--no-fail-fast", "--message-format=json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to spawn cargo test")?;

    let stdout = child.stdout.take().expect("cargo test stdout");
    let results = parse_jsonl(BufReader::new(stdout))?;

    // Wait for completion — ignore exit code (tests may fail).
    let _ = child.wait();
    Ok(results)
}

fn parse_jsonl_from_stdin() -> Result<Vec<TestResult>> {
    eprintln!("Reading test JSON from stdin (--report-only mode) ...");
    parse_jsonl(BufReader::new(std::io::stdin()))
}

fn parse_jsonl(reader: impl BufRead) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();
    for line in reader.lines() {
        let line = line.context("reading test output")?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        // cargo --message-format=json emits many event types; we want `type == "test"`.
        if val.get("type").and_then(|v| v.as_str()) != Some("test") {
            continue;
        }
        let Some(name) = val.get("name").and_then(|v| v.as_str()) else { continue };
        let Some(event) = val.get("event").and_then(|v| v.as_str()) else { continue };
        let status = match event {
            "ok" => TestStatus::Passed,
            "failed" => TestStatus::Failed,
            "ignored" => TestStatus::Ignored,
            _ => continue,
        };
        results.push(TestResult { name: name.to_string(), status });
    }
    Ok(results)
}

// ── Op coverage ───────────────────────────────────────────────────────────────

fn compute_op_coverage(tests: &[TestResult], filter: Option<&str>) -> Vec<OpCov> {
    let policies = read_policies();
    let mut covs = Vec::new();

    for p in &policies {
        if is_non_domain_service(&p.service) {
            continue;
        }
        let op = op_name(&p.service, &p.method);
        if let Some(f) = filter {
            if !op.starts_with(f) && !p.service.to_lowercase().contains(f) {
                continue;
            }
        }
        let matching = match_tests_for_op(&op, tests);
        let status = coverage_status(&matching, tests);
        covs.push(OpCov {
            op,
            service: p.service.clone(),
            status,
            matching: matching.iter().map(|t| t.name.clone()).collect(),
        });
    }

    // Stable sort: service then op name.
    covs.sort_by(|a, b| a.service.cmp(&b.service).then(a.op.cmp(&b.op)));
    covs
}

/// Derive candidate test names for an op like `agents.create`.
/// Matcher tries (in order):
///   1. snake_case op token: `agents_create` — matches any test containing it
///   2. `tests::<domain>::<method>_*` module shape
///   3. The method name alone (snake_case) for short unambiguous methods
fn match_tests_for_op<'a>(op: &str, tests: &'a [TestResult]) -> Vec<&'a TestResult> {
    // op = "agents.create" → token = "agents_create"
    let token = op.replace('.', "_");
    // domain = "agents", method = "create"
    let parts: Vec<&str> = op.splitn(2, '.').collect();
    let domain = parts.first().copied().unwrap_or("");
    let method = parts.get(1).copied().unwrap_or("");

    tests
        .iter()
        .filter(|t| {
            let n = t.name.to_lowercase();
            // Primary: test name contains full token ("agents_create")
            if n.contains(&token) {
                return true;
            }
            // Module-path shape: "tests::agents::create" or "agents::tests::create"
            if n.contains(&format!("{}::{}", domain, method)) {
                return true;
            }
            if n.contains(&format!("{}::tests::{}", domain, method)) {
                return true;
            }
            false
        })
        .collect()
}

// ── Event coverage ────────────────────────────────────────────────────────────

fn compute_event_coverage(tests: &[TestResult], filter: Option<&str>) -> Vec<EventCov> {
    let events = load_event_names();
    let mut covs = Vec::new();

    for event in &events {
        let namespace = event.split('.').next().unwrap_or("").to_string();
        if let Some(f) = filter {
            if !namespace.contains(f) && !event.contains(f) {
                continue;
            }
        }
        let matching = match_tests_for_event(event, tests);
        let status = coverage_status(&matching, tests);
        covs.push(EventCov {
            event: event.clone(),
            namespace,
            status,
            matching: matching.iter().map(|t| t.name.clone()).collect(),
        });
    }

    covs.sort_by(|a, b| a.namespace.cmp(&b.namespace).then(a.event.cmp(&b.event)));
    covs
}

/// Load event names from the payload_registry source file (static parse — no inventory needed).
fn load_event_names() -> Vec<String> {
    // The payload_registry.rs is the single SSOT; parse register_event! macro calls.
    let registry_path =
        "crates/core/ship-substrate/src/events/payload_registry.rs";
    let repo_root = crate::cli::repo_root();
    let path = repo_root.join(registry_path);
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

    // Extract first string arg from: register_event!("event.name", ...)
    let re = regex::Regex::new(r#"register_event!\s*\(\s*"([^"]+)""#)
        .expect("event regex");
    re.captures_iter(&src)
        .map(|c| c[1].to_string())
        .collect()
}

/// Match tests to an event name like `actor.created`.
/// Tries:
///   1. token = "actor_created" — test name contains it
///   2. namespace alone if method is only one segment
fn match_tests_for_event<'a>(event: &str, tests: &'a [TestResult]) -> Vec<&'a TestResult> {
    let token = event.replace(['.', '-'], "_");
    tests
        .iter()
        .filter(|t| t.name.to_lowercase().contains(&token))
        .collect()
}

// ── Coverage status ───────────────────────────────────────────────────────────

fn coverage_status(matching: &[&TestResult], _all: &[TestResult]) -> CovStatus {
    if matching.is_empty() {
        return CovStatus::Untested;
    }
    if matching.iter().any(|t| t.status == TestStatus::Passed) {
        CovStatus::Covered
    } else {
        CovStatus::Failing
    }
}

// ── Markdown renderer ─────────────────────────────────────────────────────────

fn render_markdown(ops: &[OpCov], events: &[EventCov]) -> String {
    let mut out = String::new();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC");
    out.push_str(&format!("# Ship coverage report\n\nGenerated: {now}\n\n"));

    // ── Workspace summary ────────────────────────────────────────────────────
    let op_total = ops.len();
    let op_pass = ops.iter().filter(|o| o.status == CovStatus::Covered).count();
    let op_fail = ops.iter().filter(|o| o.status == CovStatus::Failing).count();
    let op_missing = ops.iter().filter(|o| o.status == CovStatus::Untested).count();
    let op_pct = (op_pass * 100).checked_div(op_total).unwrap_or(0);

    let ev_total = events.len();
    let ev_pass = events.iter().filter(|e| e.status == CovStatus::Covered).count();
    let ev_fail = events.iter().filter(|e| e.status == CovStatus::Failing).count();
    let ev_missing = events.iter().filter(|e| e.status == CovStatus::Untested).count();
    let ev_pct = (ev_pass * 100).checked_div(ev_total).unwrap_or(0);

    out.push_str("## Workspace summary\n\n");
    out.push_str(&format!(
        "- Ops: {op_total} total — {op_pass} ✓ / {op_fail} ✗ / {op_missing} ? — coverage {op_pct}%\n"
    ));
    out.push_str(&format!(
        "- Events: {ev_total} total — {ev_pass} ✓ / {ev_fail} ✗ / {ev_missing} ? — coverage {ev_pct}%\n\n"
    ));

    // ── Per-service breakdown ────────────────────────────────────────────────
    out.push_str("## Per-service breakdown\n\n");
    let mut by_service: BTreeMap<&str, Vec<&OpCov>> = BTreeMap::new();
    for op in ops {
        by_service.entry(&op.service).or_default().push(op);
    }
    for (svc, svc_ops) in &by_service {
        let total = svc_ops.len();
        let pass = svc_ops.iter().filter(|o| o.status == CovStatus::Covered).count();
        out.push_str(&format!("### {svc} ({total} ops, {pass} covered)\n\n"));
        out.push_str("| Op | Status | Tests |\n|---|---|---|\n");
        for op in svc_ops.iter() {
            let test_count = op.matching.len();
            let tests_col = if test_count == 0 {
                "(missing — see gaps)".to_string()
            } else {
                format!("{test_count} test{}", if test_count == 1 { "" } else { "s" })
            };
            out.push_str(&format!(
                "| `{}` | {} | {} |\n",
                op.op,
                op.status.symbol(),
                tests_col,
            ));
        }
        out.push('\n');
    }

    // ── Per-namespace event breakdown ────────────────────────────────────────
    out.push_str("## Per-namespace event breakdown\n\n");
    let mut by_ns: BTreeMap<&str, Vec<&EventCov>> = BTreeMap::new();
    for ev in events {
        by_ns.entry(&ev.namespace).or_default().push(ev);
    }
    for (ns, ns_events) in &by_ns {
        let total = ns_events.len();
        let pass = ns_events.iter().filter(|e| e.status == CovStatus::Covered).count();
        out.push_str(&format!("### {ns}.* ({total} events, {pass} covered)\n\n"));
        out.push_str("| Event | Status | Tests |\n|---|---|---|\n");
        for ev in ns_events.iter() {
            let test_count = ev.matching.len();
            let tests_col = if test_count == 0 {
                "(missing)".to_string()
            } else {
                format!("{test_count} test{}", if test_count == 1 { "" } else { "s" })
            };
            out.push_str(&format!(
                "| `{}` | {} | {} |\n",
                ev.event,
                ev.status.symbol(),
                tests_col,
            ));
        }
        out.push('\n');
    }

    // ── Coverage gaps (top 10, ops first then events) ────────────────────────
    out.push_str("## Coverage gaps (top 10 by priority)\n\n");
    let mut gaps: Vec<String> = Vec::new();

    // Failing tests first, then untested ops
    let failing_ops: Vec<&OpCov> = ops.iter().filter(|o| o.status == CovStatus::Failing).collect();
    for op in &failing_ops {
        gaps.push(format!("**{}** — failing test(s): {}", op.op, op.matching.join(", ")));
    }
    let untested_ops: Vec<&OpCov> = ops.iter().filter(|o| o.status == CovStatus::Untested).collect();
    for op in &untested_ops {
        gaps.push(format!("**{}** — 0 tests (op gap)", op.op));
    }
    // Failing events
    let failing_evs: Vec<&EventCov> = events.iter().filter(|e| e.status == CovStatus::Failing).collect();
    for ev in &failing_evs {
        gaps.push(format!("**{}** — failing test(s): {}", ev.event, ev.matching.join(", ")));
    }
    let untested_evs: Vec<&EventCov> = events.iter().filter(|e| e.status == CovStatus::Untested).collect();
    for ev in &untested_evs {
        gaps.push(format!("**{}** — 0 tests (event gap)", ev.event));
    }

    for (i, gap) in gaps.iter().take(10).enumerate() {
        out.push_str(&format!("{}. {gap}\n", i + 1));
    }
    if gaps.len() > 10 {
        out.push_str(&format!("\n... and {} more gaps.\n", gaps.len() - 10));
    }

    // ── NEEDS-CONVENTION notice ───────────────────────────────────────────────
    out.push_str("\n## NEEDS-CONVENTION\n\n");
    out.push_str("The matcher uses two patterns: `<domain>_<method>` substring and \
`<domain>::<method>` module path. Tests that use neither pattern are not matched. \
Adopt one of these conventions to be counted:\n\n");
    out.push_str("- Name the test fn `<domain>_<method>_<what>` (e.g. `agents_create_returns_id`)\n");
    out.push_str("- Or place it in a module `mod agents { fn create_<what>(...) }`\n");

    out
}

// ── JSON renderer ─────────────────────────────────────────────────────────────

fn render_json(ops: &[OpCov], events: &[EventCov]) -> Result<String> {
    use serde_json::{json, to_string_pretty};

    let ops_json: Vec<_> = ops
        .iter()
        .map(|o| {
            json!({
                "op": o.op,
                "service": o.service,
                "status": match o.status {
                    CovStatus::Covered => "covered",
                    CovStatus::Failing => "failing",
                    CovStatus::Untested => "untested",
                },
                "matching_tests": o.matching,
            })
        })
        .collect();

    let events_json: Vec<_> = events
        .iter()
        .map(|e| {
            json!({
                "event": e.event,
                "namespace": e.namespace,
                "status": match e.status {
                    CovStatus::Covered => "covered",
                    CovStatus::Failing => "failing",
                    CovStatus::Untested => "untested",
                },
                "matching_tests": e.matching,
            })
        })
        .collect();

    let op_total = ops.len();
    let op_pass = ops.iter().filter(|o| o.status == CovStatus::Covered).count();
    let ev_total = events.len();
    let ev_pass = events.iter().filter(|e| e.status == CovStatus::Covered).count();

    let out = json!({
        "summary": {
            "ops": { "total": op_total, "covered": op_pass },
            "events": { "total": ev_total, "covered": ev_pass },
        },
        "ops": ops_json,
        "events": events_json,
    });

    to_string_pretty(&out).context("json serialize")
}

// ── File writer ───────────────────────────────────────────────────────────────

fn write_report(repo_root: &Path, content: &str) -> Result<()> {
    let out_dir = repo_root.join(".artifacts/coverage");
    std::fs::create_dir_all(&out_dir)
        .context("create .artifacts/coverage/")?;
    let out_path = out_dir.join("report.md");
    std::fs::write(&out_path, content)
        .context(format!("write {}", out_path.display()))?;
    eprintln!("Wrote {}", out_path.display());
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tests(names: &[(&str, &str)]) -> Vec<TestResult> {
        names
            .iter()
            .map(|(n, e)| TestResult {
                name: n.to_string(),
                status: match *e {
                    "ok" => TestStatus::Passed,
                    "failed" => TestStatus::Failed,
                    _ => TestStatus::Ignored,
                },
            })
            .collect()
    }

    #[test]
    fn op_token_match() {
        let tests = make_tests(&[("agents_create_returns_id", "ok")]);
        let matched = match_tests_for_op("agents.create", &tests);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn op_module_path_match() {
        let tests = make_tests(&[("agents::create_ok", "ok")]);
        let matched = match_tests_for_op("agents.create", &tests);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn op_no_match() {
        let tests = make_tests(&[("rules_list_all", "ok")]);
        let matched = match_tests_for_op("agents.create", &tests);
        assert!(matched.is_empty());
    }

    #[test]
    fn event_token_match() {
        let tests = make_tests(&[("actor_created_event", "ok")]);
        let matched = match_tests_for_event("actor.created", &tests);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn coverage_status_covered() {
        let tests = make_tests(&[("agents_create_ok", "ok"), ("agents_create_err", "failed")]);
        let refs: Vec<&TestResult> = tests.iter().collect();
        assert_eq!(coverage_status(&refs, &tests), CovStatus::Covered);
    }

    #[test]
    fn coverage_status_failing() {
        let tests = make_tests(&[("agents_create_ok", "failed")]);
        let refs: Vec<&TestResult> = tests.iter().collect();
        assert_eq!(coverage_status(&refs, &tests), CovStatus::Failing);
    }

    #[test]
    fn coverage_status_untested() {
        let tests = make_tests(&[]);
        let refs: Vec<&TestResult> = tests.iter().collect();
        assert_eq!(coverage_status(&refs, &tests), CovStatus::Untested);
    }

    #[test]
    fn jsonl_parser_smoke() {
        let input = r#"
{"type":"test","event":"ok","name":"foo::bar"}
{"type":"suite","event":"started","test_count":1}
{"type":"test","event":"failed","name":"baz::qux"}
{"type":"test","event":"ignored","name":"skip::me"}
"#;
        let results = parse_jsonl(BufReader::new(input.as_bytes())).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].name, "foo::bar");
        assert_eq!(results[0].status, TestStatus::Passed);
        assert_eq!(results[1].status, TestStatus::Failed);
        assert_eq!(results[2].status, TestStatus::Ignored);
    }

    #[test]
    fn load_event_names_non_empty() {
        let names = load_event_names();
        assert!(!names.is_empty(), "event registry must have entries");
        assert!(names.iter().any(|n| n == "actor.created"), "actor.created must be present");
        assert!(names.iter().any(|n| n == "pipeline.run.completed"));
    }
}
