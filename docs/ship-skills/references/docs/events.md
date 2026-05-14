---
group: Smart Skills
title: Platform Events
description: Ship built-in events, how they map to artifact types, and how the runtime routes events to agents.
audience: internal
section: reference
order: 4
---

# Platform Events

Ship events are platform primitives. They are not declared by skills — they are inferred from artifact types. When a skill produces HTML artifacts, Studio automatically enables feedback and selection events for that content.

## Ship built-in events

Built-in events live in the `studio.*` namespace — they are emitted by Ship Studio, the first app actor. There is no `ship.*` event namespace.

| Name | Direction | Triggered by |
|------|-----------|--------------|
| `studio.feedback` | in | User approved, rejected, or commented on an artifact. Payload: action (approve/reject/comment/request_changes), comment, artifact. |
| `studio.selection` | in | User selected text or content on a rendered surface. Payload: text, selector, artifact. |

Direction: `in` = human to agent. Agents already subscribe to `studio.*`, so they receive these without declaring any extra subscription.

Artifact lifecycle events — `artifact_created` and `artifact_deleted` — are **emit** topics, not subscription topics. An agent prefixes them with its skill ID at emit time (e.g. `canvas.artifact_created`); Studio subscribes to `{skill-id}.` to receive them. They are not part of an agent's inbound subscription set.

## Artifact type to event mapping

The platform maps each artifact type to the `studio.*` events that are meaningful for it. This mapping (`events_for_artifact` in `crates/core/ship-platform/src/agents/skill/artifact_events.rs`) documents which Studio actions apply to each artifact — it does not generate new subscriptions, since agents already subscribe to `studio.*`.

| Artifact type | Applicable Studio events |
|---------------|--------------------------|
| `html` | feedback, selection |
| `pdf` | feedback, selection |
| `markdown` | feedback, selection |
| `image` | feedback |
| `adr` | feedback |
| `note` | feedback |
| `url` | feedback |
| `json` | feedback |

`artifact_created` / `artifact_deleted` are intentionally absent here — they are emit topics, not subscription topics (see above).

Skills do not opt into events. They declare what they produce via the `artifacts` frontmatter field. The platform handles the rest.

## Actor-isolated event architecture

Events are routed through the KernelRouter. Each actor (agent, app, service) has its own event store and mailbox. Actors communicate through kernel-managed message passing, not shared storage.

### How routing works

1. Agent spawns as an actor with namespace `agent.{id}` and a mailbox subscribing to relevant namespaces.
2. Studio is an app actor with namespace `studio`, subscribing to `studio.*` and `agent.*`.
3. User annotates an HTML artifact in Studio → Studio emits `studio.message.visual` via its ActorStore.
4. KernelRouter routes the event to the agent's mailbox based on subscription.
5. Agent receives a `ship/event` SSE notification with the full payload. No follow-up queries needed — payloads are self-contained.

Agents do not have read access to the event store. They receive events through their mailbox only.

### Namespace boundaries

Every event type is namespaced. `RESERVED_NAMESPACES` blocks agents from emitting platform-controlled prefixes (`actor.*`, `session.*`, `studio.*`, `kernel.*`, etc.). Apps like Studio emit in their own namespace (`studio.*`) via their ActorStore.

### Visual messages

Studio supports visual feedback: users annotate HTML artifacts, add comments, draw on the canvas, then send everything as a single `studio.message.visual` event. Individual annotations are local UI state until the user explicitly sends. The agent receives one event with all annotations as a self-contained payload.

## Studio as the first app actor

Ship Studio is the first application on the runtime. It registers as an actor, owns the `studio.*` event namespace, and defines its own event types and UI interactions. Studio is a peer to agents, not a layer above them — both are actors managed by the kernel.

The event debug panel (toggle: `Ctrl+Shift+E` in dev builds) shows all events flowing through the actor's mailbox in real time.

## Stability

Stable in Ship 0.2.0:

- Per-actor event stores with namespace isolation
- KernelRouter with spawn/route/stop/snapshot/restore
- Studio as app actor with visual message flow
- Artifact-type-to-event mapping (skills declare artifacts, platform infers events)
- `ship/event` SSE notifications via actor mailboxes
- Actor snapshot and restore for migration

Not yet available:

- Service actors (sync, docs, auth)
- Ship SDK for external applications
- Typed artifact schemas with cloud docs API sync
- Custom app event types beyond Studio
