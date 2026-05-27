---
group: Architecture
order: 7
title: Truth model
description: What Ship treats as source of truth — event-sourced vs event-observed vs blob — and where each kind of state lives. The single reference; nothing should contradict it.
audience: internal
---

# Truth model

Every piece of state in Ship has exactly one **truth backend**. Conflating them
is the source of "mixed message" bugs (e.g. is a setting in the log or on disk?).
This is the canonical classification — code and docs must not contradict it.

The discriminator is **who owns the authoritative copy**, which decides how it
syncs:

| Class | Authoritative copy | Change signal | Synced by |
|---|---|---|---|
| **Log-as-truth** (event-sourced) | the event log + its projections | the event *is* the change | the per-space event log |
| **File-as-truth** (event-observed) | the file on disk (`.ship/`) | a stamped `*.changed` event the watcher emits (`fsnotify`) | the file (via the package/bundle format) |
| **Blob** | the bytes (db row / cache object) | n/a (or a coarse change event) | its own mechanism (CRDT / content-address) |

## 1. Log-as-truth — event-sourced

The projection **is** the truth; there is no canonical file. Writing means
appending an event; reading means querying a projection. This is the default
(`CLAUDE.md`: "Event-sourced storage by default").

- **What:** sessions, actors, pipeline-runs, grants (`provisioned_grants`),
  members, auth sessions, **per-space settings** (`space.setting.changed`).
- **Where:** the kernel + `ship-substrate`/`ship-platform` projections.
- **Sync:** the space's event log is the unit of sync — append + fan-out, reads
  always local (the cloud sync model). This is **Format 1** in the bundle work.

A change here is emitted, not observed. There is no watcher; the emitter is the
single writer.

## 2. File-as-truth — event-observed (the `fsnotify` class)

The **file** is the authoritative copy. The daemon does not own it — a human or
agent edits `.ship/` directly, and the watcher (`config::watcher`, fsnotify)
**observes** the change and emits a *stamped notification* (`config.file_changed`
+ the coarse `<domain>.changed`, content-hashed for dedupe). The event is a
**signal, not the truth** — replaying it does not reconstruct the file.

- **What:** the authored `.ship/` artifacts — agents, skills, views, pipelines,
  triggers, collection-schemas, rules, mcp, permissions.
- **Where:** `.ship/` on disk; the composition engine composes the base +
  dependency contributions into the effective view (see [Spaces](spaces.md)).
- **Sync:** the **files** travel (the package/bundle format — content-addressed,
  signed); the change events are local notifications, NOT synced. This is
  **Format 2** in the bundle work.

The line, stated once: **Format 1 = the log is truth; Format 2 = the file is
truth and the event is a stamped signal.** #91 (substrate-as-log-as-truth) owns
class 1; it deliberately does NOT pull class 2 into the log.

## 3. Blob — legitimately not event-sourced

Some state is bytes by nature; event-sourcing it would be wrong. #92 is about
naming these explicitly and keeping disk a *projection*, not a second truth.

- **What:** collection data (`data.db`); the content-addressed package cache
  (`~/.ship/cache/objects/<tree-hash>/`); compiled build outputs.
- **Sync:** each by its own mechanism — collections via CRDT; packages are
  content-addressed (the hash IS the identity) and signed; build outputs are
  derived, not synced.

## Settings: global vs space (a worked example of the split)

Settings are the place this split is most often gotten wrong, so it is fixed
here:

- **Global daemon settings** (`daemon_settings`, e.g. `terminal.program`,
  `daemon.port`) — **machine-local KV**, NOT space-scoped, NOT synced. A
  deliberate KV store (not event-sourced): it is per-machine infrastructure
  config, not space state. Reading falls back to a hardcoded default.
- **Space settings** (theme, layout, isolation, agent prefs) — **event-sourced**
  per space (`space.setting.changed` → `space_settings` projection, class 1).
  Synced with the space's log.
- **Effective config** = global defaults **overlaid by** the space's overrides,
  merged **at read time** in `SpaceService.Info` (`settings_json`). The global
  copy is never mutated by a space; the overlay is per-key, space wins.

So "settings" is two truth backends with one read-time merge — not one store.
`kv` skill-vars are likewise a deliberate KV class (the L1 skill-var primitive),
not a log-as-truth gap.

## The content hash (one value, three consumers)

There is **one** content hash — `compute_tree_hash` over a `.ship` tree
(`ship-platform::registry::hash`). It is the package cache object key
(`cache::store`), the registry's stored `contentHash`, and what a bundle
attestation binds (`registry::bundle_sig`). Registry ↔ attestation ↔ cache are
the same hash; there is no second "tarball-bytes" hash.

## The rule

When adding state, pick the class first:
- Mutated only through the daemon, and you want history/sync → **log-as-truth**.
- Authored by humans/agents as files, daemon reacts → **file-as-truth +
  watcher event**.
- Opaque bytes / large / externally-shaped → **blob**, with an explicit note.

Anything that doesn't fit, or a new KV store, needs a written exception
(`CLAUDE.md`) — KV and disk-as-truth are the exceptions, not the default.
