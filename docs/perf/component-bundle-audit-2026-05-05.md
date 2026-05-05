# `sui-components.js` bundle audit — 2026-05-05

Snapshot taken on `job/bundle-perf-audit` against the build-bundles output of
`crates/ship-ui-framework/assets/sui-components.js`. Methodology: re-bundle
with `esbuild --metafile`, attribute `bytesInOutput` per input file to the
component whose `dist/<tag>.compiled.js` or `<tag>.controller.ts` it came from,
sum, then prorate the gzipped total by raw share.

Raw stats and per-component attribution are checked in at:
- `.artifacts/sui-components.stats.json` (esbuild metafile)
- `.artifacts/sui-components.per-component.json` (per-tag rollup)

Reproduce with `node .artifacts/audit-bundle.mjs` (after a normal
`node scripts/build-bundles.mjs` run, so the `dist/*.compiled.js` files
exist).

## Headline numbers

| metric | value |
|---|---|
| total raw bundle | 865.5 KiB |
| total gzipped bundle | 134.8 KiB |
| component count (registered tags) | 87 |
| current gz avg / component | 1586 B (target ≤ 800 B; 2× over) |
| top-10 contribution | 281.0 KiB raw → 45.6 KiB gz (≈34 % of bundle) |
| bottom-21 contribution | 67.1 KiB raw → 10.9 KiB gz (≈8 % of bundle) |

Per the existing observability gate in `scripts/build-bundles.mjs` the
"avg ≤ 800 B/component" target is aspirational; the bundle has been over it
for some time. The audit confirms the drift is structural — a long-tail of
small components is fine, but a small number of feature-heavy components
(command-bar family, devtools family) account for most of the budget.

## Top 10 offenders

Bytes are raw input contribution from the metafile; "gz share" = raw share ×
total gzipped bytes (proportional estimate, not a per-tag re-gzip).

| # | tag | raw B | gz share B | dir | files |
|---:|---|---:|---:|---|---|
| 1 | `sui-command-palette` | 61 668 | 10 008 | `sui-command-bar/` | compiled 52.2 KB + controller 8.4 KB |
| 2 | `sui-toast` | 28 472 | 4 621 | `sui-toast/` | compiled 23.3 KB + controller 5.2 KB |
| 3 | `sui-theme-picker` | 26 581 | 4 314 | `sui-theme-picker/` | compiled 21.3 KB + controller 5.3 KB |
| 4 | `sui-devtools-source` | 26 129 | 4 240 | `sui-devtools/` | compiled 19.1 KB + controller 7.0 KB |
| 5 | `sui-command-chat` | 25 950 | 4 211 | `sui-command-bar/` | compiled 22.0 KB + controller 2.9 KB |
| 6 | `sui-table` | 25 228 | 4 094 | `sui-table/` | compiled 12.8 KB + controller 12.4 KB |
| 7 | `sui-launcher` | 23 259 | 3 775 | `sui-launcher/` | compiled 7.0 KB + controller 8.5 KB |
| 8 | `sui-panel-dock` | 23 150 | 3 757 | `sui-panel-dock/` | compiled 17.0 KB + controller 6.2 KB |
| 9 | `sui-command-bar` | 22 900 | 3 716 | `sui-command-bar/` | compiled 7.6 KB + controller 14.3 KB |
| 10 | `sui-devtools-schema` | 19 204 | 3 117 | `sui-devtools/` | compiled 12.7 KB + controller 6.2 KB |

Two clusters dominate:

- **command-bar family** — `sui-command-bar` + `sui-command-palette` +
  `sui-command-chat` together: 110.5 KiB raw, ~17.9 KiB gz. Roughly 13 % of
  the entire components bundle for three components used only by the global
  command surface.
- **devtools family** — five `sui-devtools-*` tags total 91.8 KiB raw,
  ~14.9 KiB gz, i.e. another 11 %. None of these run on user-facing pages.

Together those eight components are 24 % of the gzipped bundle and ship to
every page even when no view uses them.

## Why each top offender is large

For each top offender we measured the share of the compiled `.js` that is the
inlined CSS string (`styles: "..."`) vs. the imperative bind/effect template
code emitted by `sui-compiler`:

| tag | compiled B | style-string B | style % |
|---|---:|---:|---:|
| sui-command-palette | 43 727 | 7 563 | 17 % |
| sui-toast | 21 361 | 8 682 | 41 % |
| sui-theme-picker | 18 483 | 8 778 | 47 % |
| sui-devtools-source | 16 444 | 2 187 | 13 % |
| sui-command-chat | 18 392 | 4 925 | 27 % |
| sui-table | 10 801 | 1 608 | 15 % |
| sui-launcher | 6 368 | 2 976 | 47 % |
| sui-panel-dock | 15 048 | 4 169 | 28 % |
| sui-command-bar | 5 887 | 1 639 | 28 % |
| sui-nav-top | 11 487 | 2 320 | 20 % |
| sui-list-tile | 10 363 | 4 492 | 43 % |
| sui-graph-canvas | 5 702 | 3 411 | 60 % |
| sui-markdown | 3 122 | 1 483 | 47 % |

Three patterns explain the long tail:

1. **Compiled-template code emitted by `sui-compiler`.** Each `.sui` file
   becomes a tree of `bindAttr / bindClass / bindFor / bindIf / bindText / on`
   calls that build elements via `document.createElement`. Templates with deep
   structure (sui-command-palette: 474 lines of `.sui` → 52 KB compiled)
   produce a lot of imperative code. Style strings inflate it further:
   `sui-graph-canvas` is 60 % CSS, `sui-toast` 41 %, `sui-theme-picker` 47 %.
2. **Controllers with embedded business logic.** `sui-table` (12.4 KB
   controller, 392 lines), `sui-command-bar` (14.3 KB / 395 lines) and
   `sui-launcher` (8.5 KB / 262 lines) carry sort/filter/spawn logic inside
   the controller. Nothing in the controllers is exotic — they are just
   medium-large client modules that ship even on pages that never instantiate
   them.
3. **No tree-shaking past the registry.** `packages/ui-components/src/index.ts`
   is a 87-line list of side-effect imports
   (`import './sui-foo/dist/sui-foo.compiled.js'`). Each one calls
   `customElements.define()` at module load, which esbuild conservatively
   keeps. Any page that pulls `sui-components.js` therefore pays for every
   component, regardless of which `<sui-*>` tags it actually uses.

## Cross-bundle deps

The metafile shows two non-`ui-components` inputs leaking into the bundle:

| input | raw B | imported by |
|---|---:|---|
| `packages/ship-set/src/web-components/core/lucide-icons.generated.ts` | 12 848 | `sui-icon.controller.ts` (the entire icon registry — ~2 KB gz) |
| `packages/ship-set/src/themes.ts` | 3 546 | `sui-theme-picker.controller.ts` (theme name list + shared types) |

`@ship/ui-runtime` is properly externalised and contributes 0 B. The
`ship-set` reach-across is a soft dependency violation — components depending
on a static asset/data table that lives outside the package — and shows up as
unconditional bytes in the components bundle.

The bundle has zero `node_modules/` inputs, so there is no third-party-package
fat to shed.

## Distribution

| size bucket | count | raw total | gz share |
|---|---:|---:|---:|
| > 20 KB | 9 | 263.3 KiB | 41.7 KiB |
| 10–20 KB | 18 | 243.3 KiB | 38.6 KiB |
| 5–10 KB | 39 | 276.5 KiB | 43.8 KiB |
| < 5 KB | 21 | 67.1 KiB | 10.6 KiB |

The 9-component head accounts for ~31 % of the gzipped bundle. Removing the
21-component tail entirely would only save ~8 %. Splitting must target
the head.

## Recommendations

The acceptance criteria asks for at least three actionable recommendations.
These are ordered by expected gz-savings vs. effort.

### R1. Move command-bar + devtools families to islands (largest single win)

`packages/ui-components/src/islands/<tag>/` is already the established
on-demand-load lane (sui-terminal lives there). Each island is a separate
bundle the daemon serves on demand, with `@ship/ui-runtime` and `@ship/ui-dom`
externalised so the shared runtime loads once.

Candidates:

- `sui-command-bar`, `sui-command-palette`, `sui-command-chat` — used only
  inside the command surface mounted by the shell. Move all three together
  (they share controllers and styles internally).
  Estimated savings: **~17.9 KiB gz** off `sui-components.js`.
- `sui-devtools-source`, `sui-devtools-components`, `sui-devtools-lint`,
  `sui-devtools-schema`, `sui-devtools-signals` — only mounted by the
  devtools panel.
  Estimated savings: **~14.9 KiB gz**.
- `sui-launcher`, `sui-panel-dock`, `sui-nav-top` — chrome components
  rendered by the shell, not by user views.
  Estimated savings: **~10.3 KiB gz**.

Combined ceiling for R1: **~43 KiB gz** off the every-page bundle, dropping
sui-components.js to roughly 92 KiB gz (`avg ≈ 1.1 KB`/component).

Cost: each migration involves moving the source under `islands/<tag>/`, adding
an island bundle entry to `scripts/build-bundles.mjs` (the loop already
auto-discovers `src/islands/*/`), and updating the shell to register the
island when the relevant surface mounts. No runtime changes.

### R2. Split the registry into `sui-core` + `sui-extended` bundles

The `index.ts`-as-side-effect-registry pattern is the reason every page pays
for every component. Two-bundle split:

- `sui-core` — primitives almost every page uses: `sui-icon`, `sui-button`,
  `sui-input`, `sui-textarea`, `sui-checkbox`, `sui-switch`, `sui-radio*`,
  `sui-segmented*`, `sui-toggle*`, `sui-tabs`, `sui-tab`, `sui-card`,
  `sui-badge`, `sui-chip`, `sui-link`, `sui-divider`, `sui-separator`,
  `sui-loading`, `sui-spinner`, `sui-skeleton`, `sui-stat`, `sui-empty-state`,
  `sui-error-state`, `sui-alert`, `sui-tooltip`, `sui-dialog`, `sui-drawer`,
  `sui-popover`, `sui-toast*`, `sui-progress`, `sui-status-*`, `sui-pagination`,
  `sui-breadcrumb*`, `sui-label`, `sui-field`, `sui-mark`, `sui-kbd`,
  `sui-avatar`, `sui-collapsible`. Expected ≤ 60 KiB gz.
- `sui-extended` — feature-rich tags loaded on demand: `sui-table`,
  `sui-graph-canvas`, `sui-markdown`, `sui-diff`, `sui-code`, `sui-json-view`,
  `sui-tree-view*`, `sui-timeline*`, `sui-stepper`, `sui-context-menu`,
  `sui-dropdown-menu`, `sui-hover-card`, `sui-accordion*`, `sui-list-tile`,
  `sui-annotation*`, `sui-edit-overlay`, `sui-rail*`, `sui-app-icon`,
  `sui-status-bar`, `sui-toolbar`, `sui-theme-picker`, `sui-metachip`,
  `sui-timestamp`. Expected ≤ 35 KiB gz.

Loading: views declare which bundle they need (most views: core only;
agents/git/devtools-heavy views: core + extended). The compiler already knows
which `<sui-*>` tags appear in each view's `.sui`, so the bundle-set decision
is mechanical and could be emitted into the per-view manifest.

This is more invasive than R1 (touches the daemon's asset-serving and the
view manifest), and overlaps with R1 — if R1 lands first, R2 becomes a
narrower core/extended split of what's left.

### R3. Hoist shared CSS into `suiTonesSheet`-style adoptable stylesheets

Each compiled component carries its own `styles: "…"` string. Across the
whole bundle that string totals roughly 50–80 KB raw. After gzip the savings
shrink (CSS is repetitive across components and gzip already shares between
them well), but several patterns are duplicated literally:

- focus-ring rules, `:host([disabled])` rules, `.sr-only` blocks
- the same `var(--radius-*)`, `var(--shadow-*)`, `var(--border)` declarations
- modal/popover backdrop styles in `sui-dialog`, `sui-drawer`, `sui-popover`,
  `sui-hover-card`, `sui-context-menu`, `sui-dropdown-menu`,
  `sui-command-palette`

The runtime already exposes `suiTonesSheet` (a constructable stylesheet
shared across components — visible as an import in compiled outputs). Adding
two or three more shared adoptable sheets (focus, surface/elevation,
overlay) and having the compiler skip writing those rules into per-component
`styles:` strings would reclaim a chunk of the per-component constant cost.

Estimated savings: **3–6 KiB gz** depending on how aggressively common rules
are hoisted. Smaller than R1/R2 but lands on every component, including the
ones that stay in `sui-core`.

### R4 (optional). Lazy-load the lucide registry

`sui-icon` pulls 12.8 KiB raw of `lucide-icons.generated.ts` into the bundle
unconditionally. The registry is a static `Record<string, string>` and never
changes between sessions. Two safer patterns:

- emit the registry as `crates/ship-ui-framework/assets/lucide-icons.json` and
  have `sui-icon.controller.ts` fetch it on first mount (cached after).
- compile-time tree-shaking: walk every `.sui` for `name="…"` on `<sui-icon>`
  and emit a per-page subset. (Bigger lift; rejected as not "trivial".)

Either approach removes the cross-package leak from R0/recommendations and
saves ~2 KiB gz on every page.

## Trivial wins applied in this job

None landed in this job — the audit did not surface a "wrong dep imported
for one helper" pattern. The closest candidate was the lucide-icons leak
(R4), but rerouting it to a fetch-on-mount touches the icon component's
public mount semantics (sync vs. async first paint) and is therefore not
trivial enough to ride along with an audit. Filed as follow-up.

## Suggested follow-up specs

1. `job/sui-extract-command-bar-island` — implements R1 for command-bar.
2. `job/sui-extract-devtools-island` — implements R1 for devtools-*.
3. `job/sui-core-extended-split` — implements R2 (depends on islands work).
4. `job/sui-shared-stylesheets` — implements R3 in `sui-compiler` +
   `@ship/ui-runtime`.
5. `job/sui-icon-lazy-registry` — implements R4.

## Acceptance check

- [x] Report exists at `docs/perf/component-bundle-audit-2026-05-05.md`.
- [x] Top-10 list identifies actual offenders with byte numbers.
- [x] At least 3 specific, actionable recommendations (R1–R4).
- [x] `cargo test -p sui-compiler` green — no production code changed in
      this job. (See verification below.)

## Verification — R1 applied (job/bundle-perf-followup, 2026-05-05)

Executed R1 in three steps; each step was its own commit + bundle-script
re-run, with `cargo test -p sui-compiler` and `pnpm -F @ship/ui-components
test` green at each step.

| step | change | sui-components.js gz | island bundle gz |
|---|---|---:|---:|
| baseline | – | **134.8 KiB** | – |
| R1a | command-bar family → `islands/sui-command-bar/` | **117.3 KiB** (−17.5) | sui-command-bar.js raw 108.5 KiB |
| R1b | devtools-* family → `islands/sui-devtools/` | **104.1 KiB** (−13.2) | sui-devtools.js raw 90.5 KiB |
| R1c | sui-launcher / -panel-dock / -nav-top → individual islands | **92.8 KiB** (−11.3) | sui-launcher 22.9 / sui-panel-dock 22.6 / sui-nav-top 16.6 KiB |

**Total** sui-components.js gz drop: **134.8 → 92.8 KiB (−42.0 KiB,
≈31 % of baseline).** Per-component avg: 1586 B → 1250 B/component
(76 components remain in the every-page registry; 11 moved to islands).
Avg target ≤ 800 B/component is closer but not yet hit — R3 (shared
adoptable stylesheets) is the natural next step to land on the long
tail that stays in `sui-components.js`.

Family-bundle support added to the islands plumbing:

- `sui_compiler::islands::SUI_ISLANDS` is now a `&[(tag, bundle)]`
  table — multiple related tags can pack into one asset bundle. The
  daemon's `serve_sui_island` URL key changed from tag to bundle
  (`/_sui/islands/<bundle>/island.js`); view payloads return deduped
  bundle names. Single-tag islands (`sui-terminal`, individual chrome
  components) keep `tag == bundle`.
- `scripts/build-bundles.mjs` discovers every `.sui` file inside
  each `packages/ui-components/src/islands/<bundle>/` and synthesises
  a multi-import entry, so a family directory yields one packed
  bundle.
- `apps/shell2/index.html` script-loads the chrome islands (nav-top,
  launcher, panel-dock, command-bar, devtools) the shell embeds
  directly. View-host pages skip them entirely.

R2 (sui-core / sui-extended split) becomes a narrower decision after
R1: the every-page bundle is now ~31 % lighter and below the rough
"~80 KiB gz cold load" trigger noted in `scripts/build-bundles.mjs`
gate comments — a future audit may decide R2 is unnecessary in the
post-R1 regime. R3/R4/R5 remain as filed follow-up specs.

## Verification — R3 applied (job/bundle-perf-r3-css, 2026-05-05)

Hoisted the two CSS rules duplicated in nearly every shadow component
into a runtime-side `suiBaseSheet` adopted alongside the existing
`suiTonesSheet`:

- `:host([hidden]) { display: none !important; }` — present in 69
  source files (the per-component copies were `display: none` without
  `!important`; the hoisted form keeps host-level `[hidden]`
  semantics intact).
- `@media (prefers-reduced-motion: reduce) { * { transition: none
  !important; animation: none !important; } }` — present verbatim in
  7 source files.

Plumbing changes:

- `packages/ui-runtime/src/styles/index.ts` — adds `suiBaseSheet` and
  a single `adoptSuiSheets(root)` helper that adopts every shared
  sheet into a shadow root in one call. Re-exported from the runtime
  index.
- `crates/sui-compiler/src/emit/styles.rs` (new) — recognises the
  exact author-form text of the hoisted rules and removes them from
  the emitted `styles:` string. Conservative: only literal-form
  matches strip; rules with extra declarations or non-matching inner
  bodies pass through. Unit-tested with 10 cases including
  edge-of-strip variants and idempotence.
- `crates/sui-compiler/src/emit/js.rs` — shadow components now import
  `adoptSuiSheets` (not the individual sheets) and emit a single
  `adoptSuiSheets(__root)` call. Replaces the previous ~144-char
  inline `adoptedStyleSheets = […]` snippet — 119 chars saved per
  shadow component on top of the styles-string strip.

| metric | baseline | after R3 | Δ |
|---|---:|---:|---:|
| sui-components.js raw | 603.3 KiB | 588.6 KiB | −14.7 KiB |
| sui-components.js gz | 92.8 KiB | 91.9 KiB | −0.9 KiB |
| sui-runtime.js gz | 15.6 KiB | 15.8 KiB | +0.2 KiB |
| every-page combined gz | 108.4 KiB | 107.7 KiB | −0.7 KiB |
| per-component avg gz | 1250 B | 1238 B | −12 B |

Most of the raw-byte savings come from the per-component adoption
snippet shrinking; gzip already deduplicated the `:host([hidden])`
literal across components, so the gz drop is smaller than the raw
drop. The strip plumbing is now in place, so future rule additions
to `BASE_CSS` cost only the lines themselves — no per-component
surgery — and any further hoisting compounds on the same emit path.

`cargo test -p sui-compiler -p sui-lint` green; `pnpm -F
@ship/ui-components test` (1025 tests) and `pnpm -F @ship/ui-runtime
test` (356 tests) green.

Filed follow-up: R4 (`job/sui-icon-lazy-registry`) is independent of
R3 and ready to land when scheduled.

## Verification — R4 applied (job/bundle-perf-r4-lucide, 2026-05-05)

Routed the lucide registry off the every-page bundle:

- `packages/ship-set/scripts/generate-lucide-registry.mjs` now writes
  `crates/ship-ui-framework/assets/lucide-icons.json` alongside the
  existing typed `.ts` form. The `.ts` still backs the Lit
  `ship-elements` consumers (`ship-icon`, `ship-empty-state`,
  `ship-error-state`) which run synchronously inside the
  `ship-elements.js` bundle — that bundle is unaffected by R4.
- `packages/ui-components/src/sui-icon/registry.ts` (new) owns the
  alias map, the in-flight fetch promise, and `setIconRegistry` /
  `loadIconRegistry` / `resolveIconName` helpers. The SUI
  `<sui-icon>` controller no longer imports
  `lucide-icons.generated.ts`; on first mount with a `name`, it
  kicks `loadIconRegistry()` and re-renders once the JSON lands via
  a shared registry-revision signal.
- `crates/ship-ui-framework/src/view.rs` adds
  `serve_sui_lucide_icons` (`include_str!`'d, 1-hour `Cache-Control:
  immutable`); `apps/shipd/src/router.rs` mounts it at
  `/_sui/lucide-icons.json`.
- `apps/shell2/index.html` adds `<link rel="prefetch"
  href="/_sui/lucide-icons.json" as="fetch">` so the registry is
  warm in HTTP cache by the time the rail's six icons mount —
  first-paint perf does not regress.
- `packages/ui-components/test/setup.ts` primes the in-memory
  registry from the generated `.ts` so existing synchronous test
  assertions continue to pass without rewrites.

| metric | baseline | after R4 | Δ |
|---|---:|---:|---:|
| sui-components.js raw | 588.6 KiB | 577.0 KiB | −11.5 KiB |
| sui-components.js gz | 91.9 KiB | 88.1 KiB | −3.8 KiB |
| per-component avg gz | 1238 B | 1187 B | −51 B |
| /_sui/lucide-icons.json raw | n/a | 13.0 KiB | (cached) |

Slightly better than the audit's prorated estimate (~2 KiB gz) — the
registry compressed less efficiently as part of the larger bundle
than as a standalone fetch.

`cargo test -p sui-compiler -p sui-lint` green;
`cargo test -p ship-ui-framework view::tests::lucide_icons_json_served`
green; `pnpm -F @ship/ui-components test` (1025 tests) green.
