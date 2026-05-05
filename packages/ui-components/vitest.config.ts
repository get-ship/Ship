import { defineConfig } from 'vitest/config'
import { resolve, dirname, basename } from 'node:path'
import { existsSync, readFileSync, renameSync } from 'node:fs'
import { SHIP_VITEST_DEFAULTS } from '../../vitest.shared'
// @ts-expect-error — JS module, no .d.ts.
import { spawnBoundedOrThrow } from '../../scripts/spawn-bounded.mjs'

const repoRoot = resolve(__dirname, '../..')
const compilerBin = resolve(repoRoot, 'target/debug/sui-compiler')

// Test-time fallback for the watch/HMR loop. The `pretest` hook (see
// package.json) precompiles every .sui ahead of vitest, so cold runs
// never hit this path. spawnBoundedOrThrow wraps the child in
// `prlimit + timeout` so a buggy compile dies cleanly without taking the
// host VM with it.
function compileSui(suiPath: string, outDir: string, stem: string): void {
  const hasBin = existsSync(compilerBin)
  const cmd = hasBin ? compilerBin : 'cargo'
  const args = hasBin
    ? ['build', suiPath, '--out', outDir, '--component-name', stem, '--no-dts']
    : ['run', '--quiet', '-p', 'sui-compiler', '--', 'build', suiPath, '--out', outDir, '--component-name', stem, '--no-dts']
  spawnBoundedOrThrow(cmd, args, `sui-compiler ${suiPath}`, { cwd: repoRoot })
}

export default defineConfig({
  resolve: {
    alias: {
      '@ship/ui-runtime': resolve(__dirname, '../ui-runtime/src/index.ts'),
      // @ship/ui-dom/legacy is aliased for the test-only `*.compiled.js` shim; not bundled into production.
      '@ship/ui-dom/legacy': resolve(__dirname, '../ui-dom/src/legacy/index.ts'),
      '@ship/ui-dom': resolve(__dirname, '../ui-dom/src/index.ts'),
      // No-op stub: WebGL addon throws "WebGL2 not supported" in JSDOM.
      '@xterm/addon-webgl': resolve(__dirname, 'test/__mocks__/addon-webgl.ts'),
    },
  },
  test: {
    include: ['test/**/*.test.ts', 'src/**/*.test.ts'],
    environment: 'jsdom',
    setupFiles: ['./test/setup.ts'],
    ...SHIP_VITEST_DEFAULTS,
  },
  // `<tag>.compiled.js` shim. Production bundles use AOT-compiled modules
  // emitted by the Rust sui-compiler. For tests:
  //   (a) If the `.compiled.js` already exists (built by build-bundles or
  //       a prior test run), resolve it directly.
  //   (b) If a sibling `.sui` file exists, compile it on-demand via the Rust
  //       binary (or `cargo run` fallback) and write the output next to the
  //       source as `<stem>.compiled.js`.
  //   (c) Otherwise fall back to the legacy `.html.set` hydrate path.
  plugins: [
    {
      name: 'html-set-text-loader',
      transform(_code: string, id: string) {
        if (!id.endsWith('.html.set')) return null
        const source = readFileSync(id, 'utf8')
        return {
          code: `export default ${JSON.stringify(source)}`,
          map: null,
        }
      },
    },
    {
      name: 'compiled-js-test-shim',
      resolveId(id: string, importer?: string) {
        if (!id.endsWith('.compiled.js') || !importer) return null
        const dir = resolve(importer, '..')
        const stem = id.replace(/^\.\//, '').replace(/\.compiled\.js$/, '')
        const actualFile = resolve(dir, `${stem}.compiled.js`)

        if (existsSync(actualFile)) return actualFile

        const suiFile = resolve(dir, `${stem}.sui`)
        if (existsSync(suiFile)) {
          const compDir = dirname(suiFile)
          const tagName = basename(suiFile, '.sui')
          compileSui(suiFile, compDir, tagName)
          const emitted = resolve(compDir, `${tagName}.js`)
          if (existsSync(emitted)) renameSync(emitted, actualFile)
          if (existsSync(actualFile)) return actualFile
        }

        // Legacy fallback: virtual module using the old hydrate path.
        return `\0compiled-js-shim:${resolve(dir, stem)}`
      },
      load(id: string) {
        const prefix = '\0compiled-js-shim:'
        if (!id.startsWith(prefix)) return null
        const base = id.slice(prefix.length)
        const htmlSet = `${base}.html.set`
        return [
          `import { hydrate } from '@ship/ui-dom/legacy'`,
          `import template from ${JSON.stringify(htmlSet)}`,
          `if (typeof document !== 'undefined') {`,
          `  const __c = document.createElement('div')`,
          `  hydrate(template, undefined, { container: __c })`,
          `}`,
        ].join('\n')
      },
    },
  ],
})
