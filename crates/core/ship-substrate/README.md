# ship-substrate

**Layer:** L0 kernel infrastructure — the lowest layer any other Ship crate depends on.

ship-substrate is the event bus, actor model, SQLite persistence, projections registry, supervisor, home/project path layout, and every other low-level mechanism the upper layers stand on. No L1 domain logic lives here.

## Key seams

- `events/` — the event bus, router, and `SqliteEventStore` (the kernel's truth store).
- `persistence/` — event store, journal store, actor store, snapshot and partition primitives.
- `supervisor/` — process/actor lifecycle, space-id resolver seam.
- `projections/` — `EventBus` + L0 defaults; domain projections arrive as a
  `ProjectionFactory` constructed by ship-platform / shipd and passed into
  `init_kernel_router_with` / `PartitionedSink` / store constructors (no
  process-global install step).
- `ports` (re-exported from `ship-kernel`) — `DataStore`, `BlobStore`, `FsBroker`, `StepExecutor`, and the rest of the I/O port traits.
- `home/` — `ShipHome`, `Scope`, `Db`, `Vector`; the canonical path-layout for daemons and worktrees.

## Sub-crates

`ship-types`, `ship-kernel`, `ship-substrate-host`, `ship-events-macro`, `ship-runner`, `ship-pipelines`, `ship-js`, `durable-execution` are nested crates; ship-substrate re-exports the ports surface from `ship-kernel`.

## What it does NOT contain

Domain event registrations. Post keystone-A, every domain family registers in its owning L1 crate; ship-substrate registers only runtime mechanics (`actor.*`, `process.*`, `executor.*`, `daemon.*`, `kernel.actor.*`, `mesh.*`, `pipeline.*`, `supervisor.shutting_down`, `settings.changed`).
