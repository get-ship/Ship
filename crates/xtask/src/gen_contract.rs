// FILE_CAP_OVERRIDE: domain_catalog() is ~200 lines of static data (no logic).
// Extracting it to a separate file would violate the single-module pattern used
// by all other xtask gen-* subcommands. Generator logic itself is ~450 lines.

//! Generate per-domain consumer contract guides under `.artifacts/contract/`.
//!
//! Each domain page joins two Rust SSOTs:
//!   1. Events — parsed from `crates/core/ship-substrate/src/events/payload_registry.rs`
//!   2. Ops    — parsed from `proto/ship/ops/v1/*.proto` service/rpc declarations
//!
//! Output is ephemeral (gitignored) and regen-able from committed source only.
//! A clean checkout always reproduces output identically.
//!
//! Usage:
//!   cargo xtask gen-contract          — full regen to .artifacts/contract/
//!   cargo xtask gen-contract --check  — diff vs on-disk, exit 1 on drift

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

// ── Output paths ─────────────────────────────────────────────────────────────

const OUTPUT_DIR: &str = ".artifacts/contract";
const PAYLOAD_REGISTRY: &str =
    "crates/core/ship-substrate/src/events/payload_registry.rs";
const PROTO_DIR: &str = "proto/ship/ops/v1";

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct EventDef {
    name: String,
    scope: String,
    retention: String,
    stability: String,
    payload_type: String,
}

#[derive(Debug, Clone)]
struct OpDef {
    service: String,
    rpc: String,
    request: String,
    response: String,
    streaming: bool,
}

// ── Domain metadata ───────────────────────────────────────────────────────────

struct DomainMeta {
    key: &'static str,
    display: &'static str,
    event_prefixes: Vec<&'static str>,
    services: Vec<&'static str>,
    description: &'static str,
    coverage_audit_docs: Vec<&'static str>,
}

/// Maps a domain key → (display name, event namespace prefixes, proto service names,
/// one-line description).
fn domain_catalog() -> Vec<DomainMeta> {
    vec![
        DomainMeta {
            key: "agents",
            display: "Agents",
            event_prefixes: vec!["agent."],
            services: vec!["AgentsService", "AgentService"],
            description: "Agent-profile semantic core: harness registry, four-way provider-settings merger, skill/rule/permission CRUD, and per-skill variable store.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "sessions",
            display: "Sessions",
            event_prefixes: vec!["session.", "hook."],
            services: vec!["SessionsService"],
            description: "Session lifecycle facade: start/end/list/progress analytics rollup over actor activity.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "pipelines",
            display: "Pipelines",
            event_prefixes: vec!["pipeline."],
            services: vec!["PipelinesService"],
            description: "Pipeline runtime (orchestrator): dispatch atom, DAG advancement, fan-out/recover/replay, and per-step monitoring.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "runtime",
            display: "Runtime (actors)",
            event_prefixes: vec!["actor.", "process.", "executor.", "supervisor.", "kernel.actor.", "mesh."],
            services: vec!["RuntimeService"],
            description: "Actor lifecycle supervision, process management, mesh peer-discovery, and kernel snapshot primitives.",
            coverage_audit_docs: vec!["docs/ship-kernel/SKILL.md", ".artifacts/specs/module-audits/L0-substrate/ship-kernel.md", ".artifacts/specs/module-audits/L0-substrate/ship-supervisor.md"],
        },
        DomainMeta {
            key: "registry",
            display: "Registry",
            event_prefixes: vec![],
            services: vec!["RegistryService"],
            description: "Content-addressed package store: dependency resolution, fetch/cache/verify, lock, composition overlay, and bundle attestation.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "collections",
            display: "Collections",
            event_prefixes: vec!["collection."],
            services: vec!["CollectionsService"],
            description: "Structured data primitive: per-collection SQLite stores with schema validation, item CRUD, and event emission.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "artifacts",
            display: "Artifacts",
            event_prefixes: vec!["artifact."],
            services: vec!["ArtifactsService"],
            description: "Blob storage primitive: content-addressed local filesystem backend with GC, metadata projection, and studio integration.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "auth",
            display: "Auth",
            event_prefixes: vec!["auth.session.", "account.", "daemon.owner.", "permissions."],
            services: vec!["AccountService", "PermissionsService"],
            description: "Authentication and authorization: session tokens, peer-credential resolution, capability grants, and permission CRUD.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "skills",
            display: "Skills",
            event_prefixes: vec!["skill."],
            services: vec!["SkillsService"],
            description: "Skill CRUD and variable store: install, list, get, set/get/delete vars, and effective-skill resolution.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "rules",
            display: "Rules",
            event_prefixes: vec!["rule."],
            services: vec!["RulesService"],
            description: "Rule CRUD: create, put, delete rules scoped to project or user.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "mcp",
            display: "MCP",
            event_prefixes: vec![],
            services: vec!["McpService"],
            description: "MCP server configuration: CRUD for per-scope MCP server entries consumed by the compiler.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "sync",
            display: "Sync",
            event_prefixes: vec!["sync."],
            services: vec![],
            description: "Cloud replication client: push event-log slices to the Ship Cloud gateway.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "config",
            display: "Config",
            event_prefixes: vec!["config."],
            services: vec!["SettingsService"],
            description: "Configuration watcher: file-change events for .ship/ directory mutations.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "git",
            display: "Git",
            event_prefixes: vec![],
            services: vec!["GitService"],
            description: "Git operations: status, log, diff, branches, stage/unstage, commit, push, pull, checkout.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "view_authoring",
            display: "View Authoring",
            event_prefixes: vec!["view."],
            services: vec!["DiscoveryService", "CreateService"],
            description: "View compile/edit/snapshot primitives: source read/write, compile-check, snapshot, sibling discovery.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "hooks",
            display: "Hooks",
            event_prefixes: vec!["hook."],
            services: vec![],
            description: "Hook trigger engine: lifecycle hooks (session start/end, tool use, turn) emitted by the harness adapters.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "triggers",
            display: "Triggers",
            event_prefixes: vec![],
            services: vec![],
            description: "Platform trigger engine: event-driven action dispatch keyed on event namespace and condition predicates.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "analytics",
            display: "Analytics",
            event_prefixes: vec![],
            services: vec!["AnalyticsService"],
            description: "Session + tool-use analytics: project, query, and stream session/turn/tool-use records from the analytics projection.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "projects",
            display: "Projects (spaces)",
            event_prefixes: vec!["project.", "space.setting."],
            services: vec!["SpaceService"],
            description: "Space/project registry: register, switch, archive spaces; space-setting change events.",
            coverage_audit_docs: vec![],
        },
        DomainMeta {
            key: "daemon",
            display: "Daemon",
            event_prefixes: vec!["daemon."],
            services: vec!["OpsService"],
            description: "Daemon lifecycle: startup/shutdown/crash events and the ops meta-service.",
            coverage_audit_docs: vec![".artifacts/specs/module-audits/apps/shipd.md"],
        },
        DomainMeta {
            key: "studio",
            display: "Studio",
            event_prefixes: vec!["studio.", "human."],
            services: vec!["ChatService", "DashboardService", "FeedbackService", "AnnotationService"],
            description: "Studio UI primitives: chat turns, agent replies, dashboard state, feedback, and annotation events.",
            coverage_audit_docs: vec![".artifacts/specs/module-audits/apps/shipd.md", ".artifacts/specs/module-audits/L1-platform/sessions.md", ".artifacts/specs/module-audits/L0-substrate/ship-substrate.md"],
        },
        DomainMeta {
            key: "compiler",
            display: "Compiler",
            event_prefixes: vec!["compile."],
            services: vec![],
            description: "Agent-config compiler: resolves provider settings, capability sets, and emits compile.completed on success.",
            coverage_audit_docs: vec![],
        },
    ]
}

// ── Parsing ───────────────────────────────────────────────────────────────────

/// Parse all event registrations from payload_registry.rs.
/// Extracts: register_event!("name", Scope, Retention, Stability, "Type")
fn parse_events(repo_root: &Path) -> Result<Vec<EventDef>> {
    let src = std::fs::read_to_string(repo_root.join(PAYLOAD_REGISTRY))
        .context("read payload_registry.rs")?;

    let mut events = Vec::new();
    for line in src.lines() {
        let line = line.trim();
        if !line.starts_with("register_event!(") {
            continue;
        }
        // register_event!("name", EventScope::X, Retention::Y, Stability::Z, "Type")
        let inner = line
            .trim_start_matches("register_event!(")
            .trim_end_matches(");");
        let parts: Vec<&str> = inner.splitn(5, ',').collect();
        if parts.len() < 5 {
            continue;
        }
        let name = parts[0].trim().trim_matches('"').to_string();
        let scope = parts[1].trim().replace("EventScope::", "");
        let retention = parts[2].trim().replace("Retention::", "");
        let stability = parts[3].trim().replace("Stability::", "");
        let payload_type = parts[4].trim().trim_matches('"').to_string();
        events.push(EventDef { name, scope, retention, stability, payload_type });
    }
    events.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(events)
}

/// Parse proto service RPCs from all *.proto files in proto_dir.
/// Returns vec of OpDef with service name and rpc name.
fn parse_ops(repo_root: &Path) -> Result<Vec<OpDef>> {
    let proto_dir = repo_root.join(PROTO_DIR);
    let mut ops = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&proto_dir)
        .context("read proto dir")?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|x| x.to_str()) == Some("proto")
        })
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let content = std::fs::read_to_string(entry.path())
            .context(format!("read {}", entry.path().display()))?;
        let mut current_service: Option<String> = None;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("service ") && trimmed.ends_with('{') {
                current_service = Some(
                    trimmed
                        .trim_start_matches("service ")
                        .trim_end_matches(" {")
                        .to_string(),
                );
                continue;
            }

            if trimmed == "}" {
                // Could be closing service or message — reset service context
                // only if we're tracking one (simple heuristic: dedent == close)
                if !line.starts_with("  ") && current_service.is_some() {
                    current_service = None;
                }
                continue;
            }

            if let Some(svc) = &current_service {
                // rpc Foo(Bar) returns (Baz) {
                if trimmed.starts_with("rpc ") {
                    let after_rpc = &trimmed[4..];
                    let rpc_name = after_rpc
                        .split('(')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if rpc_name.is_empty() {
                        continue;
                    }

                    // Extract request type
                    let request = after_rpc
                        .split('(')
                        .nth(1)
                        .and_then(|s| s.split(')').next())
                        .unwrap_or("?")
                        .replace("stream ", "")
                        .trim()
                        .to_string();

                    // Extract response type and streaming flag
                    let returns_part = trimmed
                        .split("returns")
                        .nth(1)
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let streaming = returns_part.contains("stream ");
                    let response = returns_part
                        .trim_start_matches('(')
                        .trim_start_matches("stream ")
                        .split(')')
                        .next()
                        .unwrap_or("?")
                        .trim()
                        .to_string();

                    ops.push(OpDef {
                        service: svc.clone(),
                        rpc: rpc_name,
                        request,
                        response,
                        streaming,
                    });
                }
            }
        }
    }

    Ok(ops)
}

// ── Page rendering ────────────────────────────────────────────────────────────

fn render_domain_page(
    domain: &DomainMeta,
    events: &[EventDef],
    ops: &[OpDef],
    date: &str,
) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!(
        "# {} — consumer contract\n\n",
        domain.display
    ));
    out.push_str(&format!(
        "> Auto-generated by `cargo xtask gen-contract`. Do not hand-edit; changes are overwritten. Last regenerated: {}.\n\n",
        date
    ));

    // Description
    out.push_str(&format!("{}\n\n", domain.description));

    // Events emitted
    out.push_str("## Events emitted\n\n");
    let domain_events: Vec<&EventDef> = events
        .iter()
        .filter(|e| domain.event_prefixes.iter().any(|p| e.name.starts_with(p)))
        .collect();

    if domain_events.is_empty() {
        out.push_str("_No events registered for this domain._\n\n");
    } else {
        out.push_str("| Event type | Scope | Retention | Stability | Payload |\n");
        out.push_str("|---|---|---|---|---|\n");
        for e in &domain_events {
            out.push_str(&format!(
                "| `{}` | {} | {} | {} | `{}` |\n",
                e.name, e.scope, e.retention, e.stability, e.payload_type
            ));
        }
        out.push('\n');
        out.push_str("**Subscribe**: Space events sync across devices in your space. Account events sync to cloud. Device events stay local to this machine.\n\n");
    }

    // Ops exposed
    out.push_str("## Ops exposed\n\n");
    let domain_ops: Vec<&OpDef> = ops
        .iter()
        .filter(|o| domain.services.iter().any(|s| o.service == *s))
        .filter(|o| !o.rpc.ends_with("Live") && !o.streaming)
        .collect();
    let domain_ops_live: Vec<&OpDef> = ops
        .iter()
        .filter(|o| domain.services.iter().any(|s| o.service == *s))
        .filter(|o| o.rpc.ends_with("Live") || o.streaming)
        .collect();

    if domain_ops.is_empty() && domain_ops_live.is_empty() {
        out.push_str("_No Connect-RPC ops registered for this domain._\n\n");
    } else {
        out.push_str("| Op | Service | Request | Response |\n");
        out.push_str("|---|---|---|---|\n");
        for o in &domain_ops {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` |\n",
                o.rpc, o.service, o.request, o.response
            ));
        }
        if !domain_ops_live.is_empty() {
            out.push_str("\n**Streaming variants** (Connect server-side streaming):\n\n");
            out.push_str("| Op | Service | Request | Response |\n");
            out.push_str("|---|---|---|---|\n");
            for o in &domain_ops_live {
                out.push_str(&format!(
                    "| `{}` | `{}` | `{}` | `stream {}` |\n",
                    o.rpc, o.service, o.request, o.response
                ));
            }
        }
        out.push('\n');
        out.push_str("**Wire**: Connect-RPC. See `proto/ship/ops/v1/` for message definitions.\n\n");
    }

    // Coverage / audit docs
    if !domain.coverage_audit_docs.is_empty() {
        out.push_str("## Coverage\n\n");
        out.push_str("This domain is covered by the following audit documents:\n\n");
        for doc in &domain.coverage_audit_docs {
            out.push_str(&format!("- `{}`\n", doc));
        }
        out.push('\n');
    }

    // Notes
    out.push_str("## Notes\n\n");
    out.push_str("- **Events SSOT**: `crates/core/ship-substrate/src/events/payload_registry.rs`\n");
    out.push_str("- **Ops SSOT**: `proto/ship/ops/v1/*.proto`\n");
    out.push_str("- **Events proto**: `proto/ship/ops/v1/events.gen.proto`\n");
    out.push_str("- **Events TS**: `packages/cloud-types/src/events.ts`\n");

    out
}

// ── Index page ────────────────────────────────────────────────────────────────

fn render_index(domains: &[DomainMeta], event_counts: &HashMap<&str, usize>, op_counts: &HashMap<&str, usize>, date: &str) -> String {
    let mut out = String::new();
    out.push_str("# Ship Platform — consumer contract index\n\n");
    out.push_str(&format!(
        "> Auto-generated by `cargo xtask gen-contract`. Do not hand-edit. Last regenerated: {}.\n\n",
        date
    ));
    out.push_str("One page per L1 platform domain. Each page answers: *what does this domain expose and how do I consume it?*\n\n");
    out.push_str("| Domain | Events | Ops | Page |\n");
    out.push_str("|---|---|---|---|\n");
    for d in domains {
        let events = event_counts.get(d.key).copied().unwrap_or(0);
        let ops = op_counts.get(d.key).copied().unwrap_or(0);
        out.push_str(&format!(
            "| {} | {} | {} | [{}.md]({}.md) |\n",
            d.display, events, ops, d.key, d.key
        ));
    }
    out.push('\n');
    out.push_str("## Data sources\n\n");
    out.push_str("- **Events**: `crates/core/ship-substrate/src/events/payload_registry.rs` (Rust SSOT)\n");
    out.push_str("- **Ops**: `proto/ship/ops/v1/*.proto` (Connect-RPC service definitions, Rust SSOT)\n");
    out.push_str("\nBoth sources are committed Rust/proto files. A clean checkout always reproduces this output.\n");
    out
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(repo_root: &Path, check: bool) -> Result<()> {
    let date = chrono_date();

    // Parse Rust SSOTs only
    let events = parse_events(repo_root).context("parse events")?;
    let ops = parse_ops(repo_root).context("parse ops")?;
    let domains = domain_catalog();

    eprintln!(
        "gen-contract: {} events, {} ops, {} domains",
        events.len(), ops.len(), domains.len()
    );

    let contract_dir = repo_root.join(OUTPUT_DIR);
    if !check {
        std::fs::create_dir_all(&contract_dir).context("create .artifacts/contract/")?;
    }

    let mut event_counts: HashMap<&str, usize> = HashMap::new();
    let mut op_counts: HashMap<&str, usize> = HashMap::new();
    let mut total_events_shown: usize = 0;
    let mut stale = false;

    // Generate per-domain pages
    for d in &domains {
        let page = render_domain_page(d, &events, &ops, &date);

        let domain_events: usize = events.iter()
            .filter(|e| d.event_prefixes.iter().any(|p| e.name.starts_with(p)))
            .count();
        let domain_ops: usize = ops.iter()
            .filter(|o| d.services.iter().any(|s| o.service == *s))
            .count();

        event_counts.insert(d.key, domain_events);
        op_counts.insert(d.key, domain_ops);
        total_events_shown += domain_events;

        let out_path = contract_dir.join(format!("{}.md", d.key));

        if check {
            let existing = std::fs::read_to_string(&out_path).unwrap_or_default();
            let norm = |s: &str| -> String {
                s.lines()
                    .map(|l| if l.contains("Last regenerated:") { "REGEN_DATE" } else { l })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            if norm(&existing) != norm(&page) {
                eprintln!("  STALE: {}", out_path.display());
                stale = true;
            }
        } else {
            std::fs::write(&out_path, &page)
                .context(format!("write {}", out_path.display()))?;
            eprintln!(
                "  wrote {}.md ({} events, {} ops)",
                d.key, domain_events, domain_ops
            );
        }
    }

    // Generate index
    let index = render_index(&domains, &event_counts, &op_counts, &date);
    let index_path = contract_dir.join("README.md");

    if check {
        let existing = std::fs::read_to_string(&index_path).unwrap_or_default();
        let norm = |s: &str| -> String {
            s.lines()
                .map(|l| if l.contains("Last regenerated:") { "REGEN_DATE" } else { l })
                .collect::<Vec<_>>()
                .join("\n")
        };
        if norm(&existing) != norm(&index) {
            eprintln!("  STALE: {}", index_path.display());
            stale = true;
        }
    } else {
        std::fs::write(&index_path, &index)
            .context(format!("write {}", index_path.display()))?;
        eprintln!("  wrote README.md (index)");
    }

    if check {
        if stale {
            eprintln!("Contract docs are out of date. Run `cargo xtask gen-contract` and commit.");
            std::process::exit(1);
        } else {
            eprintln!("gen-contract: all files up to date.");
        }
    } else {
        eprintln!(
            "gen-contract done: {} domains, {} total events shown",
            domains.len(), total_events_shown
        );
    }

    Ok(())
}

fn chrono_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let (y, m, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
