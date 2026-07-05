# Overview

Ship is a runtime for secure, full-stack, agentic applications. A daemon dispatches
agents, advances pipelines, and event-sources the result. The event log is truth;
disk is a projection. Read this first, then the binding rules below.

Internally, Ship compiles `.ship/` config into harness-native agent files — dev
infrastructure for Ship's own agents, not a product surface.

## Scope model — ownership = sync

Every event belongs to one scope, owned by whoever holds the authoritative copy:

* **Space** — collaborative work inside a `.ship/` directory (views, agents, skills, pipelines). One daemon serves many spaces; each partitions into its own log. Syncs to collaborators.
* **Account** — identity, library, settings. Syncs across the account's own devices.
* **Runtime** — execution state (sessions, actors, processes, daemon). Machine-local in `runtime.db`; snapshot-restorable, never continuously synced.

Substrate (L0) is fat — it holds the logic and the truth. Logic moves down, not up.

## Where to go next

* Binding rules: `architecture.md`, `engineering.md`, `agent-discipline.md`, `job-protocol.md`.
* How it fits together: the `ship-architecture` doc-skill (`docs/ship-architecture/`).
* How we document / agent memory: the `ship-docs` and `memento` skills.
