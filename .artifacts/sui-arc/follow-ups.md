# SUI Arc — Follow-ups

Issues encountered during the autonomous audit that are out of scope for the
current pass (require decisions, span layers, or are non-blocking).

## Pre-existing test failures discovered during the audit

These existed before the SUI audit work began. Documented here so they
don't get attributed to the audit's changes.

### F-P1. `@ship/ui-components` — 57 test suites fail to load

Error: `Failed to resolve import "../../ship-set/src/web-components/core/lucide-icons.generated"`.
The generated file is gitignored and needs a build step to materialize.
Recommended fix: either commit a stub, regenerate as part of `pnpm install`,
or remove the import if the generated lucide registry is no longer required.

### F-P2. `@ship/sui-lsp` — `server.test.ts > server smoke > responds to initialize over stdio` times out

The LSP server stdio init handshake doesn't complete within the test
timeout. Other 53 tests pass. Likely an LSP-init race, not signal-related.

### F-P3. `@ship/sui-eval` — `cli.test.ts` and `score.test.ts` fail

19 tests pass, several fail. score.test.ts: "compiled=true for a valid
static template" returns false. Pre-existing — possibly a generated-asset
dependency, possibly a real compiler regression. Worth a dedicated look
in the sui-compiler / sui-eval audit pass.

### F-P4. SUI HTTP routes are NOT mounted on the daemon router (merge gap)

`apps/shipd/src/views/sui_*.rs` modules exist (`sui_hmr`, `sui_sse`,
`sui_mutation`, `sui_data`, `sui_patches`, `sui_patch_channel`, `sui_agent_push`)
and so do the handlers in `crates/ship-sui-server/src/{view,view_host,view_payload,assets}.rs`
— but **none of them are wired into the axum router**. Verified via:

```
grep -cE "sui_|_sui" apps/shipd/src/router.rs   # → 0
```

Symptom: a daemon launched with `--workspace <repo>` returns 200 on
`/shell/` (Lit/.j2 path) but 404 on `/view-host`, `/_sui/views/<name>`,
`/_sui/runtime.js`, `/_sui/dom.js`, `/_sui/components.js`. Browser-based
SUI verification is blocked until these routes are mounted.

This is sibling work to the bundler wiring done in 30055164 — the merge
brought over the handler modules and the server crate but not the route
table entries. The fix is mechanical: add `Route::Sui*` enum entries to
`apps/shipd/src/routes.rs` and `.ship_route(Route::Sui*, handler)` calls
to `apps/shipd/src/router.rs`. Mirror the wiring on `feat/ship-set` before
the substrate split (the pre-merge router has all the entries).

Out of scope for SUI-rendering reliability (host integration), in scope
for finishing the merge. Document for the next merge-completion pass.

### F-P5. SUI bundler depends on `~/.ship/runtime/{bun,bundler.ts}`, daemon repeatedly fails to start worker

After 30055164 wired the bundler into ShipRuntime, every daemon start
attempts to spawn the worker. Without `~/.ship/runtime/` populated (only
done by `just dev-runtime-install` which is now wired in adecb5f5), the
worker crashes on first try with exponential backoff up to ~10 minutes
between retries. Cosmetic in dev (errors in the log) but spammy. Two
fixes worth considering: skip spawn when the Bun binary is missing, or
log a single one-time "Bun toolchain not installed" message and disable
retry until a SIGHUP. Defer until users complain.

### F-P6. Platform DB migration drift after substrate merge

Existing user platform.db has migrations 3/4/5/6/202604250539 recorded
but missing from the embedded migration set. `ship daemon repair-migrations`
flags them as "orphaned (recorded but no embedded migration — investigate)"
and patches checksums but doesn't drop the orphan records. Migration 3
remains missing, daemon refuses to start.

This blocked browser-based SUI verification during the audit (workaround:
run shipd with fresh `SHIP_HOME=/tmp/...`). Pre-existing infrastructure
issue from the substrate split, not SUI-rendering work. Owner: substrate
team.

### F-P8. Webkit E2E requires system libraries (Linux only)

`packages/sui-runtime/fixtures/e2e/run.mjs` supports `SUI_E2E_BROWSER=webkit`
but on Linux/WSL the playwright webkit binary needs system libraries
(`libharfbuzz-icu`, `libwebpmux`, `libmanette`, `libenchant`, `libhyphen`,
`libsecret-1`, `libwoff2dec`, `libx264`) that aren't installed by default.
Install via `sudo npx playwright install-deps webkit` once. Then the
suite runs against all three engines.

Chromium + firefox both pass 8/8 + counter round-trip locally. CI can
add webkit once the container image has the deps baked in.

### F-C3. sui-compiler byte-slice audit — annotate or rewrite 20 sites

`scripts/lint/check-sui-compiler-str-slicing.sh` flags 20 occurrences
of `&<ident>[<var>..<var>]` patterns in walker code. Most are likely
safe (byte indices land on char boundaries by construction of the
surrounding walker), but the panic class burned us six times in one
session, so each deserves a deliberate confirmation:

- crates/sui-compiler/src/parse/html.rs:177,198
- crates/sui-compiler/src/parse/template/tokenize.rs:182
- crates/sui-compiler/src/parse/template/blocks/for_header.rs:122
- crates/sui-compiler/src/check/scope_bound_expr.rs:185
- crates/sui-compiler/src/check/controller_parse.rs:105
- crates/sui-compiler/src/check/expr_subset.rs:60,146
- crates/sui-compiler/src/emit/expr_rewrite.rs:94,138,153,278
- crates/sui-compiler/src/render/eval/mod.rs:164
- crates/sui-compiler/src/render/eval/lex.rs:138,146,186,194,201,250,286

For each: either annotate with `// allow(str-slice)` and a one-line
reason (boundary guaranteed by upstream walker, indices clamped to
byte boundaries via min(len), etc.), or rewrite using `char_indices()`
/ `is_char_boundary`. After all are addressed, flip the lint script's
trailing `exit 0` to `exit 1` so CI fails on NEW unannotated matches.

Estimated 1-2 hours to walk + annotate.

### F-P7. `<sui-rail>` toggle button has no dynamic behavior

`packages/ui-components/test/sui-rail.test.ts` has two failing tests:
- "toggle button click flips collapsed and dispatches sui-rail:toggle"
- "toggle button aria-label changes with collapsed state"

The tests assume the rail manages its own collapsed state and emits a
`sui-rail:toggle` CustomEvent on click. The component's `.sui` source
has no `on:click` handler on the toggle button and no dynamic
aria-label binding — the static `aria-label="Collapse rail"` doesn't
flip with the `collapsed` prop.

Two reads:
- **Tests are wrong**: the file header says "the shell owns state; rail
  dispatches and the shell reflects it back." Tests violate this.
- **Implementation is incomplete**: the dispatch part (on:click →
  CustomEvent) is missing regardless of who owns state.

I'd land both: add an `on:click` that calls
`host.dispatchEvent(new CustomEvent('sui-rail:toggle', { detail: { collapsed: !collapsed() }, bubbles: true, composed: true }))`,
and bind aria-label to `{collapsed() ? 'Show rail' : 'Hide rail'}`.
Update the click-flips-collapsed test to first reflect the toggle event
back onto the host (which is what the shell would do).

Defer until someone owns the rail component's design decision.

## sui-compiler

### F-C2. `expr_rewrite` mangles multi-byte chars inside string literals

`crates/sui-compiler/src/emit/expr_rewrite.rs` walks the source as
`bytes` and pushes each byte to the output string as `c as char`
(lines 73, 75). For multi-byte UTF-8 chars, this casts each byte
independently — the original char is destroyed and replaced with N
Latin-1 chars. So `"a°"` in source becomes `"a\u{c2}\u{b0}"` in the
rewritten output.

The panic class around it is fixed (see d5797cff and 0d29f587), but
the silent-corruption case persists. Visible to users only when
template-expr strings contain non-ASCII content — rare in practice
because most template strings are ASCII paths/text — but technically
wrong.

Right fix: walk via `chars()` / `char_indices()` everywhere instead
of `as char` byte casts. Estimated half-day; affects ~100 lines of
expr_rewrite. Not blocking but worth doing.

## sui-signals

### F-S1. Concurrent dispatch of the same effect across threads can lose untracks

**Symptom.** If two different signals each subscribed to effect `E` are written
from two different threads simultaneously, both threads fire `E.run` in
parallel. `EffectCore.untracks` is a `Mutex<Vec<UntrackThunk>>` released
between operations rather than held for the whole run, so thread A's stored
new untracks can be overwritten by thread B's stored new untracks, losing
A's record of subscriptions made during its run.

**Impact.** Stale subscriptions can persist when conditional deps differ
between concurrent runs. Bounded by the cases where two writes race on the
same effect AND the effect's f reads different signals each run. For most
server-side reactive use cases (one writer thread at a time per logical
input), this won't manifest.

**Fix options:**
1. Per-effect `Mutex<()>` run lock serializing runs. Risk: deadlock on
   self-recursion (E7). Solution: use a thread-local "I'm running this
   effect already" flag to skip re-entry, or use parking_lot's `ReentrantMutex`.
2. Dispatch via mpsc — writers enqueue rerun requests, a dedicated runner
   drains. Bigger architectural change.

Defer until a real consumer hits this. Document as a known limitation.


## Agent-dispatch process

### F-D1. SUI work must dispatch via `.ship/skills/spawn-agent`, not general-purpose subagents

**Observed this session.** sui-alert-dialog, sui-select, the sui-tab
consolidation, AND the form-value compiler change were all dispatched via
Claude Code's `Agent` tool with `subagent_type: general-purpose`. That
loads ZERO Ship skills — the agents worked from inline briefs only.

This is wrong. Ship has project agents (`.ship/agents/*.jsonc`) configured
with the right skills, MCP servers, permissions, and rules:

- **ship-designer** — SUI authoring: views + components. Bundles ship-views,
  ship-components, ship-design-system, no-slop skills. Use for all `.sui` +
  `.controller.ts` work under `packages/ui-components/src/<tag>/`.
- **rust-lane** — Rust surface: compiler, runtime, MCP. Use for crate
  changes (e.g. the form-value directive's compiler portion).
- **test-writer** — new-primitive tests if dispatched standalone.
- **web-lane** — app-facing code (apps/shell2, apps/site, apps/docs). Does
  NOT author SUI components.

**Right pattern.**

```bash
# 1. Write a job spec
.artifacts/job-spec-<slug>.md   # template lives in .ship/skills/spawn-agent/SKILL.md

# 2. Dispatch
bash .ship/skills/spawn-agent/scripts/dispatch.sh \
  --slug <slug> \
  --agent <agent-id> \
  --spec .artifacts/job-spec-<slug>.md
```

The agent runs in its own git worktree with the right skills loaded,
picks up the job spec on start, and signals completion via `step_done`
per CLAUDE.md's Job Protocol.

**Do not use Claude Code's `Agent` tool with `subagent_type: general-purpose`
for any Ship work that has a matching specialist agent.** General-purpose is
for one-off research and read-only lookups, not for authoring or
multi-file changes that need skill context.

Not retro-applied to in-flight form-value dispatch (would have disrupted
it). All future dispatches — including the 33-port Lit→SUI batch tracked
in `.artifacts/sui-arc/lit-port-plan.md` — MUST use spawn-agent.

## sui-compiler

### F-C5. `<prop>` does not emit `static formAssociated = true`

**Symptom.** Form-associated SUI components (sui-input, sui-textarea,
sui-checkbox, sui-switch, sui-select) all want browser form participation —
reset on form reset, restore on state restoration, value pickup on submit.
The compiler does not currently emit `static formAssociated = true` on the
generated host class, so even though controllers call `attachInternals()` +
`setFormValue()` defensively, the browser does not invoke
`formResetCallback` / `formStateRestoreCallback` on real form events.

**Workaround in controllers.** Install `formResetCallback` and
`formStateRestoreCallback` directly on the host instance in mount(), and
call `attachInternals()` lazily — defensive but does not enable native
form-reset on submit/reset.

**Right fix.** Add a `<prop>`-level or component-level directive
(e.g. `<sui-component form-associated />` or auto-detect from `attachInternals`
in the controller AST) that emits `static formAssociated = true` on the
generated class.

**Affects.** sui-input, sui-textarea, sui-checkbox, sui-switch, sui-select.
Discovered during the sui-select port.
