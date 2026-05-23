import { defineConfig } from 'vitest/config'
import { resolve, dirname, basename, relative } from 'node:path'
import { existsSync } from 'node:fs'
import { SHIP_VITEST_DEFAULTS, compileSuiToJs } from '../../vitest.shared'
// @ts-expect-error — JS module, no .d.ts.
import { spawnBoundedOrThrow } from '../../scripts/spawn-bounded.mjs'

const repoRoot = resolve(__dirname, '../..')
const compilerBin = resolve(repoRoot, 'target/debug/sui-compiler')
const mirrorRoot = resolve(repoRoot, '.ship/sui-out')

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
    alias: {
      '@ship/sui-runtime': resolve(__dirname, '../../packages/sui/runtime/src/index.ts'),
      '@ship/primitives': resolve(__dirname, '../../packages/components/primitives/src/index.ts'),
      '@ship/sui-dom': resolve(__dirname, '../../packages/sui/dom/src/index.ts'),
      '@xterm/addon-webgl': resolve(__dirname, 'test/__mocks__/addon-webgl.ts'),
    },
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
