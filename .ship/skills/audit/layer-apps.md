# Layer Apps — Audit Shape

Apps layer covers `apps/shipd`, `apps/ship-cli`, `apps/gateway`, `apps/workers`,
`apps/shell`, and similar binaries and cloud workers. The defining question at
this layer is **boundary purity**: does the app consume L0/L1 only through the
SDK/service layer, or does it reach through boundaries to raw primitives?
Audits at this layer are boundary-purity-centric and security-first.

## What apps audits cover

- **SDK-boundary purity** — imports from L0/L1 that bypass the service layer
  are `[purity-violation]` findings.
- **Transport-vs-logic** — RPC/HTTP handlers should be thin transport (validate
  input, call service, map result to proto/JSON). Business logic in handlers is
  a `[logic-in-transport]` finding.
- **Auth surface** — every external-facing endpoint must have a declared auth
  mechanism. Endpoints with no auth check are security findings.
- **Privileged-ops inventory** — enumerate C1/C2/C3 operations with their blast
  radius and the auth check that gates them.
- **File-cap discipline** — apps tend to accumulate large files. Flag hard-cap
  violations (>400 LOC, non-test) as `[file-cap: hard-violation]`.

## Section shape (10 sections)

### 0. Security findings summary (apps only)
Place at the top, before the file inventory. List all SF-N findings with
one-line summaries and severity labels. Security findings are first-class output
at the apps layer. If there are none, say so explicitly.

Format:
```
### SF-1 — <short title> — CRITICAL / CONCERNING / NOTABLE / POTENTIAL DRIFT
<one-paragraph description with file:line citation, blast-radius assessment,
and current tracking status>
```

### 1. File inventory
Two-level summary: (a) aggregate totals (`Total .rs files`, `Total LOC`,
`Production files`, `Sub-module count`), (b) sub-module tree. Top-N largest
non-vendor source files with LOC and cap status. File-cap violations table
(hard >400, soft >300, non-test, non-vendor).

### 2. Binary / package role + entry surface
What does the app binary/package do? What are its deployment targets (daemon on
host, Cloudflare Worker, Tauri shell, CLI)? Connect services exposed. REST routes
(note if REST is being retired in favor of Connect). DO RPC surfaces for workers.

### 3. SDK-boundary (auth / purity)

**For Rust apps:**
- Which L0/L1 crates are imported directly vs through the service layer?
- Imports of `ship_substrate::db::open_db` or `ship_kernel::*` in business
  logic files are `[purity-violation]` findings.
- `CallerKind::Runtime` minted outside substrate-internal lifecycle code is
  `[candidate: runtime-caller-mint-in-app]` — each site must be intentional.

**For TypeScript cloud apps (gateway, workers):**
- Are Rust crates imported? (Should be zero for cloud TS.)
- Shared code with other packages — use only `@ship-cloud/types` workspace
  package for event constants; no path imports into `crates/`.
- `@ship-cloud/types` keeping pace with daemon event vocabulary — check for
  `[event-type-drift]`.

### 4. Service surface
Enumerate all external-facing surfaces:
- Connect-RPC services: list proto services mounted, file:line of registration.
- REST routes (legacy): note if each is being retired.
- DO RPC surfaces: facade methods on each Durable Object.
- SDK skill entry points: if the app exposes skills.

### 5. Auth surface (CRITICAL for public-internet-facing apps)
One sub-section per auth mechanism:
- Token / credential handling (where are secrets, who mints tokens, what scope)
- Session auth (JWT validation, who enforces, where)
- Middleware ordering (auth before business logic?)
- Rate limiting (where enforced — gateway, middleware, DO layer?)
- CORS / TLS config

For DO-based workers: auth is delegated to the gateway by design. State this
explicitly and cite the `script_name` binding that makes the worker private.

### 6. Privileged-ops inventory
Table: `Operation | file:line | Auth at this layer | Auth at caller | Blast radius`.

Blast-radius labels follow the collection vocabulary:
- `C0_READ` — read-only, no state change
- `C1_WORKSPACE` — modifies workspace config / state
- `C2_ELEVATED` — elevated privilege (daemon token, operator auth)
- `C3_CRITICAL` — irreversible / cross-tenant operations

### 7. Layer-direction
For Rust apps: enumerate direct imports of each L0/L1 crate. Note which are
correct (consuming through service layer) vs violations (reaching raw L0 primitives
in handler code). Common violations: `ship_substrate::db::open_db` in RPC handler,
`ship_kernel::EventEnvelope` constructed in business logic.

For TS apps: confirm zero path imports into `crates/`. Check `package.json` deps.

### 8. Cross-cutting observations
Common apps-layer tags:
- `[purity-violation]` — direct L0/L1 import bypassing service layer
- `[file-cap: hard-violation]` — production file >400 LOC (list file + LOC)
- `[logic-in-transport]` — business logic in RPC handler, not in service
- `[event-type-drift]` — cloud app listens for event strings the daemon no longer emits
- `[minimal-auth-surface]` — DO workers correctly delegate auth entirely to gateway
- `[symmetric-DO-pair]` — two DOs with ~90% structural duplication (refactor candidate)
- `[generated-code-untracked]` — generated TS proto files not tracked in git
- `[arc-vs-reality gap]` — module-head doc inaccurately describes the module's role

### 9. Open calls for sign-off
Common apps-layer open calls:
- DO RPC entry points with no auth check at the DO boundary (correct if binding
  is `script_name` private; risk if model changes) — requires user decision on
  whether to add DO-level guards.
- First-writer-wins membership patterns with no secondary identity verification.
- Event-type drift: which event family should the projection track, and is it
  currently tracking the right one?
- Role-check responsibility — who enforces role checks for privileged membership
  operations?
- Observability coverage — are DO RPC invocations captured by tail workers /
  analytics engine?

### 10. Summary
One paragraph: overall boundary purity verdict, security posture, outstanding
items. Cite the number of SF-N findings, hard-cap violations, and open calls.
Compare to baseline modules (e.g., "workers has 0 cap violations — cleanest in
the apps layer; contrast with shipd's 15 hard violations").

## Apps-specific checks

### `CallerKind::Runtime` minting
`CallerKind::Runtime` bypasses all namespace enforcement in `KernelRouter::enforce_write_namespaces`.
This is intentional for the daemon's own lifecycle events but must be audited
wherever it appears in app-layer code. Enumerate every site.

### Daemon scope ladder
`Scope::Full` confers operator authority through `gate::decide`. Verify:
- Daemon token mints `Scope::Full` only — is this appropriate? (Open: #86)
- SDK tokens: what scope do they receive? (Open: #207)
- Agent tokens: scoped to declared capability set?

### Connect vs REST migration state
If the app has both a Connect-RPC service and REST endpoints, note which REST
routes are already retired (return 405) vs still live. Track as
`[dual-wire-surface]` if both are load-bearing.

### File-cap discipline for large apps
Apps like `shipd` (51K LOC, 253 files) have extensive cap violations. For each
audit of a large app:
- Count hard violations (>400 LOC, non-test, non-vendor).
- Count soft violations (>300 LOC, non-test, non-vendor).
- Identify the top-5 largest violating files as split candidates.
Tag aggregate as `[file-cap: hard-violation]` with count in the description.
