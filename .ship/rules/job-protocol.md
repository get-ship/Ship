# Job Protocol

On start, check `.artifacts/job-spec.md`. If found, execute immediately unless `## Mode: interactive`.

When invoked as a pipeline step (`SHIP_ACTOR_ID` set, inside `ship run`), signal step completion over the daemon RPC surface:

```
ship ops call pipelines.step_done --json '{"slug":"<actor>","summary":"<one-liner>","outputs":{}}'
```

`pipelines.step_done` advances the run to the next step and is the canonical completion signal every dispatched agent is expected to call when its work is verifiably complete. There is no separate completion CLI — everything flows through `ship ops call`. Narrate progress and blockers in your output and commit messages. Never exit silently.
