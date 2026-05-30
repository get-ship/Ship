---
group: Architecture
order: 7
title: State ownership & sync
description: Where each kind of Ship state lives, who owns the authoritative copy, and how it syncs across devices and team members. Canonical classification; design notes flagged as such.
audience: internal
---

# State ownership & sync

Ship's runtime is **event-sourced**: the event log is the source of truth for
**operations** on a space. The questions this doc settles: what is an operation
(in the log) vs. what legitimately lives elsewhere, and how each kind syncs.
Getting this wrong is the source of "mixed message" bugs. Code and docs should
not contradict the classification; the **sync mechanics are partly design** and
marked accordingly.

## The classes

| Class | Authoritative copy | Multi-writer | Sync |
|---|---|---|---|
| **Event log** (event-sourced) | the log + projections | append-only; interleave by HLC | cloud relay push/pull, one PlatformDO per space |
| **`.ship/` files** | the file on disk | last edit wins (it's source) | the **package format** (content-addressed, signed); **git is an opt-in, supplementary** integration — never assumed, never load-bearing |
| **Collections** (`data.db`) | the per-item **CRDT** (Automerge) doc | concurrent edits **converge** | mutations ride the event transport; `data.db` is a rebuilt projection |
| **Artifacts** (blobs) | content-addressed bytes | immutable (hash = identity) | object-store **diff** (hash exchange) |
| *derived* | — | — | not synced — recomputed (package cache, build outputs) |

## What opts OUT of the event log, and why

The log is for **operations / facts** — things that happen and need history,
audit, coordination, or replay. State opts out when it is not that:

- **`.ship/` files** — *authored source*, not operations. A human/agent edits
  them; the `fsnotify` watcher only *observes* and emits a stamped `*.changed`
  **signal** (replay does not rebuild the file). This is the event-*observed*
  vs event-*sourced* line.
- **Collections data** — *bulk, multi-writer user data*. An append log you
  replay is the wrong primitive for converging concurrent edits; a per-item CRDT
  is. (Mutations still ride the event transport, but the data's truth is the
  CRDT doc.)
- **Artifacts** — *opaque bytes*. Content-addressed object storage, not events.
- **Global daemon settings / skill-vars (KV)** — *machine-local infra config*
  (terminal program, daemon port) and the L1 skill-var primitive. Not
  operations, not synced — deliberate KV stores.
- **Derived** — the package cache and compiled outputs are recomputable.

Rule: event-source what is an operation; keep out content (files/blobs),
multi-writer structured data (collections → CRDT), machine-local config (KV),
and derived state. New KV / disk-as-truth needs a written exception
(`CLAUDE.md`).

## Event tiers — sync policy & retention (DESIGN — not yet implemented)

Today every event lives in one space-partitioned log (`space_id`, #116); there
is **no** sync-policy or retention dimension yet. The sync work needs two axes
per event (or event-type):

- **Sync policy** — `synced` (crosses devices/members; the space's portable
  history: settings, grants, members, sessions, pipeline-runs, collection
  mutations, artifact references) vs **`local`** (never leaves the machine:
  window actors are pid-bound and meaningless elsewhere; peer-cred sessions;
  machine KV).
- **Retention** — `durable` (permanent space history) vs **`ephemeral`**
  (high-volume runtime signals — heartbeats, liveness probes, PTY byte streams,
  `*.changed` watcher signals, hook firings, telemetry — kept briefly, never
  synced).

This is the load-bearing gap for cross-machine sync: without it, a second device
would either drown in ephemeral noise or sync device-local state that means
nothing there. (#91/#92 + the sync protocol own this.)

## What happens when you sync (DESIGN — relay not built)

Principle: **one event-log shape, scaled by writers** — solo-multi-device,
team, and cloud-worker are the same flow with different participant counts.

- **One space across two of your devices:** each device holds a full local
  replica and reads locally. `synced+durable` events flow through the cloud
  relay (push local, pull remote, ordered by **HLC**); collections converge via
  Automerge; artifacts replicate by hash-diff. No device is authoritative — the
  cloud appends + fans out, it is not a server-side model.
- **A space shared with 15 members all updating:** identical machinery, more
  writers on one append-only log. Events are facts → concurrent writes don't
  conflict, they interleave by HLC. Collection edits converge per-item.
  **Who may write what** is gated by **macaroons** (capability delegation), not
  the data model. Unresolvable conflicts surface as observable rejection events,
  never silent loss.

## Status — how far from two machines

The skeleton exists; the coherence machinery does not. **Have:** space-
partitioned log (#116), a `sync/client.rs` push/pull HTTP scaffold, gateway sync
routes + per-space PlatformDO/WorkspaceDO (zero-logic relay). **Missing
(load-bearing):** HLC cursors (today an opaque string), the sync-policy/retention
tiers above, `observe_external` (pulled events do **not** reach the kernel yet),
device identity + macaroon auth on sync, the Automerge write path for
collections, and #91/#92 truth-purity (so a second machine rebuilds state from
the synced log rather than from disk/KV-canonical copies). Mapped, not
greenfield — but a real program, not a wire-up.

## One content hash

There is one content hash — `compute_tree_hash` over a `.ship` tree — shared by
the package cache key, the registry `contentHash`, and bundle attestations.
