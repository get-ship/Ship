---
group: Architecture
title: Daemon Authentication Model
description: How shipd authenticates HTTP callers and scopes what each can do.
audience: internal
---

# Daemon authentication model

The Ship daemon (`shipd`) exposes HTTP endpoints on localhost. Because anything running
as the same user can reach those endpoints, the daemon authenticates every caller and
scopes what each can do.

This document is for contributors. For user-facing docs, see
[Connecting Studio to your daemon](/studio/auth/).

## Token types

The daemon accepts three kinds of bearer tokens, each with distinct provenance and scopes:

| Token | Format | Provenance | Default scopes |
|-------|--------|------------|----------------|
| **Daemon token** | 64-char hex in `~/.ship/machine/daemon/token` | Generated on first run | `Scope::Full` |
| **Session token** | `ship_sess_` + 32-char hex | Issued via pairing flow (see below) | `Scope::Full` (paired) or role-based (MCP) |
| **SDK token** | `ship_` prefix | Matches the daemon token | `Scope::Full` |

The daemon token covers CLI, Studio, and hook callers. Only the daemon token file is
readable by any process running as the user; session tokens are request-specific.
The `ship_hook_` token class has been removed — tokens with that prefix are rejected.

## Auth middleware flow

On every request the middleware:

1. Extracts the token from `Authorization: Bearer <token>` header
2. On GET requests only, falls back to `?token=` query param (for browser
   EventSource/WebSocket, which can't set headers)
3. Classifies by prefix and dispatches:
   - `ship_sess_` → looked up in `SessionRegistry`, granted the scopes **and grants** bound at creation
   - 64 hex chars → compared to the daemon token, granted `Scope::Full`
   - `ship_` prefix → SDK token path, granted `Scope::Full`
   - `ship_hook_` → rejected (the hook-token class is removed)
   - anything else that matches the expected token but fits no known format →
     **rejected (fail closed)**, not granted `Cli`/`Full`
4. Identity binding for session tokens: `agent_id` and `workspace_id` come from the
   session registry, **not** from client-supplied `x-ship-*` headers. This is intentional
   defense against session-token-wielding agents impersonating other agents.

## Host socket: a fourth, tokenless auth source

The three token types above are the **TCP** path (browser, operator CLI, SDK). A
dispatched **agent gets no token**. Instead the daemon binds `<daemon_dir>/ship.sock`
and reads the caller's identity from the connection's **kernel peer credentials**
(`SO_PEERCRED`), maps the PID to the agent's process container, and builds the
*same* `AuthContext` the token branches build — so the gate below is unchanged.
`auth_middleware` short-circuits when an `AuthContext` is already present, letting
the peer-cred middleware (socket) and the token middleware (TCP) be two sources of
one gate. Because a process can't forge its PID to the kernel, this is a per-agent
credential the agent cannot mint or spoof. Full chain, per-OS matrix, and limits:
[the agent access model](/reference/architecture/agent-access/).

## Capability gate

Identity (above) answers *who is calling*; the **gate** answers *may they call this*.
It is one decision, enforced at the dispatch chokepoint — `services::gate`, invoked by
`services::command::call` — so every write reaches it identically over REST and Connect
(there is no per-transport gate). Reads are C0, authenticated at the edge.

The model is **caller × blast-tier × per-service ceiling** — the two axes that
already exist (the proto **service** an op belongs to, and its risk **tier**), with
**no separate capability taxonomy**:

- **`blast_radius`** is the op's risk tier, on its descriptor, four levels ordered
  `C0 < C1 < C2 < C3`:
  - **C0** read — anyone.
  - **C1** workspace-write — any agent (the *operational baseline*: a dispatched
    agent reads + workspace-writes everywhere out of the box).
  - **C2** elevated — dispatch/stop/steer, authoring (agents/skills/mcp/create/view).
    Granted, per service.
  - **C3** critical — account, billing, permissions, `agent.grant`. Operator/2FA
    only; never an agent. Unrecognized → fails closed to C3.
- **An agent's authority** is a default ceiling (C1) plus the set of **services it's
  elevated in** (the C2 set), read from `AuthContext.grants`. A **service** is the
  op's domain (`git.commit` → `git`) — the proto-service curation, via `domain_of`,
  not a second taxonomy.

`decide(blast, service, caller)`:

| | C0 read | C1 write | C2 elevated | C3 critical |
|---|---|---|---|---|
| operator | allow | allow | allow | allow |
| agent | allow | allow | allow **iff service ∈ elevated** | **deny** |

<ship-alert tone="warning">
C1/C2 enforcement is least-privilege + accident-prevention, **not** a Tier-1 boundary
— a host agent shares the uid and can act outside the daemon. The real boundary is
credential location (C3 never lands on disk; it lives in the GUI/2FA session — see
#86) plus Tier-2+ isolation for untrusted work.
</ship-alert>

### Authority: require, grant, manage

- A workflow/app declares **`requires:`** (public, authored — the capabilities it
  needs). **Skills do not `grant:`** — that field is retired. An unmet requirement
  surfaces at compile/install.
- The **user grants**; the **runtime manages.** An agent's elevated services are
  operator-approved state in the per-agent **provisioned baseline** (#85), carried
  on `AuthContext.grants`. Role defaults seed it (`grants_for_role`: commander →
  `[agent]`; designer → the authoring services) until per-agent provisioning.
- **`agent.grant`** (C3, operator-only) is the human-in-the-loop approval point: it
  adds elevated services to an agent's live sessions — no re-mint, no recompile.

## Three auth paths for browser clients

Studio (and any other browser client) tries three authentication methods, in order:

1. **URL fragment token** — `ship studio` launches the browser with `#token=...`
2. **Server attestation** — zero-click via getship.dev JWT
3. **Pair code** — one-time CLI confirmation; offline-capable

All three produce the same `ship_sess_*` session token format with 30-day TTL. The daemon
doesn't distinguish how the session was issued — every session is just a row in the
registry with scopes and an expiry.

## Server attestation

getship.dev signs short-lived (1h) JWTs claiming "this authenticated user is X." The
daemon verifies the signature against getship.dev's published JWKS and binds the claim
to a local user record set at `ship login` time.

**Flow:**

```
 Browser (signed in to getship.dev)
   │
   │  1. POST /api/auth/daemon-attest  (cookie auth via Better Auth)
   ▼
 getship.dev Worker
   │
   │  2. requireSession() → user.id
   │  3. signDaemonJwt({ sub: user.id, aud: "ship-daemon", exp: +1h, plan, caps })
   │
   ▼ returns { jwt, expires_at }
 Browser
   │
   │  4. POST /auth/server-attest { jwt }
   ▼
 shipd (local daemon)
   │
   │  5. Fetch /api/jwks from configured issuer (cached 1h)
   │  6. decode+verify: alg=EdDSA, aud=ship-daemon, exp>now, iss matches config
   │  7. Compare claims.sub with stored daemon_owner (set by ship login)
   │  8. Issue session token, persist to KV
   │
   ▼ returns { session_token, expires_at_unix }
 Browser stores session_token, uses it for 30 days
```

**JWT claim structure:**

```json
{
  "iss": "https://getship.dev",
  "sub": "user-abc123",
  "aud": "ship-daemon",
  "iat": 1700000000,
  "exp": 1700003600,
  "plan": "pro",
  "caps": ["pro"]
}
```

Extra claims are ignored. The daemon's scope enforcement uses `paired_session_scopes()`
(currently `[Full]`); richer scope derivation from `caps` is a future extension.

**Key rotation:**

The gateway publishes its public key at `/api/jwks`. During rotation, both the outgoing
and incoming keys appear in the JWKS briefly. Signing uses `SHIP_JWT_PRIVATE_KEY`
(Cloudflare secret, Ed25519 PKCS8 PEM) and stamps the `kid` header from
`SHIP_JWT_PRIVATE_KEY_ID`. The daemon's JWKS cache invalidates on signature failure,
forcing a refetch — so rotation takes effect within seconds.

**User-daemon binding:**

`ship login` fetches `user.id` from `/api/me` and writes it to `platform.db` KV as
`auth.daemon_owner.user_id`. On every server-attest, the daemon compares this stored
owner against the JWT's `sub`. If they don't match → 403. If no owner is set (first
login on this machine) → bind to the first successful attest's `sub`.

This prevents "attacker signs in at getship.dev, attempts to hijack my local daemon"
via a stolen JWT. The attacker would need both a valid JWT **and** a daemon whose owner
binding matches — which requires physical access to run `ship login`.

**Configurable issuer:**

The daemon's trust root is configurable via `auth.attestation_issuer` in runtime
settings (defaults to `https://getship.dev`). This enables:

- Local dev of ship-cloud: point daemon at `http://localhost:3000`
- Self-hosted getship.dev: point daemon at your own domain
- Multiple environments: staging vs production keys

## Pairing flow (browser clients, fallback)

Used when server attestation isn't available (user not signed in, offline, self-hosted
without getship.dev account).

The browser can't read `~/.ship/machine/daemon/token` (cross-origin, no filesystem
access). The pairing flow proves physical control of the machine instead:

1. **`POST /auth/pair/init`** (unauth) — daemon returns a short code (XXX-XXX) and a
   long random `claim_token`. Code is human-readable; claim_token is unguessable.
2. **`POST /api/auth/pair/confirm`** (requires daemon token) — CLI reads the daemon token
   from disk, posts the code. Daemon marks the code approved and issues a session token
   bound to the claim_token.
3. **`GET /auth/pair/status?claim_token=...`** (unauth, but claim_token is secret) —
   client polls until it gets `{status: "approved", session_token: "..."}`. Codes are
   single-use and the claim_token is consumed on the first successful status fetch.

**Security properties:**
- Agents without daemon-token access can't complete step 2. The pairing code is
  meaningless without CLI confirmation.
- Agents with the daemon token can complete step 2, but they already have `Scope::Full` —
  nothing is escalated.
- Codes expire in 5 minutes, single-use. Claim tokens expire in 30 minutes.
- Paired sessions last 30 days, persisted in `platform.db` via the KV layer.

## Dev mode

`ship daemon start --dev` relaxes auth for local development:

- `/auth/pair/init` auto-approves with `Scope::Full` — no CLI confirmation needed
- Daemon forces `127.0.0.1` bind regardless of `--host`
- Loud warning banner on startup, `WARN`-level log on every relaxed request
- Dev sessions are marked `ephemeral` and never persist across restarts
- `/api/health` reports `dev_mode: true` so clients can display it

The flag is a **process-level CLI argument**. Agents can set environment variables or
modify config files, but they can't re-exec the daemon process with `--dev`. Only the
human who started the daemon can enable it.

## Scopes (legacy — REST route-auth + artifacts)

The capability model is the **gate** above (caller × blast-radius × grant). The
older `Scope` enum is now narrowed to two jobs and is **slated for removal with the
REST surface**:

- **REST route-auth** — `RequiredAuth::Scope(...)` / `@auth scope ...` on the
  remaining REST routes (`Scope::Full` is effectively the operator marker).
- **Artifact permissions** — `can_read/write/delete_artifacts`.

`Scope` is multi-purpose (operator marker + artifact perms + route-auth), so it
can't be deleted until the REST routes that consume it are removed; until then
`scopes` rides alongside `grants` on `AuthContext` as a migration state. New
capability decisions go through the gate, not `require_scope()`.

## Persistence

Paired (non-ephemeral) sessions are serialized to `platform.db` via the `kv_state` table
under the `auth.sessions` namespace. On daemon startup, `hydrate_sessions()` loads all
non-expired sessions and purges expired ones. Dev-mode sessions are never persisted.

Audit log rows (pair confirm, pair revoke) are written to the `auth.audit` namespace in
`kv_state`, keyed by `{unix_timestamp}-{token_prefix}` so they sort chronologically.

## Security posture ledger

The canonical list of auth/credential loose ends — what's closed, what's open, the
severity, and the path. Kept here as the single source of truth so the open items
survive a context switch into runtime/backend work.

### Credential storage (inventory)

All machine-local secrets are `0o600` and never sync across machines. Storage
*hygiene* is not the loose end — *authority* (below) is.

| Secret | Location | Notes |
|--------|----------|-------|
| Daemon token | `~/.ship/machine/daemon/token` | 64-hex; grants `Full` today (see #86) |
| Account token / MCP bearer | `~/.ship/machine/credentials.jsonc` | Ship Cloud token + optional `ship mcp serve` bearer |
| Account key | `account/store.rs` (`0o600`) | gateway-issued; revoked on logout |
| Session tokens | in-memory `SessionRegistry` + `platform.db` (paired only) | 30-day TTL; dev sessions never persist |

### Closed

- **Per-agent identity is unspoofable on the host socket** — kernel peer
  credentials, not a claimed role. See [agent access](/reference/architecture/agent-access/).
- **An agent cannot self-escalate its Ship capabilities.** The gate reads grants
  from the operator-approved, **event-sourced** `provisioned_grants` baseline
  (#85), not from the agent's skill files. Editing a skill's `requires:` changes
  what's *requested* at next compile; it never changes the *baseline*, which only
  `provision_grants` (C2, operator) writes. Tamper-evident by the log.
- **MCP role-claim escalation is blunted.** A session token is `CallerKind::Mcp`,
  denied every C2 op regardless of claimed role; on the socket, role isn't claimed
  at all. Residual risk is confined to the external `apps/mcp` adapter's token path.

### Open

| # | Item | Sev | State / path |
|---|------|-----|--------------|
| **#86** | **Daemon token grants `Full`/C2** | **High** | The on-disk token is `Cli + Full` → `is_operator` → clears *every* class incl. C2 (credential mint, billing, acting as another actor). Intended: **nothing on disk is ever C2** — the token bootstraps C0/C1; C2 needs the GUI/2FA elevated session. **Blocked on a product decision** (below). |
| — | C2-from-CLI is unaudited | Med | C2 ops invoked via the daemon token emit no security-audit event. Cheap, non-breaking first step toward #86: instrument every C2-cleared-by-operator decision. |
| — | Harness-level tool permissions are agent-editable on disk | Med | The Ship capability gate is tamper-resistant (above), but the *compiled harness* permission files (e.g. `.claude` settings) can be edited by the agent with no detection. The gate is the real boundary; harness perms are defense-in-depth only. Ties to disk-as-projection hardening + file provenance (#84/#92). |
| — | Live cross-process socket smoke not run | Low | The socket chain is unit + in-process-e2e verified; a real daemon-restart + dispatched-agent smoke (resolve → `Agent(id)` → scoped grants over `ship.sock`) is pending — needs restarting the running daemon. |
| — | macOS `peer_cred` unverified on hardware | Low | `LOCAL_PEERPID` + `getpeereid` arm written against libc; not compile-checked on a Mac. |
| — | macOS/Windows container backends | Med | Pending; those hosts are operator-only over the socket (macOS) or TCP+token (Windows). Runtime/backend phase. |

### #86 — the decision it needs

Hard-denying C2 to the daemon token breaks operator flows that legitimately run
from the CLI today — `agent.grant` (the human approval point), `ship login`
(account), `permissions.*`. So #86 isn't a flip; it needs a ruling on **which C2
ops must remain CLI-reachable vs move to GUI/2FA-only**, then: split the gate's
single `is_operator` into `operator` (C0/C1, the daemon token) × `elevated` (C2,
the GUI/2FA session); C2 checks `elevated`. The non-breaking first step (audit
instrumentation, above) can land independently and makes the eventual flip
observable.

## File layout

Pure auth primitives live in `crates/core/ship-platform/src/auth/`; the axum-typed HTTP surface lives in `apps/shipd/src/auth/`.

- `crates/core/ship-platform/src/auth.rs` — module root re-exporting the four submodules
- `crates/core/ship-platform/src/auth/scopes.rs` — `Scope` enum, `SessionRegistry`, pure scope policy
- `crates/core/ship-platform/src/auth/pairing.rs` — `PairingRegistry`, code generation, dev mode flag
- `crates/core/ship-platform/src/auth/attest.rs` — JWKS fetch+cache, JWT verify, daemon-owner binding
- `crates/core/ship-platform/src/auth/session.rs` — browser-session cookie generation/parsing/registry
- `apps/shipd/src/auth.rs` — middleware, token types, re-exports of `attest`/`pairing`
- `apps/shipd/src/auth/scopes.rs` — HTTP-coupled scope helpers + `/api/session/register` handler
- `apps/shipd/src/auth/session.rs` — `POST /api/v1/runtime/session` issuance handler
- `apps/shipd/src/auth/api/` — pairing, attest, and session-token HTTP routes
- `apps/shipd/src/auth/pairing_integration_tests.rs` — end-to-end pair flow tests
- `apps/gateway/src/lib/daemon-jwt.ts` — JWT signing (jose library), JWKS builder
- `apps/gateway/src/routes/auth.ts` — `/api/jwks` endpoint and `/api/auth/daemon-attest` route
