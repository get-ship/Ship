import { defineConfig } from 'vitest/config'
import { resolve, dirname, basename } from 'node:path'
import { existsSync, readFileSync, renameSync } from 'node:fs'
import { spawnSync } from 'node:child_process'

const repoRoot = resolve(__dirname, '../..')

export default defineConfig({
  resolve: {
    alias: {
      '@ship/ui-runtime': resolve(__dirname, '../ui-runtime/src/index.ts'),
      // @ship/ui-dom/legacy is aliased for the test-only `*.compiled.js` shim; not bundled into production.
      '@ship/ui-dom/legacy': resolve(__dirname, '../ui-dom/src/legacy/index.ts'),
      '@ship/ui-dom': resolve(__dirname, '../ui-dom/src/index.ts'),
    },
  },
  test: {
    include: ['test/**/*.test.ts', 'src/**/*.test.ts'],
    environment: 'jsdom',
    setupFiles: ['./test/setup.ts'],
  },
  // `<tag>.compiled.js` shim. Production bundles get an actual AOT-compiled
  // module emitted by the Rust sui-compiler at build time. For tests, this
  // plugin intercepts `.compiled.js` imports and either:
  //   (a) passes through to the file on disk if it already exists (prior build), or
  //   (b) invokes the sui-compiler via `cargo run` to compile the sibling `.sui`
  //       source, then passes through to the freshly compiled file.
  //
  // The legacy `.html.set` fallback is kept for any remaining legacy files.
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

        // (a) Already compiled on disk — return absolute path so vite resolves it directly.
        if (existsSync(actualFile)) return actualFile

        // (b) Compile the sibling .sui source via the Rust sui-compiler.
        const suiFile = resolve(dir, `${stem}.sui`)
        if (existsSync(suiFile)) {
          // The compiler outputs <tagName>.js into the directory containing the
          // .sui file. For imports like './sui-status-dot/sui-status-dot.compiled.js'
          // from src/index.ts, stem = 'sui-status-dot/sui-status-dot' so we must
          // derive the output dir and tag name from the resolved suiFile path.
          const compDir = dirname(suiFile)
          const tagName = basename(suiFile, '.sui')
          const r = spawnSync(
            'cargo',
            ['run', '--quiet', '-p', 'sui-compiler', '--', 'build', suiFile, '--out', compDir, '--component-name', tagName],
            { cwd: repoRoot, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] },
          )
          if (r.status === 0) {
            const emitted = resolve(compDir, `${tagName}.js`)
            if (existsSync(emitted)) renameSync(emitted, actualFile)
          } else {
            console.error(`[compiled-js-test-shim] sui-compiler failed for ${tagName}:\n${r.stderr || r.stdout}`)
          }
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
