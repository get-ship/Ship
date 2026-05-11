---
group: Concepts
title: Overview
description: Why Ship exists and the pieces you compose with.
order: 1
---

Most apps put your data on someone else's machine and ask you to fit your work into the shape they thought of. Ship goes the other way: data is local, AI is first-class from the start, and you build the shape that fits the work.

The runtime and a small set of primitives make it possible to build custom interfaces — with your own data and deep AI integration — extremely quickly.

## What this lets you do

Define a custom type. Build a workflow where agents produce records of that type. Render a dashboard that watches the whole process — from initial design through review.

The same pieces handle the type, the workflow, the interface, and the review.

## The pieces

- **Agents** — portable identities. Skills, tools, and permissions wrapped into a profile that compiles to the CLI agent you're using.
- **Skills** — instruction sets with typed variables. One skill, personalized output per user and project.
- **Pipelines** — declarative multi-step work. Steps run in order or in parallel; gates pause for approval; runs are resumable.
- **Collections** — schema-backed structured data. Define types, query them, subscribe to changes.
- **Views** — live HTML surfaces over your data. Author a template; the runtime renders.
- **Artifacts** — files agents and pipelines produce. Tracked, addressable, renderable in views.
- **Triggers** — rules that watch for events and run actions when they fire.
