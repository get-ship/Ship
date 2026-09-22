---
title: What is Ship
description: Ship in one page — what it is, who it's for, how you start, and what it runs for you.
section: Concepts
order: 1
kicker: The mental model
tags: [concepts, overview]
related: [local-first]
---

**Ship is an application runtime for AI-native apps.** You describe what your app should do — its data, its agents, its surfaces, its rules — and Ship runs it. Local-first, with cloud sync as an optional layer.

## Who it's for

People and small teams building apps for personal or internal use — not consumer startups, not one-shot demos. Ship pre-decides the boring choices (framework, database, deployment, auth) so the parts you describe are the parts that make the app yours.

## How you start

Open Ship and Studio launches with a space waiting. Describe what you want, in plain language:

> "A reading list that summarizes articles I add."
>
> "An expense tracker that pulls categories from receipts I upload."

A first-party agent reads your description and scaffolds a real app — **collections** for the data, **agents** for the work, **pipelines** to connect them, **views** for the surface, **permissions** for the boundaries. The pieces appear in your space as you watch.

The app runs the moment it's scaffolded. Edit a view's template and it hot-reloads. Ask the agent for a feature and new pieces appear. Hand the space to a friend and they run the same app.

That's the 80% path. The other 20% is hand-authoring components directly — same primitives, same runtime.

## What an app is

Ship apps live inside a **[space](/docs/spaces/)** — the application-shaped container that holds everything, on your machine. Inside a space you mix and match declarative **components**:

- **Views** — the surfaces people interact with
- **Agents** — LLM-driven workers with personas, skills, and tools
- **Skills** — reusable units of capability agents call
- **Pipelines** — orchestrations of agent and actor steps
- **Collections** — schemaful structured data
- **Files** — blob storage with metadata
- **Permissions & rules** — what the app and its agents can and can't do
- **MCP integrations** — connections to external systems

You write components as config, not code. The runtime knows how to execute them, and edits are hot — change a file, the running system reflects it. No build step.

## What Ship runs for you

- **Storage** — key-value for small values, Files for blobs, Collections for records, Events for runtime state. All four flow through one event log, so sync works uniformly.
- **Compute** — agents and pipelines run as supervised processes. Crashes are handled, conversations persist across restarts, and replay is built in.
- **Observability** — every meaningful state change is an event. Replay any session, audit any action, debug by scrubbing the log.
- **Sync** — the optional cloud layer replicates the event log between machines so teams can share a space. Solo, fully local, it works the same minus sharing.

## The primitives

| Tier | Primitive | What it is |
|---|---|---|
| **Storage** | KV | Bounded key-value store — app state, settings |
| | Files | Blob storage with metadata — uploads, generated artifacts |
| | Collections | Schemaful structured data — the records your app holds |
| | Events | Append-only log plus derived projections — the runtime's spine |
| **Compute** | Agents | LLM-driven workers with profile, skills, tools, permissions |
| | Pipelines | DAGs of agent and actor steps |
| **Config** | Skills | Reusable capability units agents call |
| | Rules & permissions | Behavioral constraints and what's allowed |
| | Views | UI surfaces, authored as `.sail` templates, served by the runtime |

> **Local-first** — the runtime, your databases, and your data all live on your machine. Cloud sync is a property added on top, not a hosting model that owns your data. [More →](/docs/local-first/)

Studio is a desktop app today. Everything you build runs against the same local daemon, whether you author in Studio, in your editor, or by asking an agent.
