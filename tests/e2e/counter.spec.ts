/**
 * Counter view e2e tests — RETIRED.
 *
 * `counter` was a tier-1 demo view (`views/counter/page.sui`). It no
 * longer exists as a registered view: the daemon's resolver
 * (`crates/ship-sui-host/src/resolver.rs`) serves a view only when a
 * `.sui` source is found under `<project>/views/<name>/page.sui` or
 * `<project>/.ship/views/<name>/index.sui`. There is no `counter`
 * source in either location, so `/_sui/views/counter` 404s with
 * `view_not_found`.
 *
 * What remains is residue, not registration:
 *   - a stale `crates/ship-sui-server/assets/views/counter/view.js`
 *     bundle (no matching `.sui` source — dead artifact), and
 *   - `packages/sui/runtime/fixtures/views/counter/` (a runtime test
 *     fixture, exercised by `bun run test:parity`, not a shell view).
 *
 * The same applies to `demos` and `sui-demo`. Restoring them would mean
 * re-authoring `.sui` sources, not flipping an allowlist — out of scope
 * for the e2e suite. This file is kept as a skipped placeholder so the
 * retirement is documented where the next agent will look; delete it
 * once `counter` is either re-authored (restore the real spec from
 * git history) or the residual bundle is removed.
 */
import { test } from '@playwright/test'

test.skip('counter view is retired — no .sui source, daemon 404s /_sui/views/counter', () => {
  // Intentionally empty: see file header. Re-author views/counter/page.sui
  // and restore the SSR/attach/interaction coverage from git history if
  // the counter view comes back.
})
