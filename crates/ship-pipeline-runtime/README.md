# ship-pipeline-runtime

**Layer:** L1 — pipeline spec CRUD and the job-advancement / step-dispatch / fanout / sweep runtime.

**Note: this crate is mid-dissolution (R13).** The P0 contract band (P0.1–P0.4) is merged; P1.0a/b journal-persist seams are merged. Remaining work (P2.1–P2.5 dissolution, P3.0 cleanup) is in progress. Do not build new features on this crate without checking the current R13 status.

## What it contains

- `pipelines` — `PipelinesService`: CRUD over `<scope>/pipelines/*.jsonc` spec files.
- `orchestrator` — the run runtime: handlers, router, `run_step`, `review_retry`, sweep, fanout, replay, dispatch, `pipeline_loader`, `pipeline_specs`, step I/O, projection reader, spawn policy, prompt composer, gate rules, git helpers.
- `orchestrator::emit` — re-export of the L0 `ship-runner` emit mechanism.
- `orchestrator::dedup`, `orchestrator::exit_handler` — re-exports of the L1 `ship_code::runner` mechanism (historic import paths preserved).
- `projections` — `PipelineRunProjection` and `PipelineStepRunProjection` (SQL-backed read models).
- `wire` — ConnectRPC handler for the `PipelinesService` shipd mount.

## Key facts

The orchestrator owns no `orchestrator.*` events and is not a service in the shipd registry. It advances runs by authoring `pipeline.*` events. Install projections at app startup via `ship_pipeline_runtime::install_projections()`.
