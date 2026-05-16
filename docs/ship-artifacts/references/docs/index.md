---
group: Artifacts
order: 1
title: Artifacts
description: Tracked files agents and pipelines produce. Addressable, queryable, renderable in views.
---

An **artifact** is a file the runtime tracks for you. Agents produce them, pipelines pass them around, and views render them. Each one has stable bytes plus metadata — name, type, size, where it came from — so the rest of Ship can find it again later.

## Three scopes

<sui-card padding="md">
<strong slot="header">Run</strong>
The default scope for agent and pipeline output. Auto-deleted after a TTL. Use for step output, scratch, logs.
</sui-card>

<sui-card padding="md">
<strong slot="header">Project</strong>
Persists for the life of the project. Use for documents, designs, and anything you want to keep across runs.
</sui-card>

<sui-card padding="md">
<strong slot="header">User</strong>
Personal storage that follows you across projects. Use for cross-project assets.
</sui-card>

You can promote a run-scope artifact to project or user scope when it's worth keeping.

## Where they show up

- **Pipelines.** Each step's output gets stored as run-scope artifacts. Downstream steps consume them by id.
- **Views.** Templates can render artifacts inline — markdown, images, PDFs, JSON. Drop the artifact id into a renderer and the view shows it.
- **Collections.** A `File` field on a collection schema is just a reference to an artifact. The artifact is the storage; the collection row is the metadata that makes it findable.
- **Chat with agents.** When an agent generates a file in conversation, it lands in the artifact store and can be referenced from anywhere else.

## What you don't have to manage

- **Storage paths.** The runtime decides where bytes live; you reference artifacts by id.
- **Garbage collection.** Run-scope artifacts expire automatically.
- **Deduplication.** Identical bytes in the same scope share underlying storage.
- **Authentication.** Local-only by default; the runtime checks scopes before serving.
