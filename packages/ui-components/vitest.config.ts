import { defineConfig } from 'vitest/config'
import { resolve, dirname, basename, relative } from 'node:path'
import { existsSync, readFileSync } from 'node:fs'
import { SHIP_VITEST_DEFAULTS } from '../../vitest.shared'
// @ts-expect-error — JS module, no .d.ts.
import { spawnBoundedOrThrow } from '../../scripts/spawn-bounded.mjs'

const repoRoot = resolve(__dirname, '../..')
const compilerBin = resolve(repoRoot, 'target/debug/sui-compiler')
const mirrorRoot = resolve(repoRoot, '.ship/sui-out')

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

// Map a `.compiled.js` virtual import to its mirror-tree artifact path.
// Source layout: `<repo>/<src-rel>/<stem>.sui` → emit at
// `<repo>/.ship/sui-out/<src-rel>/<stem>.compiled.js`.
function mirrorArtifactFor(sourceDir: string, stem: string): string {
  const srcRel = relative(repoRoot, sourceDir)
  return resolve(mirrorRoot, srcRel, `${stem}.compiled.js`)
}

export default defineConfig({
  resolve: {
    alias: {
      '@ship/sui-runtime': resolve(__dirname, '../sui-runtime/src/index.ts'),
      '@ship/primitives': resolve(__dirname, '../components/primitives/src/index.ts'),
      // @ship/sui-dom/legacy is aliased for the test-only `*.compiled.js` shim; not bundled into production.
      '@ship/sui-dom/legacy': resolve(__dirname, '../sui-dom/src/legacy/index.ts'),
      '@ship/sui-dom': resolve(__dirname, '../sui-dom/src/index.ts'),
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
  // emitted by the Rust sui-compiler into the project mirror tree at
  // `<repo-root>/.ship/sui-out/<src-rel>/`.
  // For tests, an `import '.../<stem>.compiled.js'` is resolved as:
  //   (a) `<mirror-root>/<src-rel>/<stem>.compiled.js` if it already exists
  //       (built by build-bundles, the pretest precompile hook, or a prior
  //       test run).
  //   (b) Otherwise, if a sibling `<stem>.sui` source file exists, compile
  //       it on-demand via the Rust binary (or `cargo run` fallback). The
  //       compiler emits to the mirror tree automatically.
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
        // Resolve the import against the importer's directory to recover
        // the source-tree location, then look up the mirror-tree artifact.
        const importerDir = resolve(importer, '..')
        const sourceArtifact = resolve(importerDir, id)
        const sourceDir = dirname(sourceArtifact)
        const stem = basename(sourceArtifact).replace(/\.compiled\.js$/, '')
        const mirrorFile = mirrorArtifactFor(sourceDir, stem)

        if (existsSync(mirrorFile)) return mirrorFile

        // No mirror artifact yet — compile on-demand if a sibling .sui exists.
        const suiFile = resolve(sourceDir, `${stem}.sui`)
        if (existsSync(suiFile)) {
          compileSui(suiFile, stem)
          if (existsSync(mirrorFile)) return mirrorFile
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
          `import { hydrate } from '@ship/sui-dom/legacy'`,
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
