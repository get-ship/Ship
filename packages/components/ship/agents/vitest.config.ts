// Vitest config for the @ship/agents pack.
//
// The @ship/test-setup factory's compiled-js-test-shim plugin is
// the right home for the `<tag>.compiled.js` resolution, but the
// factory hardcodes `require.resolve('@ship/deck/router')` + the
// `@ship/sui-runtime/route` + `@ship/sui-runtime/virtual` aliases
// from the test-setup package's own node_modules — three sub-paths
// that don't exist in this worktree (the deck/route package
// hasn't been published to a real registry; the sui-runtime
// `./route` + `./virtual` exports point at files that were deleted
// in the sui→deck closure — see slice2-finish-notes.md).
//
// The @ship/studio precedent side-steps the factory: its vitest
// config is a `defineConfig` with the test-setup aliased in by
// relative path (for the JSDOM polyfills), and it skips the shim
// (no test files import `*.compiled.js` because the studio pack
// has no compiled outputs in this branch). The agents pack has
// compiled outputs (every component imports its sibling
// `*.compiled.js` per the bundle-resolution test pattern), so we
// keep the shim here — inlined locally rather than going through
// the broken factory, and keyed off the WAI-built `target/debug/
// sui-compiler` binary (the existing `sui-compiler` test path).

import { defineConfig } from 'vitest/config'
import { fileURLToPath } from 'node:url'
import { createRequire } from 'node:module'
import { existsSync, readFileSync, statSync } from 'node:fs'
import { resolve, dirname, basename, relative } from 'node:path'

const require = createRequire(import.meta.url)

function findRepoRoot(): string {
  let dir = dirname(fileURLToPath(import.meta.url))
  for (let i = 0; i < 12; i++) {
    const pkg = resolve(dir, 'package.json')
    if (existsSync(pkg)) {
      try {
        if (JSON.parse(readFileSync(pkg, 'utf8')).name === 'ship-monorepo') return dir
      } catch {
        /* keep walking */
      }
    }
    const parent = dirname(dir)
    if (parent === dir) break
    dir = parent
  }
  throw new Error('[agents/vitest] could not locate the ship-monorepo root')
}

const repoRoot = findRepoRoot()
const mirrorRoot = resolve(repoRoot, '.ship/sui-out')

// @ts-expect-error — JS module, no .d.ts.
import { spawnBoundedOrThrow } from '../../../test-setup/src/spawn-bounded.mjs'

let wasmCompile: ((source: string, optsJson: string) => unknown) | null = null
function ensureWasm(): (source: string, optsJson: string) => unknown {
  if (wasmCompile) return wasmCompile
  let jsPath: string
  let wasmPath: string
  try {
    jsPath = require.resolve('@ship/sui-compiler-wasm')
    wasmPath = require.resolve('@ship/sui-compiler-wasm/sui_compiler_bg.wasm')
  } catch {
    throw new Error('[agents/vitest] sui-compiler-wasm not built. Run `just sui-wasm`.')
  }
  const mod = require(jsPath) as {
    initSync: (input: { module: Buffer }) => unknown
    compile: (source: string, optsJson: string) => unknown
  }
  mod.initSync({ module: readFileSync(wasmPath) })
  wasmCompile = mod.compile
  return wasmCompile
}

function compileSuiToJs(suiPath: string): string {
  const compile = ensureWasm()
  const source = readFileSync(suiPath, 'utf8')
  const result = compile(source, JSON.stringify({ mode: 'auto' })) as
    | { kind: 'ok'; js: string }
    | { kind: 'err'; message: string; line: number | null; col: number | null }
  if (result.kind === 'err') {
    throw new Error(
      `[agents/vitest] sui-compiler error in ${suiPath} (${result.line ?? '?'}:${result.col ?? '?'}): ${result.message}`,
    )
  }
  return result.js
}

const compilerBin = resolve(repoRoot, 'target/debug/sui-compiler')
function compileSui(suiPath: string, stem: string): void {
  const hasBin = existsSync(compilerBin)
  const cmd = hasBin ? compilerBin : 'cargo'
  const args = hasBin
    ? ['build', suiPath, '--component-name', stem, '--no-dts']
    : ['run', '--quiet', '-p', 'sui-compiler', '--', 'build', suiPath, '--component-name', stem, '--no-dts']
  spawnBoundedOrThrow(cmd, args, `sui-compiler ${suiPath}`, { cwd: repoRoot })
}

function mirrorArtifactFor(sourceDir: string, stem: string): string {
  return resolve(mirrorRoot, relative(repoRoot, sourceDir), `${stem}.compiled.js`)
}

export default defineConfig({
  server: { fs: { allow: [repoRoot] } },
  resolve: {
    alias: {
      '@ship/test-setup': fileURLToPath(new URL('../../../test-setup/src', import.meta.url)),
      '@ship/sui-dom/test-utils': require.resolve('@ship/sui-dom/test-utils'),
      '@ship/sui-runtime/template-hooks': require.resolve('@ship/sui-runtime/template-hooks'),
      '@ship/sui-runtime': require.resolve('@ship/sui-runtime'),
      '@ship/sui-dom': require.resolve('@ship/sui-dom'),
    },
  },
  test: {
    environment: 'jsdom',
    include: ['test/**/*.test.ts', 'src/**/*.test.ts'],
    setupFiles: ['./test/setup.ts'],
    pool: 'forks',
    maxForks: 2,
    minForks: 1,
    testTimeout: 10_000,
    hookTimeout: 10_000,
    passWithNoTests: false,
  },
  plugins: [
    {
      name: 'compiled-js-test-shim',
      resolveId(id: string, importer?: string) {
        if (!id.endsWith('.compiled.js') || !importer) return null
        const sourceArtifact = resolve(resolve(importer, '..'), id)
        const sourceDir = dirname(sourceArtifact)
        const stem = basename(sourceArtifact).replace(/\.compiled\.js$/, '')
        const mirrorFile = mirrorArtifactFor(sourceDir, stem)
        const suiFile = resolve(sourceDir, `${stem}.sui`)
        const mirrorFresh =
          existsSync(mirrorFile) &&
          (!existsSync(suiFile) || statSync(mirrorFile).mtimeMs >= statSync(suiFile).mtimeMs)
        if (mirrorFresh) return mirrorFile
        if (existsSync(suiFile)) {
          compileSui(suiFile, stem)
          if (existsSync(mirrorFile)) return mirrorFile
          return `\0compiled-js-shim:${suiFile}`
        }
        return null
      },
      load(id: string) {
        const prefix = '\0compiled-js-shim:'
        if (!id.startsWith(prefix)) return null
        return compileSuiToJs(id.slice(prefix.length))
      },
    },
  ],
})
