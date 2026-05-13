# lit → SUI port — batch 2

Batch 1 (#137) cleared the highest-value missing primitives off the audit:
`sui-log-stream`, `sui-pipeline-dag`, `sui-pipeline-run`, `sui-pipeline-step`,
`sui-pipeline-step-def`, `sui-provider-blob`. Batch 2 closes out the four
deferred `domain/` primitives and the substituted `core/ship-theme-toggle`.

## What this dispatch shipped

| port                       | lit LOC | sui shape                                  | notes |
|----------------------------|---------|--------------------------------------------|-------|
| `sui-pipeline-definition`  | 96      | `.sui` + `.controller.ts`                  | Composes `sui-card interactive accent-bar`. Pluralization derived. |
| `sui-theme-toggle`         | 92      | `.sui` + `.controller.ts`                  | Binary light/dark toggle distinct from `sui-theme-picker`. |
| `sui-history-row`          | 210     | `.sui` + `.controller.ts`                  | Graph lane SVG + avatar + subject split + +/- stats. Keyboard + click emit `sui-select`. |
| `sui-auth-dialog`          | 324     | `.sui` + `.controller.ts`                  | Composes `sui-dialog` + `sui-tabs` + `sui-button`. Slot-change drives "or" divider visibility. |
| `sui-git-diff` (×4)        | 342     | 4× `.sui` (+ 3 `.controller.ts`)           | Container + file + hunk + line. Split across files so each stays well under the 400-line cap. |

### Behavior-parity notes

- **`sui-pipeline-definition`** — declares the same three props (`name`, `steps`,
  `description`). Pluralization moves to a controller-derived `stepsLabel`
  (`X step` vs `X steps`); the lit version computed it inline in `render()`.
  `role="article"` + a recomputing `aria-label` mirror the lit `updated()` hook.
- **`sui-theme-toggle`** — `theme = 'auto'` is the SUI default-attribute value;
  controller `mount` resolves it to `light`/`dark` from localStorage or
  `prefers-color-scheme` and writes the resolved value back through the
  signal (so the `theme` attribute reflects). Click handler flips the value,
  persists under the same `ship-theme` localStorage key, toggles `.dark` on
  `documentElement`, and emits `sui-theme-change` (renamed from
  `ship-theme-change` per convention).
- **`sui-history-row`** — `_splitSubject()` (the conventional-commit prefix
  parser) moves to a controller derived (`scopePrefix` + `subjectRest`) reading
  the same regex. `role="button"` + `tabindex="0"` set on mount; the
  Enter/Space keydown handler emits the same `sui-select { sha }` event.
  Lane click + msg click both fire it. `human` attribute drives the avatar's
  conic-gradient via `:host([human])` selectors, unchanged from lit.
- **`sui-auth-dialog`** — composes `sui-dialog`. Inner `sui-tabs`/`sui-tab`
  drives mode flips via `sui-tab-change` (the SUI-renamed event). The lit
  `hasProviderChildren()` poll moves to a controller `slotchange` listener
  on a `bind:this`-ref'd `<slot name="providers">`; the resulting
  `hasProviders` signal gates the `<div class="or">` divider. `show()` /
  `showModal()` / `close()` imperatives are reattached to the host in `mount`
  to match the lit method-level API. Inner-dialog `sui-close` propagates to
  the host as `sui-close` and clears the `open` signal.
- **`sui-git-diff`** — split into four files because the lit module bundles
  4 elements behind one `@customElement` block:
  - `sui-git-diff.sui` — toolbar + empty-state + `<slot>`. Controller owns
    the local `allCollapsed` UI signal, derives toolbar labels + chevron
    icon, and on toggle walks `host.querySelectorAll('sui-git-diff-file')`
    in light DOM to flip each child's `collapsed` attribute.
  - `sui-git-diff-file.sui` — collapsible header. Controller derives chevron
    icon name + `aria-expanded`; click toggles `collapsed`.
  - `sui-git-diff-hunk.sui` — pure template, no controller. Renders
    `{header()}` inside the styled host.
  - `sui-git-diff-line.sui` — old/new gutters, sig, content slot, comment
    button. The `new` prop name is a JS reserved word so the controller
    aliases it via `propBag['new']` (bracket access) and the template uses
    a derived `newCol`. The comment button emits `sui-comment { old, new }`
    (renamed from `ship-comment`).

### Substitutions / renames

| lit                | sui (batch 2)              | reason |
|--------------------|----------------------------|--------|
| `ship-select`      | `sui-select`               | event-name convention |
| `ship-theme-change`| `sui-theme-change`         | event-name convention |
| `ship-mode-change` | `sui-mode-change`          | event-name convention |
| `ship-auth-submit` | `sui-auth-submit`          | event-name convention |
| `ship-close`       | `sui-close`                | event-name convention |
| `ship-comment`     | `sui-comment`              | event-name convention |
| `ship-tab-change`  | `sui-tab-change`           | consumed from existing sui-tabs port |

### Open follow-ups

- Visual baselines (`tests/e2e/visual/baselines/`) need to be captured with
  `just visual --update-snapshots` against a live daemon. Batch 1 added
  registry entries to `views.json` but never committed PNGs either, so this
  is a one-shot capture covering both batches when next a maintainer runs
  the suite.
- `ship-auth-dialog` originally used `<ship-tabs variant="segmented">`. The
  sui-tabs port has variants `underline|pill|enclosed`; this port picks
  `pill` as the visual closest match. If shell2 ever brings the segmented
  variant in, swap it back here.
- The audit now records every `domain/` lit primitive as ported. Remaining
  un-ported `apps/shell/*` chrome (`ship-shell`, `ship-launcher`, `ship-nav-top`,
  `ship-panel-dock`, `ship-command-bar`, `ship-toast-host`, `ship-rail`,
  `ship-devtools-*`) is shell-internal and lives behind island bundles
  (`packages/ui-components/src/islands/`); not in scope for the "missing
  primitives" track.
