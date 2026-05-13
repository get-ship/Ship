# SUI rendering arc — handoff

**As of:** end of session 2026-05-12
**Branch:** `feat/ship-set` (currently 50+ commits ahead of origin)
**State:** SUI rendering pass effectively complete; remaining work is migrations + polish.

---

## Where we are

### Top-line status

| Concern | State |
|---|---|
| j2 → SUI view parity | ✅ 16/16 top-level views ported (batches A–D), 5/5 partials-gap fixed |
| Shell parity test | ✅ Static + Playwright modes; 16/16 ✓ in last report |
| Daemon SSR | ✅ Compile-time emit (`crates/sui-compiler/src/emit/ssr.rs`) + `/view-host/<name>` splice |
| Dev/prod switch | ✅ `shipd --mode development\|production` |
| Performance parity vs Lit/Solid/Svelte/Vue | ✅ Same class on every benchmarked op; LIS reconciler dropped swap-1k from 52.5ms → 12.6ms |
| Per-item reactivity in `{#for}` | ✅ Compiler emits `update` hook + version-signal injection |
| sui-build (canonical bundling) | ✅ Foundation + ship-set migrated as first consumer |
| `emit/js.rs` over 300-cap | ✅ Split into 4 sub-modules |
| Templates Phase 1 retirement | ✅ -547 LOC across 14 legacy sub-views + templates.rs |
| shell2 dev workflow | ✅ Workspace fix landed (was missing from `pnpm-workspace.yaml`) |

### Performance numbers (no-throttling baseline)

| Op | SUI | Solid | Svelte | Vue | Vanilla |
|---|---|---|---|---|---|
| 01_run1k | 59.8 | 49.5 | 50.0 | 63.8 | 55.9 |
| 03_update10th1k_x16 | 12.6 | 10.7 | 11.7 | 13.8 | 10.1 |
| 04_select1k | **4.8** ✓ | 6.0 | 7.1 | 4.6 | 5.3 |
| 05_swap1k (post-LIS) | **12.6** | 12.8 | 9.7 | 7.7 | 9.3 |
| 22_run-memory (MB) | 4.1 | 2.6 | 2.8 | 3.7 | 1.8 |
| 42_size-compressed (KB) | **6.7** | 4.5 | 12.2 | 22.8 | 2.5 |

Strong: select-1k tied for best, bundle size top-3, swap parity post-LIS.
Weak: run-1k slightly behind cluster leaders (Solid/Svelte); run-memory not bottom tier but heavier than leanest peers.

---

## Active queue (priority order)

| # | Task | Agent | Notes |
|---|---|---|---|
| 34 | Migrate `ui-components` to sui-build | ship-designer | Unblocked — second sui-build consumer; mirror ship-set's pattern. Will retire `scripts/precompile-sui.mjs`. |
| — | Cleanup: sui-heatmap key-busting workaround removal | ship-designer | bindFor fix makes this safe; ~10 LOC change |
| — | Cleanup: sui-step-segments {#for} rewrite | ship-designer | Same; rewrite controller's imperative DOM construction as template `{#for}` over derived signal |
| 35 | Bun FFI for sui-compiler | rust-lane | Phase-2 perf win; load cdylib instead of spawn-per-file. ~50 LOC FFI + Cargo.toml `cdylib` target. |
| — | Templates Phase 2 retirement (chrome partials) | rust-lane | -113 LOC; needs user sign-off per `.artifacts/sui-arc/shipd-templates-audit.md` (confirm no user-authored j2 view references the 6 chrome partials) |
| 39 | ship-ui-studio v2 (Storybook-equivalent) | ship-designer | Substantial UX arc; CEM-driven knobs, state variants, theme variants, isolated rendering. |
| — | apps/shipd/templates/ Phase 2 (canvas/chrome) | varies | -113 LOC after sign-off; canvas templates stay |

### Deferred / future arcs

- **Mount() modularity refactor** — contract-based payload provider (decouple daemon-specific shape). Currently Ship-coupled by design; future-only if external embedding becomes a need.
- **`crates/core/lint` extract-shared-types refactor** — needed before any rename to `j2-lint`. Sui-lint imports `Finding`/`Severity` from it; rename would hide that role.
- **Auth-aware dynamic SSR** — beyond v1 static SSR. Per-request data-resolved rendering with actor context.
- **Component preview composition examples** — for compound components (sui-rail + items). Fits inside ship-ui-studio v2.
- **Performance baselines committed to `.artifacts/sui-arc/benches/`** — bench harness exists, baseline numbers not yet captured.

---

## Toolchain & conventions

### Dispatch pattern (USE THIS)

```bash
bash .ship/skills/spawn-agent/scripts/dispatch.sh \
  --slug <name> \
  --agent <ship-designer|rust-lane> \
  --spec .artifacts/job-spec-<name>.md \
  --model sonnet
```

Opens a wt tab with the right agent profile + skills loaded.
**Do NOT** use Claude Code's `Agent` tool with `subagent_type: general-purpose` for write work — it loads no Ship skills and has no permission constraints (this caused real damage early in the arc; see `.artifacts/sui-arc/follow-ups.md` F-D1).

### Machine concurrency limit

**Max 3 dispatched agents in parallel.** User explicitly limits this to save the machine.

### sui-build (canonical bundler)

`packages/sui-build/` — orchestrator package, in-tree.

**Per-package config**:
```ts
// <package>/sui.config.ts
export default {
  entries: { 'name': 'src/entry.ts' },
  lazyBundles: { 'lazy-name': 'src/...' },
  outdir: '../../apps/shipd/assets',
  loaders: { '.svg': 'text' },  // optional, per-extension loader override
  mode: 'production',  // optional; default 'production'
}
```

**Adapters**: Bun (default) + esbuild (secondary). Both ship; benchmark + reconcile later. Set adapter via `--bundler` flag or `SUI_BUILD_BUNDLER` env.

**Phase 2** (queued): Bun FFI replaces process-spawn for sui-compiler invocations.

### shell2 dev workflow

```bash
pnpm -F shell2 dev
# Opens vite on http://127.0.0.1:3200 (or :32xx based on daemon-port offset)
# Vite proxies /api + /_sui to the running shipd daemon
```

Daemon at `--workspace .../sui-runtime/fixtures` runs on port 9385 (path-hashed). Restart with `just dev-restart` to pick up daemon-side merges.

### Benchmark setup

Sibling repo at `~/dev/js-framework-benchmark/`. Our framework at `frameworks/keyed/sui/`:

```bash
# Build sui bundle
cd ~/dev/js-framework-benchmark/frameworks/keyed/sui && npm run build-prod

# Start http-server
cd ~/dev/js-framework-benchmark && npm start  # port 8080

# Run bench (need built peer frameworks too)
cd ~/dev/js-framework-benchmark/webdriver-ts && \
  node dist/benchmarkRunner.js --framework keyed/sui keyed/solid keyed/svelte keyed/vue keyed/vanillajs keyed/react-hooks \
    --headless --nothrottling \
    --chromeBinary /home/madvib/.cache/ms-playwright/chromium-1217/chrome-linux64/chrome
```

Results land in `webdriver-ts/results/*.json`. View with:
```bash
cd webdriver-ts && node dist/createResultJS.js
cd ../webdriver-ts-results && npm run dev  # vite serves React table
```

---

## File locations

| What | Where |
|---|---|
| SUI authoring skill (agent docs) | `.ship/skills/ship-views/SKILL.md` + `references/` |
| SUI component skill | `.ship/skills/ship-components/SKILL.md` |
| Tooling observations (running list) | `.artifacts/sui-arc/tooling-observations.md` |
| Lit→SUI port plan | `.artifacts/sui-arc/lit-port-plan.md` |
| Naming-mismatch decisions | `.artifacts/sui-arc/naming-decisions.md` |
| SSR design | `.artifacts/sui-arc/ssr-design.md` |
| Daemon templates audit | `.artifacts/sui-arc/shipd-templates-audit.md` |
| Follow-ups (active) | `.artifacts/sui-arc/follow-ups.md` |
| Parity test report | `packages/sui-runtime/fixtures/e2e/parity-baselines/parity-report.md` (regenerated per run; gitignored) |
| Agent dispatch specs (current) | `.artifacts/job-spec-*.md` |
| Bench framework entry | `~/dev/js-framework-benchmark/frameworks/keyed/sui/` |

---

## Open architectural questions (decided this session)

| Question | Decision |
|---|---|
| Static SSR opt-out flag per view | **No** — YAGNI |
| sui-build asset layout | **Flat** at configurable outdir (Ship default `apps/shipd/assets/`) |
| `lazyBundles` term | **Keep** (chose over `splits`, `islands`) |
| sui-build default mode | **Production** (development is opt-in) |
| sui-build default adapter | **Bun**; esbuild is secondary; both ship |
| Bundler-as-interface | **Yes** — SuiBundler interface; `bundler` flag/env switches adapter |
| nextjs-style file-level error/loading conventions | **No** — one way: `{#while-loading}` + `<sui-error-boundary>` (future), no `error.sui` |
| Mount()'s daemon coupling | **Intentional** — SUI is Ship-first; modularity via contract is a future concern |
| .sui.meta.json | **Keep for now** — re-evaluate in tooling conversation |
| Vite for shell2 dev | **Yes, separate concern** — shell2's vite is internal; doesn't affect framework outputs |

---

## Known issues (not blocking)

1. **`02_replace1k` harness flake** — failed for SUI/Svelte/Vue in last run; likely harness-side, not framework. Re-run with throttling to confirm.
2. **Bench's framework label "sui-vERROR..."** — fixed by adding explicit `frameworkVersion: "0.1.0"` to `frameworks/keyed/sui/package.json`. If a future re-run shows ERROR, that field's gone.
3. **`useShadowRoot: false` is achieved via post-process** in `frameworks/keyed/sui/scripts/precompile.mjs` (sed flip of `shadow: true` → `shadow: false` in the compiled bundle). Proper fix: compiler directive `<sui-component shadow="false" />`. Logged in tooling-observations.

---

## Next obvious dispatches

Round of 3 to launch when ready:

1. **ui-components migration to sui-build** (task #34, ship-designer)
2. **Cleanup batch**: sui-heatmap workaround removal + sui-step-segments {#for} rewrite (ship-designer; same file scope)
3. **Bun FFI for sui-compiler** (task #35, rust-lane)

After those land: ship-ui-studio v2 is the next big arc.

---

## Recent commit history (this session, most recent first)

```
e38d85f3 merge: sui-build v2 + ship-set migration
cddc0647 chore(gitignore): ignore .sui-build-cache.json
62d3b6a3 feat(sui-build+ship-set): production-default + loaders config
6895447c merge: parity-gaps-fixup
3ff4d053 fix(views): close 12 aria/slot-title gaps
ffdae692 merge: emit-js-split
a432494f refactor(sui-compiler): split emit/js.rs (455 LOC → 4 sub-modules)
34bbafda fix(pnpm-workspace): add apps/shell2 to packages list
5db7da10 merge: shell-parity-test
d5a76f69 feat(parity): j2 ↔ SUI view parity test harness
5fc26bc8 merge: templates-phase1-retire
37a263c8 chore(shipd-templates): retire 14 Bucket E legacy sub-views (-547 LOC)
e70d969a merge: dev-prod-switch
4be59701 feat(shipd): dev/prod mode switch
0502b449 merge: ssr-static
24e08829 feat(sui-compiler+ship-sui-server): static SSR at compile time
ef39ea53 merge: view-port-batch-d — remaining 7 views, closes top-level j2 parity gap
babfe6f5 merge: view-port-batch-c — pipelines tree (5 views)
193541e6 merge: view-port-batch-b — agents tree (4 parameterized views)
8098d34f merge: view-port-batch-a — 5 SUI views
d20d0982 merge: runtime-perf-audit — LIS reconciler in bindFor
be3e8359 merge: sui-build-foundation
42f7575f merge: bindfor-update-emit
```

(There are more before that — bindFor update hook fix, sui-tabs consolidation, form-value directive, sui-alert-dialog/sui-select ports, viz batch, editors batch, etc.)
