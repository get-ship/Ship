---
name: ship-onboarding
stable-id: ship-onboarding
description: Read this FIRST when you are new to the Ship codebase. The high-level grounding — the four-layer model, the scope model, fat-substrate/thin-daemon, and where to go next. Orients before you touch anything.
tags: [ship, onboarding, grounding, architecture]
authors: [ship]
audience: internal
---

# What Ship is

Ship is an agent-configuration compiler and a runtime for coordinating
multiple agents. It compiles declarative config in `.ship/` into harness-native
files (Claude, Codex, Cursor, opencode) and runs a daemon that dispatches
agents, advances pipelines, and event-sources the result. Read this page once
to get oriented, then follow the pointers — do not try to learn the whole thing
up front.

## The four layers

Every crate sits in exactly one layer; dependencies point downward only.

- **L0 — substrate.** The event log, persistence, the kernel (event routing),
  the supervisor. Truth lives here. (`crates/core/ship-substrate`, `ship-kernel`)
- **L1 — platform primitives.** Domain services over the substrate: agents,
  collections, artifacts, registry, sessions, the compiler, the orchestrator.
  (`crates/core/ship-platform`)
- **L2 — built-in apps.** Ship's own apps. **No privileged access** — they
  consume the same SDK as L3, never `crate::services::*` shortcuts.
- **L3 — user apps.** Anything authored in a space. Same SDK as L2.

## The scope model

Three scopes, and a single rule for each: **who owns the authoritative copy is
who syncs it.** Ownership = sync.

- **Space.** A directory with a `.ship/` inside it — the unit of
  isolation/coordination/config (views, agents, skills, pipelines). One daemon
  serves many spaces; the emit path resolves `space_id` per connection so each
  space partitions into its own log. Space settings are event-sourced + synced.
- **Account.** The identity/tenant that owns spaces. Account-scoped state syncs
  with the account.
- **Runtime.** The running machine/daemon. Runtime owns *execution* — sessions,
  actors, process lifecycle — in `runtime.db`. Machine-local; not synced.

Agents are **actors** (an L0 lifecycle concept); an agent *profile* is L1 config
that compiles to provider files. Don't conflate the two.

## Fat substrate, thin daemon

The substrate (L0) holds the logic and the truth: the event log is the source
of truth for operations on a space; disk is a projection. The daemon and the
CLI/MCP are **thin transport** — they marshal requests onto the op surface and
stream results back. Rust logic lives in the runtime, never in the transport
shell. One contract per boundary, one source of truth.

## Where to go next

- **Rules (read before writing code):** `.ship/rules/agent-discipline.md`,
  `architecture.md`, `engineering.md`, `job-protocol.md`. These are binding.
- **How it actually fits together:** the `ship-architecture` doc-skill
  (`docs/ship-architecture/`) — crate map, design decisions, the spaces and
  state-ownership detail.
- **Decisions already made (don't relitigate):** ADRs in `docs/ship-adrs/`
  (e.g. `scope-partitioned-substrate.md`), and the experimental `memento` skill
  (`.ship/skills/memento/`) for the running log of corrections and rejected ideas.
- **How we document:** the `ship-docs` skill — where a fact belongs (code
  comment / README / ADR / memento / nothing) before you write any prose.
