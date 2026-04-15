# Security Posture Audit — Unified Auth Map

Date: 2026-04-14
Auditor: Automated contributor (pipeline step 2)

---

## 1. Inventory of Authentication and Authorization Surfaces

### 1.1 Daemon Bearer Token (`daemon.token`)

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/auth.rs:74-86` (generation), `apps/shipd/src/auth.rs:89-98` (write), `apps/shipd/src/auth.rs:100-102` (read) |
| **Identity asserted** | Local machine owner — any process that can read `~/.ship/daemon.token` |
| **Issuer** | Daemon process at startup via `generate_token()` (`apps/shipd/src/auth.rs:74`) |
| **Consumer** | CLI, Studio (via fragment token or pairing), MCP server bootstrap, any local HTTP client. Auth middleware at `apps/shipd/src/auth.rs:157-292` validates. |
| **Rotation / expiry** | Regenerated on every daemon restart. No TTL — lives as long as the daemon process. Previous token becomes invalid immediately. File permissions set to `0o600` (`apps/shipd/src/auth.rs:94-95`). |
| **Failure mode** | `401 Unauthorized "invalid token"` (`apps/shipd/src/auth.rs:267`). All endpoints except unauthenticated pairing/attest routes are blocked. |

### 1.2 Hook Tokens (`ship_hook_*`)

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/auth.rs:108-128` (generation, write, read), `apps/shipd/src/auth.rs:131-142` (ensure on first use) |
| **Identity asserted** | Provider hook caller — external system delivering lifecycle events |
| **Issuer** | Daemon on first use via `ensure_hook_token()` (`apps/shipd/src/auth.rs:131`). Written to `~/.ship/hook.token` with `0o600` perms. |
| **Consumer** | External hook callers (CI providers, deploy hooks). Auth middleware at `apps/shipd/src/auth.rs:222-235` validates against expected hook token. |
| **Rotation / expiry** | Never auto-rotated. Manual rotation via CLI only (`apps/shipd/src/auth.rs:106-107` doc comment). Survives daemon restarts (persisted to disk). |
| **Failure mode** | `401 Unauthorized "invalid hook token"` (`apps/shipd/src/auth.rs:224`). Granted narrow scopes: `[EmitEvents, LogProgress]` only (`apps/shipd/src/auth.rs:232`). |

### 1.3 MCP Session Tokens (`ship_sess_*`) and Scope Model

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/scopes.rs:210-218` (generation, prefix check), `apps/shipd/src/scopes.rs:261-293` (`session_register` endpoint) |
| **Identity asserted** | MCP agent with a declared role (worker, lead, commander, contributor). Identity bound at registration — `apps/shipd/src/auth.rs:241-263` enforces that `agent_id` comes from the session, not client headers. |
| **Issuer** | Daemon via `POST /api/session/register`. MCP server calls this at startup. Scopes computed from role via `scopes_for_role()` (`apps/shipd/src/scopes.rs:40-74`). |
| **Consumer** | MCP server process, which includes the token in all subsequent daemon requests. |
| **Rotation / expiry** | No expiry (`expires_at_unix: None` at `apps/shipd/src/scopes.rs:284`). Ephemeral — lives only in memory, not persisted (`apps/shipd/src/scopes.rs:285`). Dies with daemon restart. |
| **Failure mode** | `401 Unauthorized "invalid session token"` (`apps/shipd/src/auth.rs:260`). Token not in registry = immediate rejection. |

**Scope model** (`apps/shipd/src/scopes.rs:16-30`):
- `ReadWorkspace`, `WriteWorkspace`, `ReadJobs`, `CreateJobs`, `ManageJobs`, `EmitEvents`, `MeshSend`, `MeshBroadcast`, `LogProgress`, `StepDone`, `AdminDaemon`, `Full`
- `Full` bypasses all scope checks (`apps/shipd/src/scopes.rs:34-37`)
- Role mapping: worker gets 6 scopes, lead gets 8, commander gets 9, contributor gets `Full` (`apps/shipd/src/scopes.rs:41-74`)

### 1.4 Browser Session Cookie (`ship_session` / `ship_browser_*`)

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/session.rs:28-41` (constants, generation), `apps/shipd/src/session.rs:63-87` (issuance), `apps/shipd/src/session.rs:118-148` (handler) |
| **Identity asserted** | Browser client (Studio). Agent ID hardcoded to `"browser"` (`apps/shipd/src/session.rs:78`). |
| **Issuer** | Daemon via `POST /api/runtime/session` (requires `Scope::Full` caller — `apps/shipd/src/session.rs:123`) or `mint_cookie()` in `apps/shipd/src/views_raw.rs:39-48`. |
| **Consumer** | Browser. Cookie set as `HttpOnly; SameSite=Strict; Path=/api/runtime` (`apps/shipd/src/session.rs:130-131`). Auth middleware at `apps/shipd/src/auth.rs:164-187` checks cookie before Authorization header. |
| **Rotation / expiry** | TTL: 24 hours (`BROWSER_SESSION_TTL_SECS = 86_400` at `apps/shipd/src/session.rs:31`). Ephemeral — not persisted to KV (`apps/shipd/src/session.rs:84`). Expired entries lazily evicted on lookup (`apps/shipd/src/session.rs:91-104`). |
| **Failure mode** | `401 Unauthorized "invalid or expired session cookie"` (`apps/shipd/src/auth.rs:181`). |

### 1.5 PKCE Pairing Flow (Code + Claim Token + Paired Session)

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/pairing.rs:1-22` (flow doc), `apps/shipd/src/auth.rs:350-502` (HTTP handlers) |
| **Identity asserted** | Machine owner — must have filesystem access to `daemon.token` to run `ship auth pair <code>` in terminal |
| **Issuer** | Daemon. `POST /auth/pair/init` (unauthenticated, `apps/shipd/src/auth.rs:355`) creates code + claim_token. `POST /api/auth/pair/confirm` (requires `Scope::Full`, `apps/shipd/src/auth.rs:407`) issues session token. |
| **Consumer** | Studio browser client. Polls `GET /auth/pair/status?claim_token=...` (unauthenticated, `apps/shipd/src/auth.rs:489`). |
| **Rotation / expiry** | Code TTL: 5 minutes (`apps/shipd/src/pairing.rs:45`). Claim TTL: 30 minutes (`apps/shipd/src/pairing.rs:48`). Session TTL: 30 days (`apps/shipd/src/pairing.rs:51`). Codes are one-shot — consumed on confirm (`apps/shipd/src/pairing.rs:150-151`). Max 100 pending pairings (`apps/shipd/src/pairing.rs:55`). |
| **Failure mode** | `404` unknown code, `410` expired code, `409` already approved (`apps/shipd/src/auth.rs:462-484`). Rate-limited to 100 pending (`apps/shipd/src/pairing.rs:111-112`, HTTP `429`). |

**Dev-mode relaxation**: When `--dev` flag is set (`apps/shipd/src/pairing.rs:34-42`), `pair_init` auto-approves with an ephemeral session (`apps/shipd/src/auth.rs:373-395`). No CLI confirmation required. Agent ID: `"studio-dev"`.

### 1.6 Server-Side Attestation (JWT / EdDSA via `attest.rs`)

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/attest.rs:1-6` (module doc), `apps/shipd/src/auth.rs:560-639` (`server_attest` handler) |
| **Identity asserted** | Authenticated user on getship.dev. JWT `sub` claim = user ID. |
| **Issuer** | getship.dev issues the JWT. Daemon verifies via JWKS at `{issuer}/api/jwks` (`apps/shipd/src/attest.rs:43-48`). Validation: EdDSA/Ed25519 only, audience `"ship-daemon"`, issuer match (`apps/shipd/src/attest.rs:127-129`). |
| **Consumer** | Studio web client via `PlatformProvider.tsx` (`apps/web/src/features/studio/PlatformProvider.tsx:296-321`). Calls `POST /api/auth/daemon-attest` on the web backend, which returns a JWT, then `POST /auth/server-attest` on the daemon. |
| **Rotation / expiry** | Session bound to JWT's `exp` claim (`apps/shipd/src/auth.rs:624`). JWKS cached for 1 hour (`apps/shipd/src/attest.rs:19`), auto-refetched on signature failure for mid-rotation handling (`apps/shipd/src/attest.rs:93-98`). |
| **Failure mode** | `401` on malformed/expired/bad-signature JWT. `502` if JWKS endpoint unreachable. `403` if daemon already bound to a different user (`apps/shipd/src/auth.rs:587-593`). |

**Owner binding**: First successful attestation binds the daemon to a user ID via KV (`apps/shipd/src/attest.rs:207-222`). Subsequent attestations from different users are rejected with `403`.

### 1.7 View-Session Ownership (`view_id` Scoped to Agent)

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/view_sessions_api.rs:1-15` (module doc), `apps/shipd/src/view_sessions_api.rs:48-56` (`require_agent`), `apps/shipd/src/view_sessions_api.rs:86-113` (open), `apps/shipd/src/view_sessions_api.rs:115-126` (patch) |
| **Identity asserted** | Agent ID from `AuthContext.agent_id`. Every mutating route requires an authenticated agent. |
| **Issuer** | Daemon creates view sessions; ownership derived from the caller's `AuthContext.agent_id`. |
| **Consumer** | MCP agents (via tool calls like `view_open`, `view_patch`). The `require_agent` helper at `apps/shipd/src/view_sessions_api.rs:48` gates all routes. |
| **Rotation / expiry** | No explicit TTL on view sessions. They exist in memory until explicitly closed via `DELETE /api/runtime/view-sessions/{id}` (`apps/shipd/src/view_sessions_api.rs:154-164`). |
| **Failure mode** | `401` "view-session routes require an authenticated agent id" if no `agent_id` (`apps/shipd/src/view_sessions_api.rs:50-55`). `403` NotOwner on mutating routes for non-owning agents. `404` NotFound for unknown session IDs. |

### 1.8 Annotation Event Auth (`POST /api/runtime/annotations`)

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/annotations_api.rs:1-14` (module doc), `apps/shipd/src/annotations_api.rs:97-151` (handler) |
| **Identity asserted** | Caller's `AuthContext` (daemon bearer or session token). Agent vs human distinguished by `author_kind` field in the payload. |
| **Issuer** | Standard auth middleware (daemon token, session token, or browser cookie). |
| **Consumer** | Studio (human annotations) and MCP agents (automated annotations). Routed through kernel event bus via `auth.to_daemon_emit_context()` (`apps/shipd/src/annotations_api.rs:164`). |
| **Rotation / expiry** | Inherits caller's session TTL. Annotation payloads have 60s id-dedupe window (`apps/shipd/src/annotations_api.rs:49`). |
| **Failure mode** | `429 Too Many Requests` for agent callers exceeding 10/sec bucket (`apps/shipd/src/annotations_api.rs:112-118`). `422` for duplicate IDs, invalid ULID, payload size violations (`apps/shipd/src/annotations_api.rs:176-236`). `404` for unknown view_id targets. |

### 1.9 MCP Apps CSP (`_meta.ui.csp`)

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/mcp_apps_bridge.rs:23-25` (`IFRAME_CSP` constant), `apps/shipd/src/mcp_apps_bridge.rs:41-55` (`view_session_meta`) |
| **Identity asserted** | N/A — CSP is a browser-enforced isolation boundary, not identity. |
| **Issuer** | Daemon, via the `IFRAME_CSP` string applied as `Content-Security-Policy` header on iframe shell responses (`apps/shipd/src/view_sessions_api.rs:211`) and echoed in `_meta.ui.csp` for MCP Apps hosts. |
| **Consumer** | Browser rendering engine. Constrains what scripts/connections can execute inside view iframes. |
| **Rotation / expiry** | Static policy. Changes require a code deploy. |
| **Failure mode** | Browser silently blocks disallowed resources. `script-src 'self'` prevents XSS via injected scripts. No `'unsafe-eval'` or wildcard sources (`apps/shipd/src/mcp_apps_bridge.rs:24`). Path traversal in `parse_view_session_uri` blocked by alphanumeric check (`apps/shipd/src/mcp_apps_bridge.rs:63`). |

### 1.10 `--dev` Mode Relaxations

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/pairing.rs:34-42` (global flag), `apps/shipd/src/auth.rs:362-395` (auto-pair logic) |
| **Identity asserted** | Weakened — no CLI confirmation needed. Auto-issued session gets `Scope::Full`. |
| **Issuer** | Daemon when started with `--dev` flag. `set_dev_mode(true)` called at startup. |
| **Consumer** | Any browser calling `POST /auth/pair/init`. Immediately receives a session token. |
| **Rotation / expiry** | Dev sessions are ephemeral (in-memory only, `apps/shipd/src/auth.rs:389`). Die on daemon restart. |
| **Failure mode** | No failure — that's the relaxation. Any localhost process can get `Scope::Full` without human confirmation. Logged as `tracing::warn!` (`apps/shipd/src/auth.rs:392`). |

### 1.11 Hook 401 Token Rotation on Daemon Restart

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/auth.rs:74-78` (daemon token regeneration), `apps/web/src/features/studio/PlatformProvider.tsx:209-213` (auth failure handler) |
| **Identity asserted** | N/A — this is a recovery mechanism, not an identity assertion. |
| **Issuer** | Daemon regenerates `daemon.token` on restart. All prior session tokens (in-memory, ephemeral) are lost. |
| **Consumer** | Studio detects 401/403 via `setAuthFailureHandler` (`apps/web/src/features/studio/PlatformProvider.tsx:211`), drops cached token from `localStorage`, re-initiates pairing (`apps/web/src/features/studio/PlatformProvider.tsx:283-292`). |
| **Rotation / expiry** | Immediate on restart. Non-ephemeral sessions restored from KV via `hydrate_sessions()` (`apps/shipd/src/scopes.rs:159-202`). |
| **Failure mode** | Brief auth disruption. Studio re-pairs automatically. CLI reads new `daemon.token` from disk on next command. MCP processes must re-register. |

### 1.12 `enforce_mode_tool_gate` (Role-Based Tool Gating)

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/mcp/src/server/tool_gate.rs:100-144` (main enforcement), `apps/mcp/src/server/tool_gate.rs:22-26` (core tools), `apps/mcp/src/server/tool_gate.rs:58-71` (role-based defaults) |
| **Identity asserted** | Active agent's role, resolved from the agent profile in the project dir. |
| **Issuer** | Project configuration (`.ship/agents/*.yaml`). Agent profiles declare `role` and optional `active_tools`. |
| **Consumer** | MCP server. Calls `enforce_mode_tool_gate` before executing any tool. Integration tests at `apps/mcp/src/role_gate_integration_tests.rs:1-310` validate all role permutations. |
| **Rotation / expiry** | Static — changes when agent profile is edited. No TTL. |
| **Failure mode** | Returns `Err(String)` with message "Tool 'X' blocked by active mode 'Y' (role 'Z' allows: ...)" (`apps/mcp/src/server/tool_gate.rs:115-136`). MCP server surfaces this as a tool error to the agent. |

**Core tools always bypass** (`apps/mcp/src/server/tool_gate.rs:22-26`): `create_workspace`, `complete_workspace`, `set_skill_var`. Under `unstable` feature: `event`.

### 1.13 Event Delivery Scope Filtering (Global/Workspace/Directed/Elevated)

| Attribute | Detail |
|-----------|--------|
| **Source** | `crates/core/runtime/src/events/permissions.rs:22-34` (`DeliveryScope` enum), `crates/core/runtime/src/events/kernel_security.rs:110-140` (`event_matches_actor_scope`) |
| **Identity asserted** | Actor identity bound at spawn time via `ActorIdentity`. Workspace binding via `spawn_actor_with_permissions_in_workspace` (`crates/core/runtime/src/events/kernel_security.rs:35-42`). |
| **Issuer** | Kernel router. Permissions declared at actor spawn (`crates/core/runtime/src/events/kernel_security.rs:27-31`). |
| **Consumer** | Kernel router's `deliver_with_scope()` (`crates/core/runtime/src/events/kernel_security.rs:179-256`). |
| **Rotation / expiry** | Static for the actor's lifetime. No dynamic permission changes. |
| **Failure mode** | Events silently not delivered to actors that don't match scope. Mailbox overflow triggers `actor.mailbox_full` kernel event (`crates/core/runtime/src/events/kernel_security.rs:259-277`). |

### 1.14 Reserved Event Namespace Prefix Enforcement

| Attribute | Detail |
|-----------|--------|
| **Source** | `crates/core/runtime/src/events/validator.rs:83-97` (`RESERVED_NAMESPACES`), `apps/mcp/src/tools/event.rs:24-31` (MCP enforcement) |
| **Identity asserted** | Caller kind (CLI, MCP, Runtime, SDK). Only `CallerKind::Cli` and `CallerKind::Runtime` can emit to reserved namespaces. |
| **Issuer** | Compile-time constant list. 13 reserved prefixes: `actor.`, `config.`, `gate.`, `job.`, `mesh.`, `project.`, `runtime.`, `session.`, `skill.`, `studio.`, `supervisor.`, `sync.`, `workspace.` |
| **Consumer** | MCP `handle_ship_event()` (`apps/mcp/src/tools/event.rs:8-41`) checks before envelope construction. Kernel's `enforce_emit_permissions()` (`crates/core/runtime/src/events/kernel_security.rs:143-167`) checks at route time. |
| **Rotation / expiry** | Static — compile-time constant. |
| **Failure mode** | MCP tool returns error "event_type 'X' is reserved: 'Y' prefix is platform-controlled" (`apps/mcp/src/tools/event.rs:26-30`). Kernel returns `anyhow` error "actor 'X' not permitted to emit 'Y'" (`crates/core/runtime/src/events/kernel_security.rs:158-162`). |

### 1.15 Annotation Rate Limiting (Per-Agent Token Bucket)

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/annotations_api.rs:36-93` (`AnnotationGuard`), `apps/shipd/src/annotations_api.rs:110-118` (enforcement in handler) |
| **Identity asserted** | Agent identity from `AuthContext.agent_id`. Falls back to `"anonymous-agent"` for agents without an ID (`apps/shipd/src/annotations_api.rs:111`). |
| **Issuer** | Daemon-scoped `AnnotationGuard` instance (one per daemon). |
| **Consumer** | `post_annotation` handler. Only applied to `AuthorKind::Agent` callers — humans are exempt (`apps/shipd/src/annotations_api.rs:110`). |
| **Rotation / expiry** | Token bucket: capacity 10, refill 10/sec (`apps/shipd/src/annotations_api.rs:52-53`). Dedupe window: 60 seconds (`apps/shipd/src/annotations_api.rs:49`). |
| **Failure mode** | `429 Too Many Requests` "agent annotation rate limit exceeded (10/sec)" (`apps/shipd/src/annotations_api.rs:115`). `422` for duplicate IDs within window (`apps/shipd/src/annotations_api.rs:122`). |

### 1.16 (Additionally Discovered) SDK Token Auth (`ship_*` prefix, non-session)

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/auth.rs:270-276` |
| **Identity asserted** | External SDK caller. CallerKind set to `Sdk`. |
| **Issuer** | Same as daemon token — validated against `expected_token` (`apps/shipd/src/auth.rs:266`). |
| **Consumer** | External SDK integrations using `ship_` prefixed tokens. |
| **Rotation / expiry** | Same as daemon token — regenerated on restart. |
| **Failure mode** | `401 "invalid token"`. Gets `Scope::Full` on success. |

### 1.17 (Additionally Discovered) Query-Parameter Token Auth (GET-only)

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/auth.rs:200-209`, `apps/web/src/lib/daemon-config.ts:30-35` |
| **Identity asserted** | Same as bearer — the token itself is the identity. |
| **Issuer** | Client-side. `daemonUrlWithToken()` (`apps/web/src/lib/daemon-config.ts:30`) appends `?token=...` for browser APIs that can't set headers (EventSource, WebSocket). |
| **Consumer** | Auth middleware. Only accepted on GET requests — POST/PUT/DELETE must use Authorization header to prevent token leakage into server logs (`apps/shipd/src/auth.rs:199`). |
| **Rotation / expiry** | Same as the underlying token. |
| **Failure mode** | Silently ignored on non-GET methods; falls through to header check. |

### 1.18 (Additionally Discovered) Audit Log

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/shipd/src/auth.rs:504-526` |
| **Identity asserted** | Records token prefix (first 16 chars) + agent_id on pair_confirmed, server_attest, and pair_revoked events. |
| **Issuer** | Daemon, written to `platform.db` KV under `auth.audit` namespace. |
| **Consumer** | Operators for forensic reconstruction. |
| **Rotation / expiry** | No TTL on audit entries. Grows unboundedly. |
| **Failure mode** | Write failures logged as `tracing::warn!` — non-blocking (`apps/shipd/src/auth.rs:523-525`). |

### 1.19 (Additionally Discovered) Web Frontend Token Lifecycle

| Attribute | Detail |
|-----------|--------|
| **Source** | `apps/web/src/features/studio/PlatformProvider.tsx:68-78` (constants), `apps/web/src/features/studio/PlatformProvider.tsx:249-378` (init lifecycle) |
| **Identity asserted** | Browser session — Studio user. |
| **Issuer** | PlatformProvider orchestrates: (1) fragment token from `#token=...`, (2) cached `localStorage` session token, (3) server attestation via getship.dev, (4) PKCE pairing fallback. |
| **Consumer** | `ShipClient` instance. Token set via `client.setToken()` and `setDaemonToken()` (`apps/web/src/lib/daemon-config.ts:8-9`). |
| **Rotation / expiry** | On 401/403, `setAuthFailureHandler` (`apps/web/src/features/studio/PlatformProvider.tsx:211`) clears `localStorage` and re-pairs. |
| **Failure mode** | Falls through priority chain: fragment > cached > attest > pair. Stale cached token triggers re-pairing (`apps/web/src/features/studio/PlatformProvider.tsx:268-277`). |

---

## 2. Drift and Gap Analysis

### Gap 1: MCP Session Tokens Never Expire (Severity: **major**)

MCP session tokens (`ship_sess_*`) have `expires_at_unix: None` and are ephemeral (`apps/shipd/src/scopes.rs:284-285`). While they die on daemon restart, a long-running daemon could accumulate sessions from crashed MCP processes with no garbage collection. A compromised token is valid indefinitely within a daemon lifetime.

**Recommendation**: Add a configurable TTL (e.g., 24h) to MCP sessions, with a refresh mechanism on heartbeat. Implement periodic GC to evict sessions whose MCP process is no longer connected.

### Gap 2: Hook Token Never Auto-Rotates (Severity: **major**)

The hook token (`ship_hook_*`) is generated once and never rotated (`apps/shipd/src/auth.rs:106-107`). It survives daemon restarts (persisted to disk), unlike the daemon token. If leaked, it provides permanent `EmitEvents + LogProgress` access until manual intervention.

**Recommendation**: Auto-rotate hook tokens on daemon restart or on a configurable schedule. Provide a `ship auth rotate-hook` CLI command. Log rotation events to the audit log.

### Gap 3: Browser Session and Paired Session TTL Mismatch (Severity: **major**)

Browser sessions (`ship_browser_*`) expire in 24 hours (`apps/shipd/src/session.rs:31`), but paired sessions from the PKCE flow expire in 30 days (`apps/shipd/src/pairing.rs:51`). A user who pairs once retains `Scope::Full` access for 30 days without re-verification. Meanwhile, direct browser sessions require re-issuance every 24 hours.

**Recommendation**: Align paired session TTL with a shorter window (e.g., 7 days) or add a refresh-token pattern. At minimum, provide a daemon-side session listing + revocation UI so users can audit their active sessions.

### Gap 4: `--dev` Mode Has No Safety Guard Beyond a Flag (Severity: **blocker**)

When `--dev` is set, any localhost process gets `Scope::Full` without human confirmation (`apps/shipd/src/auth.rs:373-395`). The flag is a global `AtomicBool` (`apps/shipd/src/pairing.rs:34`) with no additional safeguards — no "dev mode expires after N hours", no env-var interlock, no confirmation prompt.

**Recommendation**: Add a secondary guard: require an env var `SHIP_DEV_MODE=1` alongside the flag, or auto-disable after 8 hours, or restrict dev-mode sessions to `ReadWorkspace` scope instead of `Full`.

### Gap 5: `Scope::Full` Granted to Contributors Without Limitation (Severity: **minor**)

`scopes_for_role("contributor")` returns `[Scope::Full]` (`apps/shipd/src/scopes.rs:63`), giving contributors the same power as the daemon token itself — including `AdminDaemon`. This is intentional per the doc comment, but it means a contributor agent could list/revoke other sessions, manage jobs, or emit to any namespace.

**Recommendation**: Consider a `ContributorFull` scope that grants everything except `AdminDaemon`, or document the trust assumption explicitly in the agent profile schema.

### Gap 6: Audit Log Has No TTL or Size Bound (Severity: **minor**)

Auth audit entries in KV (`apps/shipd/src/auth.rs:504-526`) grow unboundedly. There's no GC, no max entries, and no TTL.

**Recommendation**: Add a TTL (e.g., 90 days) or max-entries cap to the audit namespace. Rotate on daemon startup.

### Gap 7: View Session Raw Endpoint Mints Cookie Without CSRF Protection (Severity: **minor**)

`serve_view_raw` (`apps/shipd/src/views_raw.rs:122-146`) unconditionally mints a browser session cookie on every request. While the cookie is `HttpOnly; SameSite=Strict`, the endpoint itself has no CSRF token. A malicious page could trick a browser into fetching this endpoint, though `SameSite=Strict` mitigates most practical attacks.

**Recommendation**: The `SameSite=Strict` policy is sufficient for current threat model (localhost daemon). If the daemon ever becomes network-accessible, add explicit CSRF tokens.

### Gap 8: SDK Token Gets Full Scope With No Distinct Identity (Severity: **minor**)

Tokens starting with `ship_` that aren't session tokens are validated against the daemon token and get `Scope::Full` with `CallerKind::Sdk` (`apps/shipd/src/auth.rs:270-276`). However, the logic at line 270 checks `is_daemon_token(token)` first (64-char hex check), and `ship_` prefixed tokens would also need to match `expected_token`. This means SDK tokens are effectively the same as the daemon token — there's no separate SDK credential.

**Recommendation**: If SDK callers need distinct identity, issue them scoped session tokens via a dedicated registration endpoint (analogous to MCP session registration).

---

## 3. Unification Proposal

Ordered by blast radius (highest first):

### Move 1: Unified Session Lifecycle with Consistent TTL

**Done predicate**: All non-daemon-token sessions (MCP, browser, paired, attested) have an explicit `expires_at_unix`, a configurable but consistent default TTL, and a GC sweep that runs on a timer (not just lazy eviction).

**Blast radius**: High — touches `scopes.rs`, `session.rs`, `pairing.rs`, `auth.rs`. Every session consumer needs to handle refresh or re-auth.

### Move 2: Dev-Mode Safety Interlock

**Done predicate**: `--dev` mode requires a secondary confirmation (env var or prompt) and auto-disables after a configurable timeout. Dev-mode sessions are logged to the audit trail.

**Blast radius**: High — affects developer workflow. Must not break `ship dev` ergonomics.

### Move 3: Hook Token Rotation on Daemon Restart

**Done predicate**: `ensure_hook_token()` regenerates the hook token on daemon restart (not just on first creation). Old token files are overwritten. Hook callers receive a `401` and must re-read the token file.

**Blast radius**: Medium — breaks existing hook integrations on restart. Requires documentation and possibly a grace period (accept old token for N seconds).

### Move 4: Scope Model Refinement — Separate `ContributorFull` from `AdminDaemon`

**Done predicate**: `Scope::Full` is split into `Scope::Full` (all except admin) and `Scope::AdminFull` (everything). Contributors get `Full`, only the daemon token gets `AdminFull`. `list_sessions`, `revoke_sessions` require `AdminFull`.

**Blast radius**: Medium — requires audit of all `Scope::Full` checks.

### Move 5: Audit Log Retention Policy

**Done predicate**: Auth audit entries are capped at 10,000 entries or 90 days (whichever is smaller). GC runs on daemon startup.

**Blast radius**: Low — no API changes.

### Move 6: Deprecate Query-Parameter Token Path

**Done predicate**: Query-parameter tokens are removed. EventSource and WebSocket connections use cookie-based auth (already available via `ship_session`).

**Blast radius**: Low — only affects legacy browser paths. Studio already uses cookie auth for views.

---

## 4. Follow-up Spec Stubs

### Stub 1: Unified Session TTL and GC

**Job title**: `feat(shipd): unified session lifecycle with TTL and GC`

**Scope**: Add `expires_at_unix` to all session types. Implement a background `tokio::time::interval` sweep in the daemon that evicts expired sessions from the registry and KV. Add a `session.ttl_seconds` config key. Update `session_register` to accept an optional TTL override.

**Acceptance criteria**:
- All session types (MCP, browser, paired, attested) have non-None `expires_at_unix`
- GC sweep runs every 60s and removes expired entries
- `cargo test -p shipd` passes with new TTL/GC tests
- No existing tests regress

### Stub 2: Dev-Mode Safety Interlock

**Job title**: `security(shipd): dev-mode safety interlock`

**Scope**: Require `SHIP_DEV_MODE=1` env var in addition to `--dev` flag. Add auto-disable after 8 hours (configurable). Log all dev-mode sessions to audit trail.

**Acceptance criteria**:
- `--dev` without `SHIP_DEV_MODE=1` prints a warning and exits
- After 8 hours (or configured TTL), dev-mode auto-disables and new pair requests require CLI confirmation
- Audit log records dev-mode session creation events
- `cargo test -p shipd` passes

### Stub 3: Hook Token Rotation

**Job title**: `feat(shipd): auto-rotate hook token on daemon restart`

**Scope**: Modify `ensure_hook_token()` to always regenerate on daemon startup (not just first creation). Add `ship auth rotate-hook` CLI command for manual rotation. Add audit log entry for hook token rotation.

**Acceptance criteria**:
- Hook token changes on every daemon restart
- `ship auth rotate-hook` regenerates the token and prints confirmation
- Audit log records `hook_rotated` events
- Existing hook callers get `401` and must re-read the token file
