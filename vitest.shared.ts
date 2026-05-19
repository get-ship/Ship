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
import { existsSync, readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

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

// WASM fallback for vitest's `compiled-js-shim` plugin — invoked when
// a `.sui` exists but no mirror artifact was found. The `pretest`
// hook precompiles every `.sui` in normal flow so this only fires
// during watch-mode HMR cycles that pick up a freshly-edited source.
//
// Replaces the legacy `@ship/sui-dom/legacy` hydrate fallback. The
// wasm module is loaded synchronously from the workspace tree so the
// vite config can call this in `load()` without going async.

let wasmCompile: ((source: string, optsJson: string) => unknown) | null = null
function ensureWasm(): (source: string, optsJson: string) => unknown {
  if (wasmCompile) return wasmCompile
  const here = dirname(fileURLToPath(import.meta.url))
  const pkgDir = resolve(here, 'packages/sui/compiler-wasm/pkg')
  const wasmPath = resolve(pkgDir, 'sui_compiler_bg.wasm')
  const jsPath = resolve(pkgDir, 'sui_compiler.js')
  if (!existsSync(wasmPath) || !existsSync(jsPath)) {
    throw new Error(
      `[vitest-shared] sui-compiler-wasm not built. Run \`just sui-wasm\`.`,
    )
  }
  // Dynamic require keeps the wasm load out of the cold-path config
  // graph; only the configs that actually fall back to it pay the
  // cost. `require` works here because vite evaluates the config file
  // in a CJS-compatible context.
  const mod = require(jsPath) as {
    initSync: (input: { module: Buffer }) => unknown
    compile: (source: string, optsJson: string) => unknown
  }
  mod.initSync({ module: readFileSync(wasmPath) })
  wasmCompile = mod.compile
  return wasmCompile
}

/** Compile a `.sui` file to its emitted JS module. Throws on compile error. */
export function compileSuiToJs(suiPath: string): string {
  const compile = ensureWasm()
  const source = readFileSync(suiPath, 'utf8')
  const result = compile(source, JSON.stringify({ mode: 'auto' })) as
    | { kind: 'ok'; js: string }
    | { kind: 'err'; message: string; line: number | null; col: number | null }
  if ((result as { kind: string }).kind === 'err') {
    const e = result as { message: string; line: number | null; col: number | null }
    throw new Error(
      `[vitest-shared] sui-compiler error in ${suiPath} (${e.line ?? '?'}:${e.col ?? '?'}): ${e.message}`,
    )
  }
  return (result as { js: string }).js
}
