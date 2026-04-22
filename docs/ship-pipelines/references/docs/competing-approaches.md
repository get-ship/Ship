# Competing Approaches

Run K agents in parallel against the same spec, then a judge step picks or merges the best result.

## When to use

Use the `fanout:` primitive when:

- The design decision is genuinely uncertain (auth flow, API shape, data model).
- Output quality matters enough to justify the compute cost.
- You want an explicit arbitration step rather than accepting the first agent's answer.

Typical uses: architecture proposals, implementation approaches, copy alternatives, test suites.

## YAML shape

```yaml
steps:
  - id: draft
    type: agent
    agent: implementer
    fanout:
      lanes:
        - id: lane-claude
          agent: implementer
          model: claude-opus-4-7
        - id: lane-sonnet
          agent: implementer
          model: claude-sonnet-4-6
        - id: lane-gemini
          agent: implementer
          model: gemini-pro
      judge:
        step: choose
      min_successful_lanes: 1   # default; how many lanes must succeed
    produces:
      - name: result
        scope: run

  - id: choose
    type: review
    after: [draft]
    agent: judge
    consumes:
      - lane-claude.result
      - lane-sonnet.result
      - lane-gemini.result
    produces:
      - name: chosen_result
        scope: run
        required: true
```

## How inputs flow

1. The fanout step's `consumes` are resolved once from upstream steps. All lanes receive the same resolved inputs at `.artifacts/input/`.
2. Each lane runs as an independent agent in its own isolated worktree.
3. Each lane calls `step_done` with `outputs: { "<name>": "<artifact_id>" }` for every declared `produces` entry.
4. Lane outputs are namespaced: lane `lane-claude` producing `result` is accessible as `lane-claude.result`.
5. The fanout step succeeds when at least `min_successful_lanes` lanes have completed successfully.

## How the judge receives lane outputs

The judge step's `consumes` references use the `<lane_id>.<output_name>` notation. Before the judge dispatches, the orchestrator resolves each scoped reference from the producing lane's `step.done_declared` event and writes it to `.artifacts/input/<lane_id>.<output_name>`.

The dispatch prompt groups fanout inputs by lane:

```
## Inputs from fanout `draft`

  lane-claude:
    - result — id: 01J..., path: .artifacts/input/lane-claude.result
  lane-sonnet:
    - result — id: 01J..., path: .artifacts/input/lane-sonnet.result
  lane-gemini:
    - result — id: 01J..., path: .artifacts/input/lane-gemini.result
```

The judge agent reads all lane outputs, compares them, and uploads its chosen (or synthesized) result as its own artifact.

## Failure semantics

| Situation | Outcome |
|-----------|---------|
| One lane fails, others succeed | Fanout advances if `ok_count >= min_successful_lanes` |
| All lanes fail | Fanout step fails, judge does not dispatch |
| Judge fails | Same as any other step failure |

Failed lanes are silently skipped — the judge sees only the lanes that produced outputs. If you need the judge to know which lanes failed, set `min_successful_lanes` higher than 1 and the pipeline will halt before dispatch.

## Configuration reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `fanout.lanes` | `FanoutLane[]` | required | Parallel execution lanes |
| `fanout.judge.step` | `string` | required | Step ID of the judge |
| `fanout.min_successful_lanes` | `uint` | `1` | Minimum lanes that must succeed |
| `lane.id` | `string` | required | Unique within this fanout |
| `lane.agent` | `string` | required | Agent profile for this lane |
| `lane.model` | `string?` | — | Model override (falls back to step, then job) |
| `lane.provider` | `string?` | — | Provider override (same fallback) |

## Non-goals

- **Judge-as-primitive.** The judge is a normal `review` step. No special step type.
- **Automatic merging.** The judge agent decides: pick, merge, or synthesize. Ship does not auto-merge lane outputs.
- **Cross-pipeline fanout.** All lanes share the pipeline run. Cross-pipeline orchestration is out of scope.
- **Dynamic lane count.** Lanes are declared statically in the pipeline YAML.
