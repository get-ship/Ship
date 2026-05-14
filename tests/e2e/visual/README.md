# Visual baselines

End-to-end Playwright suite that exercises compile → daemon serve → SSR
splice → shell mount → hydrate for every view + a representative slice of
components, then asserts pixel-stable output against committed PNG
baselines. This is the gate for runtime, compiler, and view changes.

## Run

```sh
just dev            # daemon must be alive (port discovery via path_hash)
just visual         # diff current render vs committed baselines
```

`just visual` forwards extra args to Playwright, so the usual flags work:

```sh
just visual --grep "sui-button"
just visual --update-snapshots          # refresh baselines after an intentional change
just visual reactivity.spec.ts          # one spec only
```

## What's covered

- **Views** — every top-level entry in `views.json` rendered at
  `/sui/<name>` × { `paper`, `ship-dark` }.
- **Components** — curated slice of `sui-*` primitives rendered in a
  synthesized harness page (origin = daemon, so /\_sui/\* resolves) ×
  { `paper`, `ship-dark` }.
- **Reactivity smoke** — counter SSR markup contains the seed value;
  attach + click + reset produces the expected reactive update.

`ship-ui-studio` and views with unstable runtime output are excluded
from `views.json` — see the policy below.

## Adding / updating baselines

When a deliberate change moves pixels:

1. Inspect the diff PNGs in `tests/e2e/visual/output/` (the test reports
   the file path; the HTML report at `output/report/index.html` is the
   pretty view).
2. If the new render is correct: `just visual --update-snapshots`.
3. Commit only the PNGs you've eyeballed.

## Stable baselines only

The harness's job is to catch regressions on **working** views. If a
baseline capture renders a white page, an error overlay, or obviously
broken markup, do **not** commit that PNG — exclude the view from
`views.json` with a one-line comment instead. Committing broken baselines
means future regressions stay invisible.
