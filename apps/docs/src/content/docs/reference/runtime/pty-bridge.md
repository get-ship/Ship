---
title: "PTY Bridge"
description: "WebSocket ↔ tmux attach-session bridge. Lets a browser hold a live terminal into a workspace's tmux session."
sidebar:
  label: "PTY Bridge"
  order: 13
---
`apps/shipd/src/pty_handler.rs`. A WebSocket endpoint that spawns `tmux attach-session` and pipes bytes between the WebSocket and the tmux process's stdio. Used by Studio (browser) to attach to an agent's live terminal session.

Route: `GET /api/workspaces/:id/pty`

Upgrades to WebSocket, spawns tmux, bridges the frame stream. Multiple clients can attach to the same tmux session concurrently — tmux handles the multiplexing. Limit: 5 concurrent attachments per workspace.

## Flow

```rust
pub async fn workspace_pty(
    Path(workspace_id): Path<String>,
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    request: Request,
) -> Response
```

1. **Scope check.** `auth.has_scope(Scope::WriteWorkspace)` — PTY access is shell access, scoped to write. 403 if missing.
2. **Workspace lookup.** `runtime::workspace::get_workspace_by_id(ship_dir, &workspace_id)`. 404 if absent.
3. **Resolve tmux session name.** `runtime::db::workspace_state::get_workspace_tmux_session_db(&workspace_id)`. The session name was set via `set_workspace_tmux_session` when the workspace started interactive. 404 if unset.
4. **Connection limit.** Increment a per-workspace counter in `state.pty_connections()`. 429 if at `MAX_PTY_CONNECTIONS = 5`.
5. **Manual WebSocket upgrade.** Extract the upgrade after validation succeeded — the handler has to do this manually because otherwise Axum's `WebSocketUpgrade` extractor runs first and returns 426 before the validation code can respond with 404/429. The ordering is deliberate so tests can reach those paths without an actual upgrade.
6. **Spawn tmux + bridge.** `handle_pty_socket` spawns `tmux attach-session -t {session_name}`, wires stdin/stdout to the WebSocket via `tokio::io::{AsyncReadExt, AsyncWriteExt}`.

A `PtyConnectionGuard` decrements the counter on drop so client disconnects free the slot.

## Why tmux, not a raw PTY

- Multiplexing across clients: several browser tabs / studios can share one session without fighting for stdin.
- Session persistence: closing the browser tab doesn't kill the agent process. Reattaching picks up where things were.
- Scrollback: tmux already buffers history, so new connectors see the context.
- Standard behavior: operators can also attach from a local terminal with `tmux attach -t {name}` and get the same state.

The session is created by the `TmuxWindowExecutor` (or legacy `TmuxExecutor`) when it spawns an interactive job. The PTY handler just hooks existing sessions — it doesn't create them.

## Connection limit

`MAX_PTY_CONNECTIONS = 5` per workspace. Counter stored in `ApiState.pty_connections`: `Arc<Mutex<HashMap<String, usize>>>`. Increment on accept, decrement on drop via `PtyConnectionGuard`.

Limit is per workspace, not per user or per session. A single workspace can have at most 5 active PTY WebSockets across all clients. Tmux itself handles the fan-out to attached clients.

## Bytes on the wire

- **Client → server**: WebSocket binary frames, fed to tmux's stdin.
- **Server → client**: tmux stdout chunks, sent as WebSocket binary frames.

No framing protocol beyond WebSocket. Terminal resize isn't handled today — the tmux session geometry is set by whatever first created it.

## Auth

Session token-backed. `AuthContext.has_scope(Scope::WriteWorkspace)` is the single check. PTY access is as privileged as any other workspace write because pasting into a terminal is arbitrary code execution.

See [Scopes](/reference/runtime/scopes/) for the scope model.

## Edge cases

- **No tmux session**: the workspace exists but `set_workspace_tmux_session` was never called. Usually means the workspace hasn't run an interactive job yet. 404 with message.
- **tmux binary missing**: `Command::new("tmux")` fails to spawn. The handler surfaces a connection error over the WebSocket; the frontend should display it and allow retry.
- **Session dies during attach**: tmux exits, the stdout reader sees EOF, the handler closes the WebSocket cleanly. Reconnect shows 404 if the underlying session is gone.

## Not yet implemented

- **Resize handling.** Clients send resize events (rows/cols change in the browser); the handler should forward to tmux via its control socket. Currently: geometry is fixed at session-create time.
- **Input record/replay.** No audit of keystrokes. PTY sessions are interactive; tmux's own logging is the only trail.
- **Per-user connection limit.** Only per-workspace. A single malicious token could fill all 5 slots.

## Related

- [Supervisor](/reference/runtime/supervisor/) — `TmuxWindowExecutor` creates the sessions the bridge attaches to.
- [Scopes](/reference/runtime/scopes/) — `Scope::WriteWorkspace` gates PTY access.
- [Daemon](/reference/runtime/daemon/) — where this route is registered.
