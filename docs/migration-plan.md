# Plan: Aligning the Codebase with Ship's Story

This is the migration from current state to target state. Current state is documented in [the architecture audit](../.artifacts/audit/architecture-map.md). Target state is described in [`what-is-ship.md`](what-is-ship.md). The plan below sequences the gap, with v1 launch criteria defined and post-v1 work explicitly deferred.

## v1 launch criteria

The wedge works:

- Open Ship → describe app in plain language → first-party agent scaffolds the components → it runs in five minutes
- The generated app has real data, real agents, real interactivity. Not 2D.
- User edits any component; hot-reload reflects it. User asks the agent to extend; new components appear.
- Solo experience is rock-solid. Cloud sync is available for teams.
- First-party views are dogfood-grade, authored as content the same way third-party views are.

If first-party views can't be authored as content, the wedge agent can't either. First-party migration is the proof.

## Gap analysis

**Stays as-is:**

- `crates/core/ship-kernel` — clean, the kernel substrate
- Runtime substrate (`runtime/{events, persistence, db, actor, supervisor, projections, sessions, mesh}`) — the L0 spine
- `crates/core/compiler` — pure crate; consumed by `ship-cli`, not the daemon
- Auth — already in place; wires to project sync in Phase 7
- Much of the `cloud/` Worker code

**Changes:**

- L1 services emit `<domain>.changed` events on every config-file write (the sync substrate)
- KV extracted from `platform.db` into a separate durable store
- The runtime's parallel provider-emit (`runtime/agents/export/`) deleted (Phase 1 ✓)
- First-party views (`apps/shipd/src/services/{chat,settings,specs}` + matching `views/`) migrate from Rust to content
- Workspace residue (URL space, `MeshRegisterReq.workspace_id`, `actor.created.workspace_id`) eliminated; `/workspaces/{id}/...` becomes `/projects/{id}/...`
- Service-registry naming rationalized (per audit §7 row 1)
- Git split into storage adapter (one of three: filesystem, git, cloud) and execution-context substrate (worktrees)

**New:**

- `KvService` trait + durable separate KV store (backend TBD: sled or separate sqlite)
- Cloud sync substrate over event-log replication
- The first-party generator agent + its scaffolding tools — the v1 launch deliverable
- Project switcher in the Studio shell

## Phases

### Phase 0 — Lock the story

**Output:** [`docs/what-is-ship.md`](what-is-ship.md) (done), this plan, and a charter ADR documenting the "registered exceptions" — explicitly stating which storage tiers are file-canonical and why event-sourcing is not dogma.

### Phase 1 — Eject the parallel provider emit ✓

Deleted `crates/core/runtime/src/agents/export/` (~5,500 LOC). Migrated to compiler:

- `.mcp.json` merge now preserves user-authored entries via `_ship.managed` marker (Z)
- Codex Starlark execpolicy generation added to compiler (X)
- Session-start compile replaced with `ship compile` subprocess — best-effort, soft-warn (V)
- Provider registry moved to `runtime::agents::providers`; import functions to `runtime::agents::import`
- `ship compile` is now the single emit path

### Phase 2 — Substrate split (~2–3 weeks) [in progress]

Carve `crates/core/runtime/` into two crates:

- `ship-substrate` (L0) — events, persistence, db, actor, supervisor, projections, mesh, sessions, kernel-router. Knows nothing about agents, skills, etc.
- `ship-primitives` (L1) — agents, collections, hooks-engine, artifacts, package-registry. Imports `ship-substrate`. Imports nothing else.

Compile-time barrier enforces what charter currently only asserts about layers. Substrate split is early because every later phase benefits from the boundary already existing.

**Phase 2A done**: `crates/core/ship-substrate` created and compiles. Moved: events, persistence, db, actor, projections, supervisor, home, project (L0 surface), git, log, fs_util, env, errors, data, projects, trash, access, services/{mesh, dispatch_ports, human_gateway, process_tree_probe, server_session}. 385 tests pass. `runtime` now depends on `ship-substrate` and re-exports everything. Three L1 leaks repaired: `home/daemon.rs` port-pin (inline struct), `project/identity.rs` JSONC parse (use fs_util), `data.rs` startup hook (no-op in substrate).

**Phase 2B done**: `runtime` rewritten as a re-export shim over `ship-substrate`. 688 tests pass.

**Phase 2C done** (current): `crates/core/ship-primitives` created. Moved from runtime: agents, artifacts, collections, config, hooks, sync, services/sync. 304 tests pass. `compiler::jsonc::from_jsonc_str` replaced with `serde_json::from_str` + `fs_util::strip_jsonc_comments` (8 call sites). `runtime` now re-exports L1 from ship-primitives + keeps registry, sessions, orchestrator, providers (compiler deps).

**Remaining Phase 2 work**:
- sessions/ still in runtime — `session_lifecycle.rs` uses config hooks and `compiler::list_providers()`
- registry/ still in runtime — uses `compiler::manifest::ShipManifest::from_file`
- services/{orchestrator, providers, provider_adapter} still in runtime — orchestrator uses `compiler::vars::resolve_template`
- All three stall on compiler dependency; move them to ship-primitives once compiler deps are addressed or the calls are inlined

### Phase 3 — KV extraction + KvService (~1–2 weeks)

Move KV out of `platform.db` into its own durable store. `KvService` trait gates writes. Namespaces declared, not ad-hoc. Writes emit `kv.<namespace>.changed` events. Skill vars and runtime config migrate. Backend choice (sled vs separate sqlite) is decided in this phase.

### Phase 4 — Event-derived signals on L1 declarative writes (~3–4 weeks)

Filesystem watcher emits `<domain>.changed` events for every config-file write. Operational state (sessions, jobs, KV, files, collections) is already event-sourced; this phase covers the declarative tier. After this, the event log is the universal sync substrate — every meaningful state change in any storage tier flows through it.

### Phase 5 — First-party views migrated to content (~2–3 weeks)

`chat`, `dashboard`, `settings`, `specs`, `project-switcher` move out of `apps/shipd/src/services/` and `apps/shipd/src/views/` into scaffolded content under `<global-config>/components/`. Likely intersects with the Ship UI rendering branch already in flight; coordinate.

This is where dogfood proves the platform. If first-party views work as content, third-party authoring works. If they don't, the wedge can't work either.

### Phase 6 — The generator agent + wedge polish (~3–4 weeks)

Build/refine the first-party agent that scaffolds Ship apps from natural-language descriptions. The component-authoring path needs to be clean enough that an agent can produce real apps; that's what Phases 1–5 set up.

The agent uses the same primitives and the same authoring flow a user would. It just goes faster.

**This is the v1 launch deliverable. Everything before serves it.**

### Phase 7 — Cloud sync substrate (~4–6 weeks)

Extend the `cloud/` Worker code to accept event-log replication. Project-sync protocol. Wire the existing identity layer to projects. After this, teams can share a project and see live changes from each other.

### Phase 8 — Cleanup + renames (~1 week)

- Workspace URL space → projects (completes the workspace retirement per audit §7.2)
- Naming-clash renames: `runtime::services` → `runtime::service_actors`; `apps/shipd/src/sessions/` → `apps/shipd/src/pty/`; `crates/core/runtime/src/registry/` → `runtime::package_registry/`; `apps/shipd/src/services/registry/` either renamed or collapsed once first-party views are content
- Dead-code deletions: `runtime::actor` (audit §8.1, zero production callers), unbound `Route` variants (audit §5-5.C), `runtime::actor::supervisor` submodule
- **Daemon thin-transport:** `apps/shipd/` becomes a thin HTTP routing layer. All business logic lives in `ship-substrate` and `ship-primitives` (established in Phase 2); `shipd` is reduced to route registration, auth middleware, and startup. Route handlers call into substrate/primitive APIs — no logic inline.
- **Single binary:** Fold `shipd` into `ship` as `ship daemon`. Eliminates the separate install artifact; `ship` spawns its own daemon subprocess. `shipd` crate becomes an internal library rather than a standalone binary.

Cosmetic but quality-affecting. Each rename eliminates a category of confusion.

### Phase 9 — v1 launch

Solo + local-first wedge experience as the primary product. Cloud sync available to teams. First-party views as reference for what good Ship views look like.

## Post-v1

- Triggers as a user-facing primitive (`TriggersService`, HTTP/MCP/view-shape ops)
- Marketplace polish — curated Shop UI, component discovery, ratings
- Cross-project access (explicit affordance for `ship.project("other").collection(...)`-shape calls)
- Browser-fork Studio shell (Ship apps and web apps in the same tab strip)
- Cloud team workflows beyond sync — comments, notifications, permission inheritance

## Risks and decisions still owed

1. **Sync conflict resolution.** CRDT vs LWW vs application-level merge per primitive. Decision needed before Phase 7 starts.
2. **Ship UI branch landing.** Phase 5 depends on the rendering migration being far enough along. If it slips, Phase 5 forks: view-as-content migration on the existing rendering layer first, Ship UI swap as a follow-up.
3. **Multi-active project runtime in v1.** Probably ship single-active and design substrate for multi-active. Decide explicitly so the substrate split (Phase 2) sets up the right namespacing for the future.
5. **KV backend.** sled vs separate sqlite. Bias toward sled for the no-migration property; revisit if its tooling is too thin.

## Sequencing rationale

Substrate split (Phase 2) is early because it informs every later phase — once L0/L1 are separate crates, the KV move (Phase 3), event-derivation (Phase 4), and first-party migration (Phase 5) can't accidentally cross boundaries. The trade-off is heavier reorganization up front; the payoff is each later phase has a smaller surface to reason about.

The generator agent (Phase 6) is sequenced last among the wedge-building phases because it depends on the authoring primitives being clean (Phase 5 proves this) and the runtime being predictable (Phases 1–4). An agent generating apps on a fragile runtime will produce fragile apps.

Cloud sync (Phase 7) follows the wedge because the wedge is local-first by design. Sync is the upgrade path, not the entry path.

Renames (Phase 8) are last because they're easiest with the structural work done — moving code that's already correctly placed costs less than moving code that's mid-refactor.
