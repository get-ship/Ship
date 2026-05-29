---
title: Storage
description: Four stores — collections, files, key-value, events — all flowing through one event log, so sync and history work uniformly.
section: Data
order: 1
kicker: Data
tags: [data, storage]
related: [collections]
---

Ship gives your app four kinds of storage, and they all flow through **one event log** underneath — which is why sync and history work the same way across every one of them.

- **[Collections](/docs/collections/)** — schemaful structured data. The records your app holds: tasks, drafts, interviews, anything you'd otherwise put in a database.
- **Files** — blob storage with metadata. Uploads and the artifacts agents and pipelines produce.
- **Key-value** — a bounded key-value store for small values: app state and settings.
- **Events** — the append-only log plus derived projections. It's the runtime's spine, and what makes [replay and audit](/docs/why-a-runtime/) free.

Because every write lands as an event on that one log, the optional cloud layer can sync a space by replicating the log — collections, files, and KV all travel together, no per-store machinery.
