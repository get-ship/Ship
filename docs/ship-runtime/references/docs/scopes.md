---
group: Runtime
order: 15
title: Scopes & Auth
description: Token types, scope model, role-to-scope mapping, session registry, persistence.
audience: internal
---

# Scopes & Auth

`apps/shipd/src/scopes.rs`, `apps/shipd/src/auth.rs`. Bearer-token auth with per-endpoint scope checks. The daemon never runs unauthenticated.

## Tokens

Four kinds. All formatted as bearer headers: `Authorization: Bearer <token>`.

| Token | Format | Provenance | Default scopes |
|---|---|---|---|
| **Daemon token** | 64-char hex in `~/.ship/daemon.token` | Generated on first run | `Scope::Full` |
| **Hook token** | `ship_hook_` + 32-char hex in `~/.ship/hook.token` | Generated on first run, never rotates automatically | `EmitEvents`, `LogProgress` |
| **Session token** | `ship_sess_` + 32-char hex | Issued via pairing flow | `Full` (paired) or role-based (MCP) |
| **SDK token** | `ship_` prefix | Matches daemon token | `Full` |

The auth middleware reads the header, matches the prefix, and resolves to an `AuthContext` carrying agent id + granted scopes.

## Scope enum

```rust
pub enum Scope {
    ReadWorkspace,
    WriteWorkspace,
    ReadJobs,
    CreateJobs,
    ManageJobs,
    EmitEvents,
    MeshSend,
    MeshBroadcast,
    LogProgress,
    StepDone,
    AdminDaemon,
    Full,  // bypasses all checks
}
```

Twelve scopes. Handlers call `require_scope(&auth, Scope::X)` at entry; middleware doesn't pre-filter. A failure returns 403 early.

`Scope::is_granted_by(granted)` returns true if `granted` contains the scope OR `Full`.

## Role → scope mapping

`scopes_for_role(role)` returns the default scope list for each role string:

| Role | Scopes |
|---|---|
| `commander` | ReadWorkspace, WriteWorkspace, ReadJobs, CreateJobs, ManageJobs, EmitEvents, MeshSend, MeshBroadcast, LogProgress |
| `lead` | ReadWorkspace, WriteWorkspace, ReadJobs, CreateJobs, LogProgress, StepDone, MeshSend, EmitEvents |
| `contributor` | **Full** |
| `worker` (default) | ReadWorkspace, ReadJobs, LogProgress, StepDone, MeshSend, EmitEvents |
| unknown / explicit fallback | ReadWorkspace, ReadJobs, LogProgress |

MCP-initiated sessions get scopes via this role mapping. The CLI path (CLI token = daemon token) bypasses via `Full`.

## DaemonSession

```rust
pub struct DaemonSession {
    pub token: String,
    pub agent_id: String,
    pub scopes: Vec<Scope>,
    pub workspace_id: Option<String>,
    pub created_at: Instant,
    pub expires_at_unix: Option<i64>,
    pub ephemeral: bool,
}
```

Session registry: `Arc<Mutex<HashMap<String /* token */, DaemonSession>>>`. Lives on `ShipRuntime.session_registry`.

- `expires_at_unix: None` — no expiry (in-memory only, typically paired dev sessions).
- `ephemeral: true` — not persisted. Dev-mode auto-pair uses this.

`is_expired()` checks against `chrono::Utc::now().timestamp()`.

## Persistence

Persisted sessions live in `platform.db`'s `kv_state` under namespace `auth.sessions`, keyed by token. Stored as JSON:

```rust
struct PersistedSession {
    token, agent_id, scopes, workspace_id,
    created_at_unix, expires_at_unix,
}
```

Ephemeral sessions are skipped. Sessions with `expires_at_unix: None` also skip (conservative — no expiry stored means no auto-cleanup, so don't persist).

`persist_session(&session)` — writes on pairing success.
`remove_persisted_session(token)` — removes on logout.
`hydrate_sessions(&registry)` — called at daemon startup (outside dev mode). Loads all persisted sessions, drops expired ones, stuffs the rest into the registry. Expired sessions are deleted from KV on the way through.

## Pairing flow

`apps/shipd/src/pairing.rs`. Browser clients (Studio) don't have access to the daemon token — they authenticate via pairing:

1. Client requests a pairing code (`POST /api/pair/request`).
2. Daemon generates code, stores pending pairing.
3. User approves the pairing in the shell/CLI (`ship daemon pair <code>` or Studio prompt).
4. Daemon issues `session token + scope set` and writes it to `DaemonSession`.
5. Client uses the session token for subsequent requests.

Dev mode (`run_network(dev=true)`) auto-pairs — new browser clients immediately get a session token with `Full` scopes. The registry still tracks them, but they're `ephemeral: true` (no DB persistence).

## Attestation

`apps/shipd/src/attest.rs`. `AttestationVerifier` handles cryptographic proofs for SDK tokens — SDK callers sign requests with a keypair, daemon verifies. Used when the SDK runs on an untrusted host and needs to prove provenance.

## Middleware

Every route goes through the auth layer, which:

1. Reads `Authorization` header.
2. Matches token prefix → token type.
3. For daemon/hook tokens: compare against `~/.ship/*.token` bytes in constant time.
4. For session tokens: look up in the session registry, check expiry.
5. Build an `AuthContext { agent_id, scopes, ... }` and stash it in request extensions.
6. Reject with 401 if no match, 403 if expired.

Handlers extract via `Extension(auth): Extension<AuthContext>` and call `require_scope` at the top.

## Invariants

- Daemon token never expires. Rotate via `~/.ship/daemon.token` replace + daemon restart.
- Hook token never rotates automatically. Pre-configured hooks rely on stable value.
- Session tokens expire by policy. Expired = removed on access or by hydration.
- `Scope::Full` is a single-word bypass — auditors should flag every token holding it.
- Ephemeral sessions never touch the database. Dev-mode-only.
- `require_scope` returns 403 cleanly — no handler ever runs with insufficient scope.

## Related

- [Daemon](/runtime/daemon/) — startup wires token + session registry.
- [PTY Bridge](/runtime/pty-bridge/) — uses `Scope::WriteWorkspace`.
- [Events](/runtime/events/) — `EmitEvents` and `LogProgress` scopes gate event emission for untrusted callers.
