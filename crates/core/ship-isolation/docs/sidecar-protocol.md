# Sidecar Protocol — RFC

**Status: draft.** Wire shape committed in `src/sidecar/protocol.rs`.
Handlers not yet implemented. This document is the contract the
in-sandbox daemon (`ship-sidecar`) will speak and the host-side
isolator backends will speak to. Modeled on `rivet-dev/sandbox-agent`
with our policy types.

## Transport

- **HTTP/1.1** for control-plane requests.
- **Server-Sent Events (SSE)** for the event stream.
- One transport binding per backend topology:
  - **Local container backend** — Unix socket bind-mounted into the
    container at `/run/ship-sidecar.sock`.
  - **Local microVM backend** — `vsock` listener; host connects on the
    VM's CID.
  - **Cloud backend (Fly, E2B, etc.)** — TCP+TLS on the public
    address of the sandbox VM, bearer-token auth via
    `Authorization: Bearer $SHIP_SIDECAR_TOKEN`.

The HTTP request shape and JSON payloads are identical across all
three transports.

## Authentication

- Local Unix-socket / vsock — transport is the auth (per-sandbox
  socket, no other process can connect).
- Cloud / TCP — `Authorization: Bearer <token>` against
  `SHIP_SIDECAR_TOKEN` set in the sandbox's env at spawn time.
  Missing or wrong token returns `401`.

## Endpoints

### `GET /v1/health` → `200`

Liveness probe. Response:

```json
{ "status": "ok", "version": "0.1.0" }
```

### `POST /v1/session/start` → `200`

Initialize a sandbox session inside the sidecar. The sidecar applies
the policy (mounts, env, network namespace shape) and returns the
session id used by every subsequent call.

```jsonc
// request
{
  "label": "claude-job-abc",
  "policy": { /* SandboxPolicy */ }
}
// response
{ "session_id": "ses_..." }
```

### `POST /v1/exec` → `200`

Start a child process inside the session.

```jsonc
// request
{
  "session_id": "ses_...",
  "bin": "/usr/bin/claude",
  "args": ["-p", "go"],
  "env": { "FOO": "bar" },
  "cwd": "/work"
}
// response
{ "exec_id": "exe_...", "pid": 12345 }
```

### `POST /v1/signal` → `204`

Send a signal to a running exec.

```json
{ "exec_id": "exe_...", "signal": 15 }
```

### `POST /v1/stdin` → `204`

Write bytes to a running exec's stdin.

```json
{ "exec_id": "exe_...", "data_b64": "..." }
```

### `POST /v1/mount` → `204`

Add a runtime mount to a session.

### `POST /v1/policy` → `204`

Replace the network/env policy on a live session. Mount-set changes
require `POST /v1/mount` calls.

### `POST /v1/port-publish` → `200`

Publish an internal port to the host. Response includes the assigned
external port.

```jsonc
// request
{ "session_id": "ses_...", "internal_port": 3000, "external_port": null }
// response
{ "external_port": 17234 }
```

### `POST /v1/shutdown` → `204`

Gracefully tear down a session.

### `GET /v1/events` — text/event-stream

SSE feed of every event the session emits. One JSON object per
`data:` line. Event types:

- `stdout` — `{ "exec_id", "data_b64" }`
- `stderr` — `{ "exec_id", "data_b64" }`
- `exited` — `{ "exec_id", "code", "signal" }`
- `policy_denied` — `{ "session_id", "kind", "detail" }`
- `network_blocked` — `{ "session_id", "host" }`
- `sidecar_shutdown` — `{ "reason" }`

All events carry a `type` discriminator (serde tag).

## Versioning

The path prefix is `/v1`. Breaking changes bump to `/v2`. Additive
fields land in `/v1` and are tolerated by older sidecars (use
`#[serde(default)]` for new optional fields).

## Open questions

- **Backpressure.** Should `stdout`/`stderr` events ever be dropped
  under load, or always queued? Currently undecided; default to
  queued with a generous bound and document the bound.
- **Multi-exec stdio multiplexing.** Today every exec's stdout flows
  through a single SSE stream tagged by `exec_id`. Reasonable for
  observability; clients filter by id.
- **Reauth.** If the sandbox lives long enough that the bearer token
  rotates, how does the host hand the sidecar a new token? Open.
  Probably a `POST /v1/auth-rotate` endpoint with the old token in
  the header and the new one in the body.
- **Bulk file transfer.** `data_b64` over JSON is fine for stdio but
  bad for copying multi-MB blobs. A separate `POST /v1/files` with
  raw body and `Content-Type: application/octet-stream` likely belongs
  in v2.
