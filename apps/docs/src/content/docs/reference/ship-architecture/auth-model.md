---
title: "ship-architecture"
---
The Ship daemon (`shipd`) exposes HTTP endpoints on localhost. Because anything running
as the same user can reach those endpoints, the daemon authenticates every caller and
scopes what each can do.

This document is for contributors. For user-facing docs, see
[Connecting Studio to your daemon](/reference/studio/auth/).

## Token types

The daemon accepts four kinds of bearer tokens, each with distinct provenance and scopes:

| Token | Format | Provenance | Default scopes |
|-------|--------|------------|----------------|
| **Daemon token** | 64-char hex in `~/.ship/daemon.token` | Generated on first run | `Scope::Full` |
| **Hook token** | `ship_hook_` + 32-char hex in `~/.ship/hook.token` | Generated on first run, never rotates automatically | `EmitEvents`, `LogProgress` |
| **Session token** | `ship_sess_` + 32-char hex | Issued via pairing flow (see below) | `Scope::Full` (paired) or role-based (MCP) |
| **SDK token** | `ship_` prefix | Matches daemon token | `Scope::Full` |

Only the daemon token file is readable by any process running as the user. The others
are either request-specific (session) or purpose-specific (hook).

## Auth middleware flow

On every request the middleware:

1. Extracts the token from `Authorization: Bearer <token>` header
2. On GET requests only, falls back to `?token=` query param (for browser
   EventSource/WebSocket, which can't set headers)
3. Classifies by prefix and dispatches:
   - `ship_hook_` → compared to `hook.token`, granted narrow scopes
   - `ship_sess_` → looked up in `SessionRegistry`, granted the scopes bound at creation
   - 64 hex chars → compared to `daemon.token`, granted `Scope::Full`
   - `ship_` prefix → legacy SDK token path, granted `Scope::Full`
4. Identity binding for session tokens: `agent_id` and `workspace_id` come from the
   session registry, **not** from client-supplied `x-ship-*` headers. This is intentional
   defense against session-token-wielding agents impersonating other agents.

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

The browser can't read `~/.ship/daemon.token` (cross-origin, no filesystem access). The
pairing flow proves physical control of the machine instead:

1. **`POST /auth/pair/init`** (unauth) — daemon returns a short code (XXX-XXX) and a
   long random `claim_token`. Code is human-readable; claim_token is unguessable.
2. **`POST /api/auth/pair/confirm`** (requires daemon token) — CLI reads daemon.token
   from disk, posts the code. Daemon marks the code approved and issues a session token
   bound to the claim_token.
3. **`GET /auth/pair/status?claim_token=...`** (unauth, but claim_token is secret) —
   client polls until it gets `{status: "approved", session_token: "..."}`. Codes are
   single-use and the claim_token is consumed on the first successful status fetch.

**Security properties:**
- Agents without daemon.token access can't complete step 2. The pairing code is
  meaningless without CLI confirmation.
- Agents with daemon.token can complete step 2, but they already have `Scope::Full` —
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

## Scopes

Coarse-grained capability scopes, checked by handlers via `require_scope()`:

| Scope | Grants |
|-------|--------|
| `ReadWorkspace` | GET workspace/session state |
| `WriteWorkspace` | Write session files, agent profiles, pipelines, PTY access |
| `ReadJobs` / `CreateJobs` / `ManageJobs` | Job lifecycle operations |
| `EmitEvents` | Emit events to the kernel |
| `MeshSend` / `MeshBroadcast` | Send/broadcast via mesh |
| `LogProgress` / `StepDone` | Progress reporting |
| `AdminDaemon` | Settings mutations, rebuild-projections |
| `Full` | Bypass — granted by daemon token and paired sessions |

Role-based defaults for MCP-issued session tokens live in `scopes_for_role()`.

## Persistence

Paired (non-ephemeral) sessions are serialized to `platform.db` via the `kv_state` table
under the `auth.sessions` namespace. On daemon startup, `hydrate_sessions()` loads all
non-expired sessions and purges expired ones. Dev-mode sessions are never persisted.

Audit log rows (pair confirm, pair revoke) are written to the `auth.audit` namespace in
`kv_state`, keyed by `{unix_timestamp}-{token_prefix}` so they sort chronologically.

## Known limitations (unresolved)

### Daemon token file access
`~/.ship/daemon.token` is mode 0o600 but readable by any process running as the same
user. We don't use OS keyrings yet. Consequence: a compromised agent running as your
user can read the daemon token and escalate to `Scope::Full`.

**Mitigation path**: move to OS-keyring-backed storage (macOS Keychain, Linux Secret
Service, Windows Credential Manager).

### MCP session role claim
`POST /api/session/register` trusts the `role` field in the request body. An MCP server
that claims `role: "contributor"` gets `Scope::Full`. Today the MCP server is trusted
because it's spawned by the daemon itself, but this trust boundary isn't enforced.

**Mitigation path**: daemon stores approved role per agent/workspace in platform.db at
`ship use` time; `session_register` looks up the approved role instead of trusting the
request.

## File layout

- `apps/shipd/src/auth.rs` — middleware, token types, pair/attest/session endpoints, audit log
- `apps/shipd/src/scopes.rs` — `Scope` enum, `SessionRegistry`, persistence helpers
- `apps/shipd/src/pairing.rs` — `PairingRegistry`, code generation, dev mode flag
- `apps/shipd/src/attest.rs` — JWKS fetch+cache, JWT verify, daemon-owner binding
- `apps/shipd/src/pairing_integration_tests.rs` — end-to-end pair flow tests
- `apps/web/src/lib/daemon-jwt.ts` — JWT signing (jose library), JWKS builder
- `apps/web/src/routes/api/jwks.ts` — public JWKS endpoint
- `apps/web/src/routes/api/auth/daemon-attest.ts` — authenticated JWT minting
