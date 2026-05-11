// Shared vitest defaults — workers + timeouts.
//
// Why: vitest defaults to one fork per CPU core. Each fork in a
// jsdom-using config holds ~500 MB. On a 10-core machine the total
// peak crosses 4 GB and exhausts memory-constrained dev environments
// (notably WSL2). Cap parallelism here once.
//
// Override the cap with VITEST_MAX_FORKS=N in the env. CI can raise it
// when more memory is available.

import type { ViteUserConfig as VitestUserConfig } from 'vitest/config'

const MAX_FORKS = Number(process.env.VITEST_MAX_FORKS ?? 2)

export const SHIP_VITEST_DEFAULTS: NonNullable<VitestUserConfig['test']> = {
  pool: 'forks',
  // vitest 4 lifted poolOptions.forks.* to top-level.
  maxForks: MAX_FORKS,
  minForks: 1,
  // Hang detection. A test stuck in a never-resolving Promise would
  // otherwise hold a fork open indefinitely.
  testTimeout: 10_000,
  hookTimeout: 10_000,
}
