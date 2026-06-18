# Visual baselines

End-to-end Playwright suite that exercises compile → daemon serve → SSR
splice → shell mount → hydrate for every view, then asserts the
page is attach-stable (no console errors, no thrown page errors,
#sui-root has children). The pixel baselines in `baselines/` are
**manual artifacts** — the run that produced them is documented in
the commit log, but the test runner no longer auto-diffs against
them (the diff was a noise surface; the actual regression channel
is runtime errors at attach time, which the behavior spec catches).

## Run

```sh
just dev            # daemon must be alive (port discovery via path_hash)
just visual         # run the behavior suite
```

`just visual` forwards extra args to Playwright, so the usual flags work:

```sh
just visual --grep "agents"
just visual --update-snapshots          # refresh baselines (manual artifact path)
just visual reactivity.spec.ts          # one spec only
```

## What's covered

- **views.spec.ts** — every top-level entry in `views.json` rendered at
  `/sui/<name>` × { `paper`, `ship-dark` }. The test gates on
  `expect(root).not.toBeEmpty()` plus zero `console.error` events and
  zero `pageerror` events. A regression in the compile pipeline, the
  attach path, or any controller that throws during mount surfaces here.
- **reactivity.spec.ts** — counter SSR markup contains the seed value;
  attach + click + reset produces the expected reactive update.
- **agent-attach.spec.ts** — the agent-attach + xterm bridge round-trips
  PTY bytes through the SUI runtime.

`sui-*` component harnesses are no longer part of this suite; the
component matrix is covered by the unit tests in
`packages/components/sui/primitives/test/` (happy-dom under vitest).

## Pixel baselines (manual artifacts)

The `baselines/` directory retains the original PNGs from when the
spec did pixel diff. Use `just visual --update-snapshots` to refresh
after an intentional design change; eyeball the new render in
`output/` before committing. The behavior spec doesn't depend on
these files; updating them is a manual reviewer step, not a CI gate.

## Stable baselines only

The original baseline capture was a guard against regressions on
**working** views. If a baseline render shows a white page, an error
overlay, or obviously broken markup, do **not** treat the PNG as
authoritative — exclude the view from `views.json` with a one-line
comment and fix the view instead. (Same rule applies to the
behavior spec: a view that surfaces console errors during mount is
excluded by removing it from `views.json` until the regression is
fixed; keeping the entry would be a baseline-on-broken-glass.)
