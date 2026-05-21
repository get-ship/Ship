//! Registry dispatcher for `mesh.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::json;

use super::wire::MeshSnapshotResponse;
use super::RegistryError;
use crate::ship_runtime::ApiState;

ship_ops! {
    MESH_OPS;
    {
        name: "mesh.snapshot",
        desc: "Kernel actors + Ship processes + integrations — Mission Control mesh view.",
        perm: "ship.mesh.read",
        args: &[],
        ret: MeshSnapshotResponse,
        run: |_args, state, _files| snapshot(state),
    },
}

/// Build the mesh snapshot: kernel actors merged with projection metadata,
/// Ship system processes from /proc, and known integrations. `mesh.snapshot`
/// is dispatchable but intentionally absent from `schema::OPS`.
fn snapshot(state: &ApiState) -> Result<MeshSnapshotResponse, RegistryError> {
    let value: Result<MeshSnapshotResponse, RegistryError> = {
        {
            // 1. Kernel actor list (mesh truth) merged with projection
            //    metadata where available.
            let kernel_actors: Vec<(String, ship_substrate::events::ActorConfig)> = {
                let kernel = state.kernel().clone();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        kernel.lock().await.list_actors()
                    })
                })
            };
            let projection_rows = ship_substrate::projections::actor::list_actors(
                &ship_substrate::projections::actor::ListActorsFilter {
                    limit: 500,
                    ..Default::default()
                },
            ).unwrap_or_default();
            let projection_by_id: std::collections::HashMap<String, _> =
                projection_rows.into_iter().map(|r| (r.id.clone(), r)).collect();
            let actors: Vec<serde_json::Value> = kernel_actors
                .into_iter()
                .map(|(id, cfg)| {
                    let proj = projection_by_id.get(&id);
                    json!({
                        "actor_id": id,
                        "kind": proj.map(|p| p.kind.clone())
                            .unwrap_or_else(|| {
                                // No projection row: classify from id pattern.
                                if id.starts_with("service.") { "service".into() }
                                else if id.starts_with("agent.") { "agent".into() }
                                else { "unknown".into() }
                            }),
                        "agent_id": proj.and_then(|p| p.agent_id.clone()),
                        "harness": proj.and_then(|p| p.harness.clone()),
                        "transport": proj.and_then(|p| p.transport.clone()),
                        "workspace_id": proj.and_then(|p| p.workspace_id.clone()),
                        "pid": proj.and_then(|p| p.pid),
                        "last_event_at": proj.and_then(|p| p.last_event_at.clone()),
                        "subscribe_namespaces": cfg.subscribe_namespaces,
                    })
                })
                .collect();

            // 2. Ship system processes — daemon + supervisor from the
            //    pidfile/port. These don't always register as kernel
            //    actors but they're the most important "Ship is doing
            //    this on your machine" rows.
            let global_dir = ship_substrate::project::get_global_dir().unwrap_or_default();
            let home = ship_substrate::ShipHome::at(global_dir.clone());
            let daemon_pid: Option<i64> = std::fs::read_to_string(
                home.daemon_file(ship_substrate::home::DaemonFile::Pid),
            )
                .ok()
                .and_then(|s| s.trim().parse().ok());
            let log_path = home.machine_dir().join("logs").join("shipd.log");
            let pid_alive = ship_substrate::projections::actor::pid_alive;
            let mut processes: Vec<serde_json::Value> = Vec::new();

            // Daemon + supervisor — the "Ship is running" anchor.
            if let Some(pid) = daemon_pid {
                processes.push(json!({
                    "name": "shipd",
                    "role": "daemon",
                    "pid": pid,
                    "alive": pid_alive(pid),
                    "log": log_path.display().to_string(),
                    "version": build_info::build_info!().short(),
                }));
                if let Ok(p) = std::fs::read_to_string(format!("/proc/{pid}/status")) {
                    let ppid: Option<i64> = p.lines()
                        .find(|l| l.starts_with("PPid:"))
                        .and_then(|l| l.split_whitespace().nth(1))
                        .and_then(|s| s.parse().ok());
                    if let Some(pp) = ppid
                        && pp > 1 {
                            processes.push(json!({
                                "name": "ship daemon supervisor",
                                "role": "supervisor",
                                "pid": pp,
                                "alive": pid_alive(pp),
                            }));
                        }
                }
            }

            // Walk /proc and pick out harness CLIs + `ship mcp serve`
            // children. These are processes that exist because of Ship
            // (or via Ship's MCP integration). Trust contract: if a
            // ship-spawned `ship` is running on this machine, we show it.
            let interesting_bins = [
                ("claude", "claude"),
                ("codex", "codex"),
                ("opencode", "opencode"),
                ("gemini", "gemini"),
                ("cursor", "cursor"),
            ];
            if let Ok(entries) = std::fs::read_dir("/proc") {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let pid: i64 = match name.to_string_lossy().parse() {
                        Ok(p) => p,
                        _ => continue,
                    };
                    let cmdline = match std::fs::read(format!("/proc/{pid}/cmdline")) {
                        Ok(b) => String::from_utf8_lossy(&b).replace('\0', " ").trim().to_string(),
                        _ => continue,
                    };
                    if cmdline.is_empty() { continue; }

                    // ship mcp serve children (any path).
                    if cmdline.contains("ship mcp serve")
                        || cmdline.contains("/ship mcp serve")
                    {
                        let parent: Option<String> = std::fs::read_to_string(
                            format!("/proc/{pid}/status")
                        )
                            .ok()
                            .and_then(|s| s.lines()
                                .find(|l| l.starts_with("PPid:"))
                                .map(|l| l.to_string())
                            );
                        let ppid: Option<i64> = parent
                            .and_then(|s| s.split_whitespace().nth(1).map(String::from))
                            .and_then(|s| s.parse().ok());
                        let ppid_cmd = ppid.and_then(|p| {
                            std::fs::read(format!("/proc/{p}/cmdline"))
                                .ok()
                                .map(|b| String::from_utf8_lossy(&b)
                                    .replace('\0', " ").trim().to_string())
                                .map(|s| s.split_whitespace().next()
                                    .unwrap_or("")
                                    .rsplit('/')
                                    .next()
                                    .unwrap_or("")
                                    .to_string())
                        });
                        processes.push(json!({
                            "name": "ship mcp serve",
                            "role": "mcp-bridge",
                            "pid": pid,
                            "alive": true,
                            "parent": ppid,
                            "parent_cmd": ppid_cmd,
                        }));
                        continue;
                    }

                    // Harness CLIs (claude, codex, etc.) — the actual
                    // agent binaries running on the user's machine.
                    let bin = cmdline.split_whitespace().next().unwrap_or("");
                    let bin_basename = bin.rsplit('/').next().unwrap_or(bin);
                    for (probe, label) in &interesting_bins {
                        if bin_basename == *probe {
                            // Distinguish ship-spawned vs user-direct:
                            // ship-spawned has --name job-... or is a
                            // child of a `ship` parent process.
                            let role = if cmdline.contains("--name job-") {
                                "agent (ship-spawned)"
                            } else {
                                "agent (user-launched)"
                            };
                            processes.push(json!({
                                "name": *label,
                                "role": role,
                                "pid": pid,
                                "alive": true,
                                "cmdline_preview": cmdline.chars().take(120).collect::<String>(),
                            }));
                            break;
                        }
                    }
                }
            }

            // 3. Integrations — known external bridges Ship can talk
            //    to. Today: Telegram if env vars are set. Will
            //    expand as user-inbox / Discord / Slack land.
            let mut integrations: Vec<serde_json::Value> = Vec::new();
            let tg_token = ship_substrate::env::read(ship_substrate::env::Var::TelegramToken);
            let tg_chat = ship_substrate::env::read(ship_substrate::env::Var::TelegramChatId);
            integrations.push(json!({
                "name": "Telegram (user inbox)",
                "kind": "inbox",
                "active": tg_token.is_some() && tg_chat.is_some(),
                "hint": if tg_token.is_some() && tg_chat.is_some() {
                    "configured"
                } else {
                    "set SHIP_TELEGRAM_TOKEN + SHIP_TELEGRAM_CHAT_ID"
                },
            }));
            // Channel capability is always declared by the MCP server.
            integrations.push(json!({
                "name": "claude/channel",
                "kind": "channel",
                "active": true,
                "hint": "Ship MCP declares experimental.claude/channel capability",
            }));

            let counts = json!({
                "actors": actors.len(),
                "processes": processes.len(),
                "integrations": integrations.iter().filter(|i| i["active"] == true).count(),
            });
            Ok(MeshSnapshotResponse { actors, processes, integrations, counts })
        }
    };
    value
}
