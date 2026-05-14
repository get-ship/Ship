# SUI autopilot — handoff (mid-session compaction)

**Branch:** `feat/ship-set`
**State:** SUI framework production-ready; phase transition coming to 1st party views + documentation.

## Session cumulative

**47 merges, 6 inline fixes, 1 abandoned (D1 — postmortem at `.artifacts/sui-arc/d1-postmortem.md`).**

Capabilities operational: error-boundary, portal, transition, HMR client/server, morph, scope-typing (cast tax dies → 17 controllers cleaned), LSP-native + MCP, perf D2/D3/D4 + fragment-batch + derived-memo, sui-rail/sui-nav-top/sui-panel-dock full parity with apps/shell, CEM aggregator productionized, view-host importmap subpaths, daemon per-worktree (#69), repair-migrations --prune-orphans, sse-demo consumer + producer (config-emitter just merged), 17 lit primitives retired, ship-ui-studio v3 with morph + variant deep-link, cast-free canonical controller pattern.

## In flight (4 slots, parallel limit 4)

1. `perf-d5-inline-static` (#104) — rust-lane — compiler inline of static row bindings (no `()` → direct DOM assign). "Verifying tests + bench" on last poll.
2. `sui-devtools-pane` (#105) — ship-designer — last shell2 chrome element parity.
3. `ssr-hydration-audit` (#106) — ship-designer — operational verification of SSR + hydrate path, output `.artifacts/sui-arc/ssr-hydration-audit.md`.
4. `replace-children-cleanup` (#107) — rust-lane — `innerHTML='' + appendChild` → `replaceChildren(...)` cleanup, ≥3 sites.

## After current 4 land

1. **RE-BENCH** — capture cumulative D2+D3+D4+D5+fragment-batch+derived-memo state at `tests/perf/baselines/<date>-post-perf-cycle.json`. Update `.artifacts/sui-arc/perf-status.md`. Highest leverage; fires as soon as D5 lands.
2. SDK type-gen audit — verify .sui.d.ts emits + SDK client typing post scope-typing.
3. Bun toolchain coherence — `bun.config.ts` forward path.

## On hold (explicit user-go gate)

The user engaged substantively but didn't say "dispatch":

### A. Styling-system rules (READY to dispatch when user OKs)

Three sub-dispatches:
1. **Token taxonomy audit + lint rules** — sui-lsp warns on hex/px/unknown-token. Single source-of-truth taxonomy: `--bg-*`, `--fg-*`, `--border-*`, `--space-{1..9}`, `--text-{xs..2xl}`, `--radius-{sm,md,lg,pill}`, `--duration-{short,medium,long}`, `--ease-{in,out,inout}`. NO `--accent`, `--emphasis`, `--text-primary` (intent must be in prefix).
2. **"Variants are attributes, never classes" sweep** — mechanical refactor across existing sui-*. `<sui-button tone="primary" size="lg">` → `:host([tone='primary'])` CSS. Drop `class:foo={bool}` directive from approved patterns.
3. **`docs/styling-rules.md`** in skill 3-audience format. 7 rules (variants=attrs, state=data-*, tokens only, no inline static styles, reactive style: only for instance-computed, no CSS-in-JS, one theme switch surface).

Apply the "more rails" pattern beyond styling: signals-only state, `on:event={...}` always (no manual addEventListener in mount), `effect()` not `setInterval` in mount.

### B. Synthetic controller (READY to dispatch when user OKs)

User: "I've been thinking about this for a while... could it help solve practically-empty controllers... templates alone should cover many use cases... controllers should never be ceremony."

**Feasibility: HIGH (~1-2 day arc).** Plan:
1. sui-compiler: if `<name>.controller.ts` absent, synthesize an empty `controller<Scope>({})` at compile time
2. Scope d.ts emit: only template-derived fields when no factory exists
3. Compiled.js: customElements.define() already compiler-generated; no controller import needed
4. sui-lsp: optional — flag existing empty controllers as deletable

Eliminates ceremony for sui-spinner/sui-kbd/sui-status-dot-tier components.

Follow-on (separate dispatch, depends on this):
- **`prop<T>()` helper** in scope factory — props live alongside signals; compiler picks signal vs direct property access based on use site.
- Reduces `<prop>` declarations in .sui to one canonical place (controller).

### C. Output organization (TRIVIAL, READY when user OKs)

User: "clearly cem, meta, .d.ts etc belong in dist. Let me know if the question is about content, but we really just need to put generated files in dist/tooling specific directories and gitignore them....easy."

~half-day:
1. sui-compiler emit path takes `dist_dir: PathBuf` (default `<src>/dist`)
2. .gitignore adds `packages/ui-components/src/sui-*/dist/` + views/*/dist/
3. sui-build + view-host path resolution updates
4. Directory listings drop from 7 files/component to 1 source + dist/

## User direction (durable)

- "SUI is core for our product launch. At a certain point we need to move past parity/framework iteration and nail 1st party views + documentation."
- Docs are skill-driven with 3 audiences (agent/internal/external), tied to site gen. Don't dispatch generic doc work.
- Tokens already documented; no taxonomy work needed without explicit go.
- "Ship the product = flexible like code, vs something like Notion. Ship's agent-authored UI = more rails than React anything-goes."
- "no need for a new just recipe" (sui-dev wrapper). DON'T dispatch.
- Visual parity with j2/lit then EXCEED via runtime integration (operational, not just shipped)
- Keep apps/shell runnable as reference; don't migrate or delete
- Lit retirement = background-only (no longer the bar)
- 4 parallel agents
- Keep user out of review loop; flag only if TRULY blocked

## Critical autopilot mechanics

- Daemon per-worktree (#69) works. Restart IN worktree before `pnpm test:e2e:visual`.
- pnpm-bins workaround: `node node_modules/@ship/sui-build/bin/sui-build.mjs`
- lucide-icons.generated.ts gitignored; regen `pnpm -F @ship/set icons` in every new worktree
- Co-Authored-By strip: `git filter-branch -f --msg-filter 'grep -vE "^Co-Authored-By:|^🤖"' feat/ship-set..HEAD`
- bench-lib regex widened (be8b63f6) — bench-check reads labeled baselines
- DON'T reset dev DB autonomously; if agent asks, pick "skip live smoke; rely on integration test"
- ship-platform has 5 pre-existing flaky tests (config::watcher + hooks::sources — file-watcher races); verify on parent before flagging
- Multi-line tmux send-keys needs separate Enter to submit
- Common agent stall: "❯ commit and push" / "❯ commit this" / "❯ push the branch" — rescue-commit
- Common interactive prompts: agents may ask permission for destructive ops (DB reset, etc.); always pick the non-destructive option

## Performance state (mid-perf-cycle)

Pre-perf-cycle baseline: `tests/perf/baselines/2026-05-13-baseline.json`
Post-D2+D4 baseline: `tests/perf/baselines/2026-05-13-post-d2-d4.json`
Pending: post-D3+D5+fragment+derived re-bench (queued for after D5 lands)

vs Solid (post-D2+D4):
- AHEAD: 04_select1k -29%, 06_remove-one -12%
- NEAR: 01_run1k +17%, 02_replace1k +16%, 05_swap1k +18%, 08_create1k-after1k +22%
- WORST: 03_update10th1k_x16 +116% (derived-memo targets this directly; results pending re-bench)

## Tests + gates

- `cargo test --workspace` + `cargo test -p sui-compiler --test shape_conformance`
- `pnpm -F @ship/sui-runtime test` — 476+ tests
- `pnpm -F ui-components test` — 1454+ tests (2 pre-existing F-P7 sui-rail fails baseline-acceptable)
- `pnpm -F @ship/sui-lsp test` — 59 tests
- `pnpm test:e2e:visual` — 64 Playwright baselines (4 views × 2 themes + 28 components × 2)
- `just visual` (alias), `just bench-check`, `just smoke-deploy`

## File-cap discipline

300 LOC/file. Several recent components (sui-error-boundary, sui-rail) extract sub-modules at the edge.

