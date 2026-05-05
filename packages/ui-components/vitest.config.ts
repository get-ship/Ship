import { defineConfig } from 'vitest/config'
import { resolve, dirname, basename } from 'node:path'
import { existsSync, readFileSync } from 'node:fs'
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
function compileSui(suiPath: string, stem: string): void {
  const hasBin = existsSync(compilerBin)
  const cmd = hasBin ? compilerBin : 'cargo'
  const args = hasBin
    ? ['build', suiPath, '--component-name', stem, '--no-dts']
    : ['run', '--quiet', '-p', 'sui-compiler', '--', 'build', suiPath, '--component-name', stem, '--no-dts']
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
  // emitted by the Rust sui-compiler into a sibling `dist/` directory.
  // For tests, an `import '.../<stem>.compiled.js'` is resolved as:
  //   (a) `<importer-dir>/dist/<stem>.compiled.js` if it already exists
  //       (built by build-bundles, the pretest precompile hook, or a prior
  //       test run).
  //   (b) Otherwise, if a sibling `<stem>.sui` source file exists, compile
  //       it on-demand via the Rust binary (or `cargo run` fallback). The
  //       compiler emits to `dist/<stem>.compiled.js` automatically.
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
        // Resolve the import to the absolute artifact path. Test files
        // import the dist-based path directly, e.g.
        // `'../src/sui-foo/dist/sui-foo.compiled.js'`, so a relative
        // resolution from the importer hits the actual artifact location.
        const importerDir = resolve(importer, '..')
        const actualFile = resolve(importerDir, id)

        if (existsSync(actualFile)) return actualFile

        // Source layout: `<source-dir>/<stem>.sui` → emit at
        // `<source-dir>/dist/<stem>.compiled.js`. Walk one segment up
        // from the artifact's directory to find the source.
        const stem = basename(actualFile).replace(/\.compiled\.js$/, '')
        const sourceDir = dirname(dirname(actualFile))
        const suiFile = resolve(sourceDir, `${stem}.sui`)
        if (existsSync(suiFile)) {
          compileSui(suiFile, stem)
          if (existsSync(actualFile)) return actualFile
        }

        // Legacy fallback: virtual module using the old hydrate path.
        return `\0compiled-js-shim:${resolve(sourceDir, stem)}`
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
