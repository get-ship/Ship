import { defineConfig } from 'vitest/config'
import { resolve, dirname, basename, relative } from 'node:path'
import { existsSync } from 'node:fs'
import { SHIP_VITEST_DEFAULTS, compileSuiToJs } from '../../../vitest.shared'
// @ts-expect-error — JS module, no .d.ts.
import { spawnBoundedOrThrow } from '../../../scripts/spawn-bounded.mjs'

const repoRoot = resolve(__dirname, '../../..')
const compilerBin = resolve(repoRoot, 'target/debug/sui-compiler')
const mirrorRoot = resolve(repoRoot, '.ship/sui-out')

// Test-time fallback for the watch/HMR loop. The `pretest` hook
// precompiles every `.sui` ahead of vitest, so cold runs never hit
// this path.
function compileSui(suiPath: string, stem: string): void {
  const hasBin = existsSync(compilerBin)
  const cmd = hasBin ? compilerBin : 'cargo'
  const args = hasBin
    ? ['build', suiPath, '--component-name', stem, '--no-dts']
    : ['run', '--quiet', '-p', 'sui-compiler', '--', 'build', suiPath, '--component-name', stem, '--no-dts']
  spawnBoundedOrThrow(cmd, args, `sui-compiler ${suiPath}`, { cwd: repoRoot })
}

function mirrorArtifactFor(sourceDir: string, stem: string): string {
  const srcRel = relative(repoRoot, sourceDir)
  return resolve(mirrorRoot, srcRel, `${stem}.compiled.js`)
}

export default defineConfig({
  resolve: {
    alias: [
      { find: '@ship/sui-runtime', replacement: resolve(__dirname, '../runtime/src/index.ts') },
      { find: '@ship/sui-dom', replacement: resolve(__dirname, '../dom/src/index.ts') },
      // The bridge inlines Excalidraw's 144 KB stylesheet by a relative
      // node_modules path; under jsdom we don't need it (the bridge guards
      // an empty value), so stub it to keep the test run light + hermetic.
      { find: '../../node_modules/@excalidraw/excalidraw/dist/prod/index.css', replacement: resolve(__dirname, 'test/empty-css.ts') },
    ],
  },
  test: {
    include: ['test/**/*.test.ts', 'src/**/*.test.ts'],
    environment: 'jsdom',
    setupFiles: ['./test/setup.ts'],
    ...SHIP_VITEST_DEFAULTS,
  },
  plugins: [
    {
      name: 'compiled-js-test-shim',
      resolveId(id: string, importer?: string) {
        if (!id.endsWith('.compiled.js') || !importer) return null
        const importerDir = resolve(importer, '..')
        const sourceArtifact = resolve(importerDir, id)
        const sourceDir = dirname(sourceArtifact)
        const stem = basename(sourceArtifact).replace(/\.compiled\.js$/, '')
        const mirrorFile = mirrorArtifactFor(sourceDir, stem)

        if (existsSync(mirrorFile)) return mirrorFile

        const suiFile = resolve(sourceDir, `${stem}.sui`)
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
        const suiPath = id.slice(prefix.length)
        return compileSuiToJs(suiPath)
      },
    },
  ],
})
