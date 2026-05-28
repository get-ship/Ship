# @ship/eval-harness

Measure how **SKILL.md tone + structure + lint enforcement** affect agent
authoring quality on SUI views and components. Dispatches agents against
a corpus of (prompt, target) pairs, captures the artifacts the agent
produced, and evaluates them against compile + lint + structural reference.

> **Why this exists.** The arcade-game-agent retro (#176/#177/#179) was a
> sample-size-one signal that wrong-exemplar copying + type-system lies +
> silent-failure tooling cause real productivity loss. After tightening the
> tone (#174), restructuring for progressive disclosure (#175), and
> promoting lints to Error (#166), we need a measurement that says whether
> those tightenings actually move the needle for the NEXT agent. This is
> that measurement.

## The matrix

Each cell of the eval is a triple:

```
(prompt, SKILL.md variant, agent profile) → artifact + score
```

- **prompt** — `corpus/<id>/prompt.md` — natural-language ask the agent
  must satisfy ("build a `<sui-counter>` component", "build a `/tasks`
  view listing `ship.skills.summaries()` with retry").
- **SKILL.md variant** — which version of the SUI authoring skill the
  agent loads. Test (a) current main + cardinals (b) cardinals stripped
  (control) (c) old verbose tone (regression test).
- **agent profile** — which model + system-prompt configuration. Test
  whatever Ship ships agents with.

## The score

For each generated artifact the harness runs:

1. **Compiles?** — `cargo run -p sui-compiler -- build <path>`. Pass/fail.
2. **Lint clean?** — sui-lint findings; Error-severity blocks compile,
   Warn-severity contributes to score but doesn't fail.
3. **Structural match?** — diff against `references/<id>/expected.*`:
   - For components: tag name, declared props, declared slots, event
     emissions present in CEM.
   - For views: manifest fields, declared `<data>` sources, declared
     state keys.
4. **Renders?** — pending #164 (webview test harness). For v1 we skip
   this and rely on compile + structural as a proxy.

Output: `results/<run-id>.json` — per-cell pass/fail + the lint findings
table + the structural diff.

## Corpus

Each case under `corpus/<id>/`:

- `prompt.md` — the agent-facing ask. Includes the expected output path
  ("write the file at `packages/components/primitives/src/<tag>/<tag>.sui`").
- `meta.json` — case metadata: target kind (`view` | `component`),
  reference id, score weights.

Cases land here as we collect agent-feedback patterns. Start small (3-5
canonical cases) — over-corpus is worse than no corpus because it lies
about coverage.

## References

`references/<id>/expected.*` holds the **structural ground truth** —
NOT a verbatim file to copy. We compare on signal:
- presence of cardinal directives (`<prop>`, `<data>`, `<state>`)
- shape of controller exports (`scope:`, `derived:`, `on:`, `mount`)
- absence of anti-pattern shapes (`fetch` in controller, `innerHTML`,
  `state.x.set(v)`, etc.)

The reference is what a competent human agent would write — not what
any specific past agent did write.

## Running

```sh
# Single case, default agent + current SKILL.md
bun run eval -- --case sui-counter

# Full matrix
bun run eval:matrix -- --cases all --skills cardinals,stripped,verbose
```

Results land in `results/`. Each run produces:

```
results/<run-id>/
  manifest.json    # the matrix exec config
  cells/<n>.json   # per-cell artifact + scores
  summary.md       # human-readable rollup
```

## Limitations (honest)

- **Render verification waits on #164.** The webview test harness is the
  honest way to know if a mounted view actually paints. Until then, we
  proxy with compile + structural — which is a real signal but not a
  full one.
- **Sample size starts small.** A 3-case corpus is fine to iterate on
  the harness itself. Trust the score curves only after the corpus
  covers each major authoring shape: simple component, stateful
  component, list view, master-detail view, mutation view.
- **Agent dispatch is the slowest dimension.** Each cell can take
  minutes. The matrix mode caches by (prompt, skill, agent) hash so a
  re-run only re-executes cells that changed.

## Status

**v0 scaffolding.** The corpus + reference shape is documented; one
seed case lands here (`sui-counter`). The runner is stubbed — it knows
how to read a case + invoke an agent + score the artifact, but the
agent-invocation step needs the actual binding to the agent-of-record
(Anthropic SDK direct, or via `spawn-agent`'s dispatch). See
`runner/agent.ts` for the seam.
