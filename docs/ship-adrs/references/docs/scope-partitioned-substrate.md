---
title: "Scope-Partitioned Substrate"
description: "ADR: the event log and its projections are physically partitioned by scope into runtime.db, account.db, and one space.db per mounted space. Each backend has its own kernel router, its own sync channel, its own encryption key."
group: ADRs
order: 30
audience: internal
status: proposed
date: 2026-05-31
supersedes: [home-layout-and-scope-convention, collections-as-data-layer]
tags: [storage, substrate, partition, kernel, sync, security, encryption, scope]
---

# Scope-Partitioned Substrate

## Status — shipped vs. proposed (reconciled 2026-06-03)


> Per-section reconciliation against the landed code. The front-matter still reads
> `proposed`; the rows marked SHIPPED are now reality and are candidates for ratification.

| Section | Status | Note |
|---|---|---|
| §1 Three backends (per-scope physical partition) | **PARTIAL** | The three per-scope event-log files DO exist on disk and are the live emit target (runtime.db, account.db, spaces/<id>/space.db), routed by PartitionedSink on EventScope. BUT (a) the third file is runtime.db, NOT device.db (Device scope renamed to Runtime, System deleted), and (b) blobs and the encryption-key framing in this section are not shipped (see §5). The partition is realized at the SINK over one shared router, not as three router/log instances. |
| §2 Filesystem location: OS-conventional (XDG/AppSupport/APPDATA) via `directories` crate, SHIP_DATA_HOME override | **PROPOSED** | Not shipped. State lives under ~/.ship (home_dir().join(".ship")). No `directories` crate dependency, no SHIP_DATA_HOME, no $XDG_DATA_HOME resolution for the data root. Per-scope dbs land at ~/.ship/runtime.db, ~/.ship/account.db, ~/.ship/spaces/<id>/space.db. |
| §2 machine/ bucket removed | **PROPOSED** | Not shipped. machine/ is alive and load-bearing: MACHINE_DIR/MACHINE_DB_DIR/MACHINE_DAEMON_DIR constants, machine_dir(), daemon_dir()=machine/daemon, and the legacy db resolver still joins machine/db/<name>. |
| §3 Kernel router becomes scope-aware (routes by EventScope to a per-scope backend) | **PARTIAL** | Emit DOES route by EventScope to a per-scope db. BUT the routing key is the ScopeBinding (via enforce_scope stamping scope_key), not ctx.space_id; routing happens at PartitionedSink, not via the device/account/spaces DashMap<SpaceId,Arc<KernelLog>> trio in the §3 code block. One KernelRouter holds one PartitionedSink over one inner ship_kernel router; emit returns Result<()>, not Cursor. The per-scope-router shape is explicitly deferred ('the seam that Phase 2 will replace with a per-scope router'). |
| §4 Sync model maps 1:1 to backends (per-backend cursors, per-log sync wire) | **PROPOSED** | Not shipped. Sync cursors remain per-platform (push:platform:{space_id} / pull:platform:{space_id}) in a single runtime.db cursors table; sync push reads one unified events table filtered by elevated=1 from runtime.db. No per-backend cursor namespace, no per-space-log sync wire, no account.db-specific sync channel. |
| §5 Per-space SQLCipher encryption + OS-keychain keys | **PROPOSED** | Not shipped at all. Per-scope dbs open as plaintext sqlite (only PRAGMA journal_mode=WAL); zero 'sqlcipher' hits in the repo, no keyring/security-framework/secret-service dependency, no ship.device/ship.account.<id>/ship.space.<id> key namespaces. All key-lifecycle guarantees are target-only. |
| §6 Agent process isolation as substrate defense | **PARTIAL** | A real agent isolation system (ship-isolation: tiers None/Process/Container/Vm/Cloud; strategies including SrtWrapper/bubblewrap, Docker, Youki, Lima, Wsl2, AppleContainer, Firecracker) exists and is wired into spawn (loud-fail if a requested strategy has no backend). BUT it is WORKSPACE-confinement, not substrate-protection: the sandbox mounts the work_dir and has zero references to the ship data dir, so it does not hide space.db; Tier-1 (default) does not confine reads; encryption-at-rest row is unmet (§5); and the strategy is resolved from isolation.* settings + OS probes + an enforce flag (off by default), not from per-agent harness capability/grant specs (ActorProfile still: 'No explicit isolation axis on ActorProfile today'). |
| §7 Search becomes naturally per-scope | **PARTIAL** | Per-scope search IS shipped and wired in shipd: this-space opens the active space's backend; Global fans the same scope-blind SearchService across each reachable space (SpaceContext::for_space) with a per-space search.global opt-out gate and merges/re-ranks; hits stamped with space_id + ship://space/<id> deeplink. BUT the 'hub search' surface as described (account.db + per-space manifest-summary table, 'conceptually different surface') is NOT what shipped — Global reuses the identical in-space surface lifted one level, with no account.db aggregation and no manifest-summary read. |
| §8 data.db disappears as a file | **PROPOSED** | Not shipped. data.db is still a first-class production file: Db::Data => "data.db", data_db_path() resolves it, DataService::open creates it at boot, first-run detection keys off its existence, and trigger actions open it. Collection projections are NOT yet folded into per-space space.db only. |
| §8-adjacent: platform.db retired from emit/read path | **SHIPPED** | platform.db is dead on the emit/read path — never created or read by production. The only site that can materialize it is the .exists()-guarded one-time legacy drain at boot, so a fresh install never creates it. PartitionedSink is the live kernel sink; shutdown WAL-flush iterates only the per-scope dbs actually mounted. |
| ScopeBinding as authoritative routing source of truth (supersedes EmitContext space_id) | **SHIPPED** | ScopeBinding is the landed authoritative binding: private fields, sole minters from_session (auth seam) and daemon_internal (trusted internal), key_for(EventScope) returning Space->space_id / Account->account_id / Runtime->device_id. emit stamps author + enforces scope from the binding BEFORE persist; PartitionedSink routes on the stamped (scope, scope_key). Commit 41ebe1d: EmitContext->ScopeBinding. |
| Atomic event+projection write transaction (S7) — NOT documented in ADR | **SHIPPED** | Shipped and test-proven but absent from the ADR. SqliteEventStore::append wraps insert_event_row + EventBus::dispatch in one BEGIN IMMEDIATE..COMMIT on the same connection; any projection-handler error rolls back the event row too. Reached via the canonical kernel path (EventSink::persist -> append). Needs a new decision section (see new_sections). |
| Aggregate / cross-space fan-out principle (central log never queried directly) — under-stated in ADR | **PARTIAL** | The 'reads are per-space; cross-space reads fan out + aggregate' principle is realized concretely — but ONLY in search. run_global runs the scope-blind service per space over each space's own backend and merge_global aggregates+re-ranks; SingleBackendDataStore proves every read is WHERE space_id = ? (no union query of a central log). A GENERAL reusable aggregate service (for collections/artifacts/hub-discovery) is NOT built — it remains a pattern, not a service. Needs a new decision section (see new_sections). |
| §9 Phasing / Migration / status front-matter ('proposed', 2026-05-31) | **PARTIAL** | Phase 1 (SpaceContext/DataStore abstraction) and the substrate physical-partition emit path landed; the migrator, per-domain DataStore backfill (data.db fold-in), per-backend sync cursors, sandbox-as-substrate-isolation, and key flow have not. Front-matter status 'proposed' remains defensible because §2/§4/§5/§8 and the §6 substrate-isolation half are still target-only — but the parts that DID land (per-scope emit, ScopeBinding, atomic tx, per-scope search) are now shipped reality, so the operator should decide whether to ratify those. |
| Space identity — `space_id` derivation (under-specified in ADR) | **CORRECTION** (2026-06-19) | The ADR keys spaces by `space_id` and requires sync across collaborators/devices, but the impl derives `space_id = path_hash(path)` — location-coupled, so the same space gets a different id per machine, breaking that premise. Fix: a **stable, location-independent id** minted at space creation, carried in the space's own event log (so it travels); the `path → id` map is a per-device index (the registry). |


## Context

Three forces have converged on the substrate at the same time:

1. **Event classification is shipped.** All 122 event types are tagged `{scope, retention}` (see `.artifacts/specs/space/event-classification.md` and `crates/core/ship-substrate/src/events/payload_registry.rs`). `scope ∈ device | account | space` — and the doc names this explicitly as *"which log it rides → what it syncs with."* Fail-closed default: `device, durable`.

2. **Sync is shipped — events as the wire.** `crates/core/ship-platform/src/sync/` pushes local events to cloud, pulls remote events down, with per-`{push,pull}:platform:{space_id}` cursors. Event envelopes are the sync wire format. The "events vs sqlite-CRDT vs WAL-replication" question is already decided in code.

3. **Reactive (live data) is the next first-class surface.** SUI is already patch-based and dedup'd client-side via `sharedSource`. Collection writes already emit `collection.item_*` kernel events as space-scope, durable, classified. The reactive path is "subscribe to kernel + re-query + diff snapshot → patches → fanout."

But the **storage** has not caught up:

- `platform.db` is one file containing every scope's events mixed together. The `space_id` column added in #116 is logical partition only; physically every space's events sit next to every other space's events in the same file.
- `data.db` is one file containing every space's collection items mixed together.
- A device with five mounted spaces has all five spaces' events and items physically co-located.
- Sync still requires per-row filtering (`WHERE space_id = ?`) on every push/pull, every reactive subscription, every projection write — not because the scope was unclear, but because storage doesn't honor it.
- The privacy/sovereignty boundary is at the row level, not the file level. Backup, restore, delete-space, share-space-with-collaborator, encrypt-per-space — all require row-by-row code paths.
- A rogue process with filesystem access reads every space's data with one `sqlite3 platform.db`.

The `home-layout-and-scope-convention` ADR codified the present layout as a step forward and explicitly anticipated the next move: *"the split itself already anticipates the XDG boundary."* The `collections-as-data-layer` ADR codified the conceptual split between runtime state and user content but did not anticipate scope-physical partitioning because the scope axis had not yet been classified.

This ADR finishes the move both anticipated.

## Decision

**Physically partition the substrate by scope.** Three backends, three filesystem locations, three kernel router instances, three sync channels, three encryption keys.

### 1. Three backends

```
runtime.db             ← runtime-scope events + projections
                         (actor.*, executor.*, daemon.*, hook.*, harness.*, …)
                         Never syncs. Local diagnostics and runtime.
                         (formerly named device.db; the Device scope was
                         renamed Runtime and the former System scope folded in.)

account.db             ← account-scope events + projections
                         (account.*, auth.session.*, agent.grants.*, …)
                         Syncs across user's devices via cloud account peer.

spaces/<space-id>/
├── space.db           ← space-scope events + ALL space projections
│                        (collection.*, artifact.* metadata, agent.session.*,
│                         pipeline.*, session.*, view.*, skill.*, …)
│                        Syncs across collaborators on this space.
└── blobs/             ← artifact bytes, content-addressed
                         (separate sync channel; not in the event log)
```

Classification of which events go to which backend is **the existing payload registry** (`payload_registry.rs`). No new classification work — that's done.

### 2. Filesystem location: OS-conventional for state, `~/.ship/` for config

State (the backends) lives in the OS-conventional application-data location:

```
Linux:    $XDG_DATA_HOME/ship/           (default: ~/.local/share/ship/)
macOS:    ~/Library/Application Support/ship/
Windows:  %APPDATA%\ship\
```

Resolved via the `directories` Rust crate. Overridable via `SHIP_DATA_HOME` for sandbox scenarios (Flatpak, devcontainers, CI). `SHIP_HOME` continues to override the config root only.

`~/.ship/` shrinks to user-inspectable config:

```
~/.ship/
└── library/                          ← portable user-scope authoring content,
    ├── agents/ skills/ collections/    unchanged from `home-layout-and-scope-convention`
    │   views/ pipelines/ triggers/     §1 / §2 (the vector enum)
    │   rules/ permissions/ themes/
    │   mcp/
    └── settings.jsonc                ← portable user preferences
```

The `machine/` bucket is **removed**. Its former contents move:

- `machine/db/platform.db` → split per §1 into `<XDG>/ship/{device,account,spaces/*}/...`
- `machine/db/data.db` → folded into per-space `space.db` (no separate data.db file exists)
- `machine/daemon/` → `<XDG_RUNTIME_DIR>/ship/daemon/` on Linux, OS-equivalents elsewhere (IPC handshake is runtime state, not data state)
- `machine/cache/`, `machine/crashes/`, `machine/logs/`, `machine/sessions/`, `machine/actors/`, `machine/projects/` → `<XDG>/ship/...` (state)
- `machine/local.jsonc` → `<XDG_CONFIG_HOME>/ship/local.jsonc` (machine-specific config)

The two-bucket discipline survives in spirit (portable vs not). It's implemented by OS-convention boundaries instead of one `~/.ship/` dir.

### 3. Kernel router becomes scope-aware

Today: one `KernelRouter` writing to one `platform.db`. After (TARGET SHAPE — see note below; the landed implementation routes at the sink, not via three KernelLog instances):

```rust
// AS LANDED: one router, one scope-routed sink. Scope fan-out is the
// sink's job; routing keys come from the authoritative ScopeBinding
// (commit 41ebe1d, EmitContext->ScopeBinding), not from a ctx.space_id field.
pub async fn emit(&self, mut event: EventEnvelope, ctx: &EmitContext) -> Result<()> {
    // ... validators + write-namespace enforcement ...
    ship_types::stamp_author(&mut event, &ctx.binding);     // stamp author identity
    ship_types::enforce_scope(&mut event, &ctx.binding)?;   // stamp scope_key, or reject
                                                            // CrossSpace / NoSpaceForSpaceEvent
    self.inner.route(event).await   // persist via the single PartitionedSink, then deliver
}

// PartitionedSink::route then keys on the stamped (scope, scope_key):
//   EventScope::Space   -> spaces/<scope_key>/space.db
//   EventScope::Account -> account.db
//   EventScope::Runtime -> runtime.db   (fail-closed default; replaces Device)
```

Each `KernelLog` owns its own append-only event log, its own broadcast channel, its own projection rebuild cursors. Subscribers subscribe to one log. Sync runs per-log. Reactive resolvers attach per-log.

### 4. Sync model maps 1:1 to backends

- `runtime.db`: never syncs. Local-only by definition.
- `account.db`: syncs to cloud account peer. Wire is the account event log's appends.
- `space.db` per space: syncs to that space's collaborator graph. Wire is the space event log's appends.

Per-backend cursors replace the current per-`{push,pull}:platform:{space_id}` namespace. The `space_id` column on every event becomes redundant for space-scope events (they're already in a space-specific log) but is retained on the envelope for cross-cutting analysis and as belt-and-suspenders.

### 5. Each space gets its own encryption key

SQLCipher per backend file:

- `runtime.db`: encrypted with a device-installation key in OS keychain (`ship.device`). [PROPOSED — no SQLCipher/keychain shipped; see §5 status note]
- `account.db`: encrypted with an account key in OS keychain (`ship.account.<account-id>`).
- `space.db` per space: encrypted with a per-space key in OS keychain (`ship.space.<space-id>`).

Keys never touch disk. OS keychain ACL bound to daemon binary identity; not inherited by spawned child processes by default.

Per-space-key choice (vs single installation key): smaller blast radius on key compromise, clean revocation model when removing a collaborator, and a `cp -r` of a `spaces/<space-id>/` dir is useless without the corresponding keychain entry (which is a security feature, not a bug — `ship space clone` re-keys explicitly).

### 6. Agent process isolation is now a real defense

The partition enables filesystem-level isolation that wasn't possible with a single mixed `platform.db`. Layered defense for rogue or compromised agents:

| Layer | Mechanism |
|---|---|
| Encryption at rest | SQLCipher per-backend, key in OS keychain, daemon-identity-bound ACL |
| Filesystem sandbox | [PARTIAL] OS sandboxes are shipped (ship-isolation: SrtWrapper/bubblewrap, Docker, Youki, Lima, Wsl2, AppleContainer, Firecracker) and wired at spawn, but they confine the agent WORKSPACE, not the substrate — the sandbox has no reference to the ship data dir, so it does not yet exclude `<XDG>/ship/`. Tier-1 (default process) does not confine reads. Substrate-hiding is target, not landed. |
| RPC gating | All substrate access goes through daemon Connect RPCs; token + capability/grant check on every call |
| PID-bound identity | Token mint at spawn binds to PID (per #110-#112); a process that didn't go through supervisor spawn has no valid token |

The combination means: a rogue agent on the host cannot acquire a token out-of-band (no PID-bound mint) [shipped], and — once §5/§6 substrate-isolation land — cannot `sqlite3 space.db` (file not mounted into its view) and the bytes are ciphertext. NOTE (today): the file IS reachable on disk (sandbox confines the workspace, not the substrate) and bytes are plaintext sqlite (no SQLCipher). These two guarantees are TARGET, not current.

Sandbox configuration is applied by the supervisor at spawn time. [PARTIAL] As landed, the strategy is resolved from isolation.* settings + OS capability probes + an `enforce` flag (off by default) — NOT yet from the agent's harness capability/grant spec; ActorProfile still notes 'No explicit isolation axis on ActorProfile today' (#83/#85 grants integration is target).

### 7. Search becomes naturally per-scope

Cross-cutting work that previously needed row-level filtering becomes file-level scoping:

- In-space search: `ShipdSearchContext` for that space opens that space's `space.db`, builds snapshots from its projections. Throughput bounded by one space's data regardless of how many spaces are mounted.
- Hub search (the user-level discovery surface): a separate context aggregating `account.db` + a lightweight per-space summary from each `space.db`'s manifest table. Conceptually different surface from in-space search.

Same pattern applies to triggers (subscribe to the kernels for the scopes the trigger declares), analytics (split cleanly along the scope axis), and audit (the union of event logs IS the audit log, rendered as view, not stored again).

### 8. Data.db disappears as a file

[PROPOSED — NOT yet landed] The `data.db` file does not exist after this change. Collection projections live inside `space.db` alongside artifact metadata, sessions projection, agent_sessions projection, etc. — they are all tables in the same per-space backend, fed by the same per-space event log.

The collections-as-universal-data-layer principle from the superseded ADR survives in full: all user-authored content goes through collections, collections emit events, query API is uniform. Only the physical realization changes — from `data.db` to per-space tables in `space.db`.

### 9. Phasing

This ADR is a target architecture, not a one-shot migration. Sequencing:

1. **First:** ship the `SpaceContext` / `DataStore` abstraction in `ship-collection` (and any other crate that will be partition-affected) so domain code is *future-shaped* before the substrate moves. Domain crates compile against the abstraction; the abstraction binds to today's single-backend substrate.
2. **Substrate arc:** physical partition lands as its own dedicated arc (see "Migration" below). All projection rebuilds happen there.
3. **Per-domain backfill:** each domain (collections, artifacts, sessions, …) flips its `DataStore` binding from the legacy single-backend impl to the per-scope backend impl. No domain-code changes.

The reverse order — refactoring domain code against the new substrate after the substrate lands — would force a flag-day flip. Doing the abstraction first gives a clean cutover per domain.

## 10. Atomic event + projection write

> Status: **shipped** (test-proven). This decision was implemented but never written down in the original ADR; recording it here closes that gap.

An event row and every projection it feeds commit **in a single SQLite transaction on one connection**, or roll back together. There is no window in which a durable event exists without its derived state, and no orphaned projection write for an event that failed to persist.

### What it guarantees

`SqliteEventStore::append` opens `BEGIN IMMEDIATE`, inserts the event row, then dispatches the projection bus on the **same** `sqlx` connection, then `COMMIT`s. If the row insert OR any projection handler errors, it `ROLLBACK`s and returns `Err` — leaving **zero** rows in `events` (`crates/core/ship-substrate/src/persistence/event_store.rs:224-241`). The contract is proven by dedicated tests: a deliberately-failing projection makes `append` return `Err` and leaves the `events` table empty (`append_rolls_back_event_row_when_projection_handler_fails`), and a clean append commits exactly one row (`crates/core/ship-substrate/src/persistence/tx_atomicity_tests.rs:75-117`).

### Why co-location = one transaction

The event log and its projections live in the **same per-scope backend** (§1) — `collection.*` events and the collection projection are both tables in that space's `space.db`. Because they share a file and a connection, the event-row insert and the projection apply can be the same atomic unit instead of two coordinated writes across files. `EventBus::dispatch` applies every matching projection on that same `&mut SqliteConnection` and **propagates the first handler error** (first-failing handler short-circuits the dispatch); only the `bump_cursor` bookkeeping write is treated as non-fatal and logged (`crates/core/ship-substrate/src/projections/registry.rs:65-87`). This first-error-wins propagation is exactly what makes `append`'s rollback all-or-nothing.

### Reached via the canonical kernel path

`append` is not a side door — it IS the write path the kernel takes. `SqliteEventStore` implements `ship_kernel::EventSink`, and `EventSink::persist` delegates straight to `append` (`crates/core/ship-substrate/src/persistence/event_store.rs:270-279`). So `kernel.emit -> inner.route -> sink.persist -> append` lands every emitted event in the same atomic event+projection transaction.

### Consequence for §1

This is the per-scope-co-location payoff stated as a guarantee: because projections live in the same backend as their source events, derived state is transactionally consistent with the log per scope, with no cross-file two-phase commit.

## 11. Aggregate reads fan out across per-scope backends

> Status: **partial** — the principle and one concrete instance (search) are shipped; a general reusable aggregate service is **not** built (it is a pattern, not a service).

Once the substrate is per-scope (§1), there is no single file to query across spaces. The landed posture: **reads are physically per-space, and cross-space reads are a fan-out over many single-space backends followed by an in-memory aggregate — the central log is never queried directly.**

### Reads are per-space by construction

`SingleBackendDataStore` scopes every query / count / update / delete to `WHERE space_id = ?` over a single backend (one `space.db` / `runtime.db`) (`crates/core/ship-substrate/src/ports/single_backend.rs:151-211`). There is no store that reads the union of all spaces. Any cross-space answer is therefore necessarily a fan-out over N single-backend stores plus a merge.

### The landed instance: global search

The pattern is realized concretely — and, today, **only** — in search. The pure core (`merge_global` / `run_global`) runs the **same scope-blind `SearchService` once per included space** over that space's own context, stamps each space's hits with its `space_id`, and folds them into one stable-sorted, capped, re-ranked list — no I/O in the core (`crates/core/ship-platform/crates/ship-search/src/global.rs:54-121`). It is wired in shipd: `ShipdSearchWire::run_global_search` enumerates live spaces via `SpaceService.List`, builds a per-space `SearchContext` bound to each space's own physical db (`SpaceContext::for_space`), applies the per-space `search.global=false` opt-out gate, and calls `ship_search::run_global` (`apps/shipd/src/search_rpc.rs:77-111`). The central log is never read; results come from per-space backends and are aggregated at this seam.

### Honest scope: pattern, not yet a service

This is **not** a general aggregate-projection service. `run_global` / `merge_global` / `SpaceTarget` live only in `ship-search` and are consumed only by `search_rpc.rs`; there is no `MultiBackendDataStore` / `AggregateStore` trait, and the `DataStore` port is strictly single-space. Collections, artifacts, and hub-discovery have **no** cross-space enumeration path of their own — if they need one, today they would re-implement the search pattern by hand. (Note: `orchestrator/fanout_aggregate.rs` is pipeline-lane fan-out — parallel pipeline steps — an unrelated concept, not a cross-space data aggregator.)

The §7 'hub search' surface as originally described (a distinct context aggregating `account.db` + a per-space manifest-summary table) is likewise **not** what shipped: global search reuses the identical in-space surface lifted one level, with no `account.db` aggregation and no manifest-summary read.

### Decision

Affirm the principle as architecture: cross-scope reads MUST fan out over per-scope backends and aggregate; no consumer may reintroduce a union query of a central log. Generalizing the fan-out into a reusable aggregate service (so collections/artifacts/hub-discovery do not each re-implement it) is **deferred, explicitly tracked as future work**, not credited as shipped.

## Consequences

### Positive

- **Sovereignty boundary equals filesystem boundary.** Backup, restore, delete-space, collaborator revoke, encryption rotation — all become file operations instead of row-level SQL.
- **Sync is per-file replication.** Per-scope cursors map to per-file logs. No `WHERE space_id = ?` on every sync query.
- **Reactive throughput bounded per scope.** Watch resolver for collection X in space A only sees space A's events. No filtering, no cross-space noise.
- **Agent isolation gets a defense in depth that single-file substrate can't provide.** Filesystem sandbox can hide entire backends; encryption is per-key; token gate at RPC; PID-bound identity.
- **Each scope has independent retention, encryption, sync policy.** Future divergence (e.g., space.db with stricter retention than account.db) is configuration, not a code refactor.
- **Cross-cutting concerns (search, triggers, audit) get simpler, not more complex.** Each becomes a per-scope service with a unified-view layer on top for the few cases that genuinely need cross-scope (hub search, system audit).
- **`data.db` disappears as a concept.** One less per-space file to reason about. Collections are just one of several projections inside the per-space backend.

### Negative

- **Substrate refactor is the highest-cost arc in the codebase.** Per-scope kernel routers, per-backend log instances, per-backend projection rebuild, migration of every emit site to honor scope routing.
- **Migration of existing installs is real work.** Existing `platform.db` files contain mixed-scope events. Migrator must classify (using the registry), split into per-backend logs, and rebuild per-backend projections. Atomic; not partial.
- **Per-space key model creates a `cp -r` footgun.** Casual copy of `spaces/<id>/` cannot be mounted without the key. This is a security feature but needs a clear UX (a real `ship space clone` op that re-keys) and clear error messages on the failure case.
- **Some currently-trivial joins become awkward.** E.g., "this actor.crashed event was about pipeline work in space X" — actor.crashed lives in `device.db`, pipeline.run lives in `space.db`. Cross-backend tracing requires payload-carried foreign keys, not SQL joins.
- **Cloud peer architecture must keep up.** Per-space sync channels mean N concurrent peer connections per device for a heavy user. Cloud-side fanout and per-space ACL get more interesting.
- **Three encryption-key sources is more complexity than one.** OS keychain integration on three platforms times three key namespaces.

### Net

The complexity moves to the right places. The substrate gets harder; everything that builds on the substrate gets simpler. The current single-backend substrate is a leaky abstraction that forces every consumer (sync, reactive, search, triggers, analytics, audit) to re-implement scope filtering. Centralizing scope at the storage layer is the simpler architecture across the codebase, even if the storage layer itself becomes more involved.

## Alternatives Considered

### Status quo (logical partition via `space_id` column)

What `#116` shipped. Adequate for v1, but every consumer pays the filtering cost forever, and the sovereignty boundary stays at row level — no clean per-space backup, encryption, or revoke. Defers the cost rather than paying it. Rejected: the cost compounds with every new consumer (and reactive + per-scope sync are imminent).

### Hybrid (one event log, partitioned projections)

Single platform-wide event log; projections split per-scope. Sync still reads from the unified log. Half-measure: gets the per-scope projection wins without the per-scope sync wins or the encryption/isolation wins. Most of the migration cost for half the benefit.

### Postpone until post-launch

Tempting. Rejected because every new domain refactor (collections, agents, artifacts, …) bakes in single-backend assumptions, and prying them out post-launch is N domains × N consumers of work. Doing the abstraction first costs once; doing it never costs every-refactor-forever.

### Keep everything under `~/.ship/` (no XDG migration)

Simpler for users — one path. Rejected because backup/sync tools treat `~/.ship/` as "user files" (Time Machine, iCloud Documents, Dropbox all happily include it) which produces accidental data leakage for what should be opaque encrypted backends. OS conventions exist precisely for this distinction.

### Single per-account encryption key, not per-space

Operationally simpler. Rejected because losing one key leaks everything; revoking a collaborator from one space means rotating the key for every space; `cp -r` of a space dir gives the destination a usable copy without explicit re-keying.

### Run agents as a different OS user

Strongest possible isolation. Rejected for v1 because it requires user provisioning at install time, breaks dev ergonomics, and the sandbox + encryption + token + PID-binding combination provides sufficient defense without OS-user separation. Revisit if specific threat models justify it.

## Migration

Owned by a dedicated arc (`substrate-physical-partition`). Domain refactors pause until the abstraction in §9.1 lands.

Sketch:

1. **Define the `SpaceContext` / `DataStore` / `KeyMaterial` ports** in the relevant platform crates. Bind today's single-backend `platform.db` + `data.db` to these ports — zero behavior change, codebase now compiles against the abstraction.
2. **Land the per-scope `KernelRouter`** wrapping today's single backend (still one file under the hood). Routes by `EventScope`. Unblocks per-scope subscribers immediately even before storage splits.
3. **Ship the per-backend `KernelLog`** implementations + per-backend projection rebuild + per-backend sync cursors. Three real backends now exist; old single-backend code paths are dead.
4. **Migrator runs on `shipd` start**: detects pre-partition layout, classifies every event in the legacy `platform.db` using the registry, splits into `device.db` + `account.db` + `spaces/*/space.db`, rebuilds per-backend projections, deletes legacy files. Atomic; rollback on partial. Same one-shot pattern as `home-layout-and-scope-convention` §8.
5. **Per-domain `DataStore` binding flips** for each domain that has one (collections, artifacts metadata, sessions, …). Each is a small, isolated change because the port already exists.
6. **Sandboxing wiring** lands at supervisor spawn sites. Per-OS sandbox profile applied to every agent process.
7. **Key flow lands** alongside §5. Keychain entries provisioned at first mount; SQLCipher pragmas applied on every open.

## How to Measure Success

- `<XDG>/ship/` contains exactly `runtime.db`, `account.db`, and `spaces/<space-id>/` per mounted space; no other top-level files. [TARGET: filesystem location is still ~/.ship, not <XDG>; see §2 status]
- No `data.db` file exists anywhere.
- `~/.ship/` contains only `library/` and `settings.jsonc` (no `machine/`, no DBs).
- `rg "platform.db"` returns zero hits outside migration code.
- Per-backend cursors visible in each backend's `cursors` table; legacy `push:platform:{space_id}` cursors do not exist.
- Spawning an agent in a sandboxed profile, then attempting `sqlite3 <XDG>/ship/spaces/.../space.db` from inside that agent fails with "file not found" (sandbox) or "file is encrypted" (key not in agent's keychain ACL).
- Deleting `<XDG>/ship/spaces/<space-id>/` and the corresponding keychain entry removes that space's data with no orphaned rows anywhere else.
- Sync of a single space to a peer transfers exactly one event log (that space's), not a filtered view of a shared log.
