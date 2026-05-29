---
title: Spaces
description: A space is the container everything lives in — closest to an application, and the boundary for isolation, context, and sync.
section: Concepts
order: 2
kicker: The mental model
tags: [concepts, spaces]
related: [what-is-ship]
---

A **space** is the container everything in Ship lives inside. The closest familiar thing is an **application** — but a space can scale up and hold sub-applications of its own.

A space is load-bearing in several ways at once:

- **Isolation** — each space is its own process boundary.
- **Agent context** — an agent's context is scoped to its space; agents don't reach across spaces.
- **Coordination & sync** — a space is the unit people share and the unit that syncs.
- **Its own surface** — a space renders in its own window on the host.
- **Its own contents** — views, pipelines, config, agents, and data all belong to the space that holds them.

So when you build with Ship, you're building inside a space: its views are your app's surfaces, its pipelines are your app's workflows, its agents and data are scoped to it. You can run many spaces, each its own isolated world, and a space large enough to need them can contain sub-applications.
