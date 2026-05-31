---
title: "Scope-Partitioned Substrate"
description: "ADR: the event log and its projections are physically partitioned by scope into device.db, account.db, and one space.db per mounted space. Each backend has its own kernel router, its own sync channel, its own encryption key."
group: ADRs
order: 30
audience: internal
status: proposed
date: 2026-05-31
supersedes: [home-layout-and-scope-convention, collections-as-data-layer]
tags: [storage, substrate, partition, kernel, sync, security, encryption, scope]
---

# Scope-Partitioned Substrate

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
device.db              ← device-scope events + projections
                         (actor.*, executor.*, daemon.*, hook.*, harness.*, …)
                         Never syncs. Local diagnostics and runtime.

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

Today: one `KernelRouter` writing to one `platform.db`. After:

```rust
pub struct KernelRouter {
    device: Arc<KernelLog>,                          // appends to device.db
    account: Arc<KernelLog>,                         // appends to account.db
    spaces: DashMap<SpaceId, Arc<KernelLog>>,        // per-space, lazy mount
}

impl KernelRouter {
    pub async fn emit(&self, envelope: EventEnvelope, ctx: &EmitContext) -> Result<Cursor> {
        let log = match envelope.scope {
            EventScope::Device  => &self.device,
            EventScope::Account => &self.account,
            EventScope::Space   => self.spaces.get(&ctx.space_id?).ok_or(SpaceNotMounted)?,
        };
        log.append(envelope).await
    }
}
```

Each `KernelLog` owns its own append-only event log, its own broadcast channel, its own projection rebuild cursors. Subscribers subscribe to one log. Sync runs per-log. Reactive resolvers attach per-log.

### 4. Sync model maps 1:1 to backends

- `device.db`: never syncs. Local-only by definition.
- `account.db`: syncs to cloud account peer. Wire is the account event log's appends.
- `space.db` per space: syncs to that space's collaborator graph. Wire is the space event log's appends.

Per-backend cursors replace the current per-`{push,pull}:platform:{space_id}` namespace. The `space_id` column on every event becomes redundant for space-scope events (they're already in a space-specific log) but is retained on the envelope for cross-cutting analysis and as belt-and-suspenders.

### 5. Each space gets its own encryption key

SQLCipher per backend file:

- `device.db`: encrypted with a device-installation key in OS keychain (`ship.device`).
- `account.db`: encrypted with an account key in OS keychain (`ship.account.<account-id>`).
- `space.db` per space: encrypted with a per-space key in OS keychain (`ship.space.<space-id>`).

Keys never touch disk. OS keychain ACL bound to daemon binary identity; not inherited by spawned child processes by default.

Per-space-key choice (vs single installation key): smaller blast radius on key compromise, clean revocation model when removing a collaborator, and a `cp -r` of a `spaces/<space-id>/` dir is useless without the corresponding keychain entry (which is a security feature, not a bug — `ship space clone` re-keys explicitly).

### 6. Agent process isolation is now a real defense

The partition enables filesystem-level isolation that wasn't possible with a single mixed `platform.db`. Layered defense for rogue or compromised agents:

| Layer | Mechanism |
|---|---|
| Encryption at rest | SQLCipher per-backend, key in OS keychain, daemon-identity-bound ACL |
| Filesystem sandbox | Spawn agents in OS sandboxes (bubblewrap on Linux, sandbox-exec on macOS, AppContainer on Windows) that do not mount `<XDG>/ship/` into the agent's view at all |
| RPC gating | All substrate access goes through daemon Connect RPCs; token + capability/grant check on every call |
| PID-bound identity | Token mint at spawn binds to PID (per #110-#112); a process that didn't go through supervisor spawn has no valid token |

The combination means: a rogue agent on the host cannot `sqlite3 space.db` (file not mounted into its view), cannot `python -c "import sqlite3..."` (same), cannot acquire a token out-of-band (no PID-bound mint), and even if it bypassed everything to read raw bytes, they're ciphertext.

Sandbox configuration is the supervisor's responsibility at spawn time, parameterized by the agent's harness capability spec (see #83 / #85 grants work).

### 7. Search becomes naturally per-scope

Cross-cutting work that previously needed row-level filtering becomes file-level scoping:

- In-space search: `ShipdSearchContext` for that space opens that space's `space.db`, builds snapshots from its projections. Throughput bounded by one space's data regardless of how many spaces are mounted.
- Hub search (the user-level discovery surface): a separate context aggregating `account.db` + a lightweight per-space summary from each `space.db`'s manifest table. Conceptually different surface from in-space search.

Same pattern applies to triggers (subscribe to the kernels for the scopes the trigger declares), analytics (split cleanly along the scope axis), and audit (the union of event logs IS the audit log, rendered as view, not stored again).

### 8. Data.db disappears as a file

The `data.db` file does not exist after this change. Collection projections live inside `space.db` alongside artifact metadata, sessions projection, agent_sessions projection, etc. — they are all tables in the same per-space backend, fed by the same per-space event log.

The collections-as-universal-data-layer principle from the superseded ADR survives in full: all user-authored content goes through collections, collections emit events, query API is uniform. Only the physical realization changes — from `data.db` to per-space tables in `space.db`.

### 9. Phasing

This ADR is a target architecture, not a one-shot migration. Sequencing:

1. **First:** ship the `SpaceContext` / `DataStore` abstraction in `ship-collection` (and any other crate that will be partition-affected) so domain code is *future-shaped* before the substrate moves. Domain crates compile against the abstraction; the abstraction binds to today's single-backend substrate.
2. **Substrate arc:** physical partition lands as its own dedicated arc (see "Migration" below). All projection rebuilds happen there.
3. **Per-domain backfill:** each domain (collections, artifacts, sessions, …) flips its `DataStore` binding from the legacy single-backend impl to the per-scope backend impl. No domain-code changes.

The reverse order — refactoring domain code against the new substrate after the substrate lands — would force a flag-day flip. Doing the abstraction first gives a clean cutover per domain.

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

- `<XDG>/ship/` contains exactly `device.db`, `account.db`, and `spaces/<space-id>/` per mounted space; no other top-level files.
- No `data.db` file exists anywhere.
- `~/.ship/` contains only `library/` and `settings.jsonc` (no `machine/`, no DBs).
- `rg "platform.db"` returns zero hits outside migration code.
- Per-backend cursors visible in each backend's `cursors` table; legacy `push:platform:{space_id}` cursors do not exist.
- Spawning an agent in a sandboxed profile, then attempting `sqlite3 <XDG>/ship/spaces/.../space.db` from inside that agent fails with "file not found" (sandbox) or "file is encrypted" (key not in agent's keychain ACL).
- Deleting `<XDG>/ship/spaces/<space-id>/` and the corresponding keychain entry removes that space's data with no orphaned rows anywhere else.
- Sync of a single space to a peer transfers exactly one event log (that space's), not a filtered view of a shared log.
