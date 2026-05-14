---
group: Runtime
order: 1
title: Runtime Overview
description: Ship daemon architecture, startup sequence, and API surface.
audience: internal
---

# Runtime Overview

The Ship daemon (`shipd`) is a long-running Axum HTTP server that provides the event kernel, service mesh, supervisor, and PTY bridge. It listens on `127.0.0.1:9315` by default and serves both MCP (Streamable HTTP) and REST endpoints.

## Startup sequence

`shipd::run_network(host, port, dev, workspace_dir, daemon_mode)` performs these steps in order:

1. **Initialize KernelRouter** — initializes the event store. Events live in `platform.db` at `~/.ship/machine/db/platform.db`; there is no separate kernel database.
2. **Spawn MeshService** — registers a `service.mesh` actor subscribed to `mesh.*` events. Returns a `SharedMeshRegistry` for REST read access.
3. **Subscribe job events** — spawns `service.job-dispatch` actor to handle `job.*` lifecycle (create, update, complete, merge).
4. **Spawn human gateway** — if `SHIP_TELEGRAM_TOKEN` and `SHIP_TELEGRAM_CHAT_ID` are set, spawns a `service.human-gateway` actor.
5. **Write PID and port files** — under `~/.ship/machine/daemon/`.
6. **Build Axum router** — mounts MCP service at `/mcp`, REST APIs under `/api/v1`.
7. **Listen** — binds TCP and serves with graceful shutdown on SIGINT/SIGTERM.

On shutdown, PID and port files are removed.

## API surface

### MCP endpoint (`/mcp`)

Streamable HTTP via the `rmcp` library. Each HTTP session gets its own `NetworkServer` instance sharing a single `KernelRouter`. MCP tools: `register_mesh_agent`, `send_mesh_message`, `broadcast_mesh_message`, `discover_mesh_agents`, `read_mesh_inbox`, `update_mesh_status`.

### REST mesh API (`/api/v1/mesh/*`)

Bypasses MCP for relay clients (e.g. `ship-mcp`). Endpoints:

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/v1/mesh/register` | Register agent, spawn kernel actor |
| POST | `/api/v1/mesh/send` | Directed message to another agent |
| POST | `/api/v1/mesh/broadcast` | Broadcast to all (or filtered) agents |
| GET | `/api/v1/mesh/discover` | List registered agents |
| POST | `/api/v1/mesh/status` | Update agent status |
| GET | `/api/v1/mesh/events/{agent_id}` | SSE stream of agent's mailbox |
| GET | `/api/v1/mesh` | Read-only actor list (registry snapshot) |

### Runtime API

Read-only endpoints for Studio:

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/sessions` | List sessions |
| GET | `/api/v1/actors` | List registered actors |
| GET | `/api/v1/events/stream` | SSE stream of all kernel events |
| GET | `/api/v1/projects/{id}/pty` | WebSocket PTY bridge |

The `workspace` runtime primitive was retired — there are no `workspaces` endpoints. Worktrees are a git feature, not a daemon-tracked resource.

### Project API (`/api/v1/*`)

Session files, git status/diff/log, agent/skill listing, and event emission.

## CORS

Allowed origins: `https://getship.dev`, `localhost:*`, `127.0.0.1:*`. Other browser origins are rejected. Non-browser clients (no `Origin` header) pass through.

## State sharing

All API routes share `ApiState`:

- `kernel: Arc<Mutex<KernelRouter>>` — the single event router
- `mesh_registry: SharedMeshRegistry` — read-optimized agent registry
- `agent_mailboxes: Arc<Mutex<HashMap<String, Mailbox>>>` — stashed for SSE endpoints
- `pty_connections: Arc<Mutex<HashMap<String, usize>>>` — per-branch connection counts
