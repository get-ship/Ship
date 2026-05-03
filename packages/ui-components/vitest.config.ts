import { defineConfig } from 'vitest/config'
import { existsSync, readFileSync, renameSync } from 'node:fs'
import { spawnSync } from 'node:child_process'
import { resolve, dirname } from 'node:path'

// Prefer the pre-built debug binary; fall back to `cargo run` if absent.
const compilerBin = resolve(__dirname, '../../target/debug/sui-compiler')

function compileSui(suiPath: string, outDir: string, stem: string): void {
  const hasbin = existsSync(compilerBin)
  const cmd = hasbin ? compilerBin : 'cargo'
  const args = hasbin
    ? ['build', suiPath, '--out', outDir, '--component-name', stem, '--no-dts']
    : ['run', '--quiet', '-p', 'sui-compiler', '--', 'build', suiPath, '--out', outDir, '--component-name', stem, '--no-dts']
  const result = spawnSync(cmd, args, {
    cwd: resolve(__dirname, '../..'),
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  })
  if (result.status !== 0) {
    throw new Error(`sui-compiler failed for ${suiPath}:\n${result.stderr || result.stdout}`)
  }
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
  },
  // Two transform paths for SUI primitives:
  //
  //  1. `.html.set` raw-text loader (legacy format). Retained for any
  //     components that haven't been ported to `.sui` yet.
  //
  //  2. `<tag>.compiled.js` shim. Production bundles use AOT-compiled
  //     modules emitted by the Rust sui-compiler. For tests:
  //     - If the `.compiled.js` already exists (built by build-bundles or
  //       a prior test run), resolve it directly.
  //     - If a sibling `.sui` file exists, compile it on-demand using the
  //       Rust binary (target/debug/sui-compiler) and write the output
  //       next to the source as `<stem>.compiled.js`.
  //     - Otherwise fall back to the legacy hydrate path for `.html.set`.
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
        const importerDir = resolve(importer, '..')
        const rel = id.replace(/^\.\//, '').replace(/\.compiled\.js$/, '')
        const basePath = resolve(importerDir, rel)
        const compiledPath = `${basePath}.compiled.js`

        // Already built — use the physical file (handles build-bundles output
        // and on-demand compilations from a prior test run).
        if (existsSync(compiledPath)) return compiledPath

        // .sui source present — compile on-demand via the Rust binary.
        const suiPath = `${basePath}.sui`
        if (existsSync(suiPath)) {
          // outDir and tag-stem must match the .sui file's own directory.
          const suiDir = dirname(suiPath)
          const tagStem = suiPath.split('/').pop()!.replace(/\.sui$/, '')
          compileSui(suiPath, suiDir, tagStem)
          // Compiler emits <tagStem>.js; rename to <tagStem>.compiled.js.
          renameSync(`${suiDir}/${tagStem}.js`, compiledPath)
          return compiledPath
        }

        // Legacy fallback: .html.set + runtime hydrate path.
        return `\0compiled-js-shim:${basePath}`
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
