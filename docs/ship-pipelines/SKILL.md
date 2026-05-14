---
name: ship-pipelines
stable-id: ship-pipelines
description: Reference docs for Ship pipelines — JSONC-declared DAGs of steps that the runtime walks automatically. Covers the file format, step types, resolution order, and the ship-jobs crate that owns the types.
tags: [ship, pipelines, extension, reference]
authors: [ship]
---

# Ship Pipelines

A pipeline is a JSONC file at `.ship/pipelines/<name>.jsonc` that declares a DAG of steps. The runtime walks the DAG when a job runs the pipeline — when a step completes, the orchestrator finds unblocked dependents and dispatches them.

- **[Overview](/pipelines/)** — concept, minimal example, when to reach for a pipeline vs a single agent.
- **[Reference](/pipelines/)** — file format, step types, resolution order, schema, short/long form, gate retries.
- **[ship-jobs crate](/pipelines/ship-jobs/)** — contributor-internal detail on the types and normalization that backs the user-facing pipeline file.

The orchestrator is what executes pipelines at runtime — see [Orchestrator](/runtime/orchestrator/) for how events flow through the DAG.
