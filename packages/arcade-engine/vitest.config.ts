import { defineConfig } from 'vitest/config'

// The arcade engine is headless logic (loop / world / collide / input /
// particles / audio / palette / stage). Most modules are pure math or small
// state machines; the few that reach for browser globals
// (requestAnimationFrame, AudioContext, document, MutationObserver) get those
// stubbed per-test. The vendored PixiJS bundle is mocked (see test/_pixi-mock.ts)
// so world/particles tests never load the 800 KB renderer.
//
// `environment: 'node'` keeps each fork lean — no jsdom/happy-dom dependency,
// matching the "pure/headless" nature of the package.
export default defineConfig({
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'node',
    pool: 'forks',
    maxForks: 2,
    minForks: 1,
    testTimeout: 10_000,
    hookTimeout: 10_000,
  },
})
