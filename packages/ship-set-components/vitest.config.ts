import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { readFileSync } from 'node:fs'

export default defineConfig({
  resolve: {
    alias: {
      '@ship-set/runtime': resolve(__dirname, '../ship-set-runtime/src/index.ts'),
      // @ship-set/dom is aliased for the test-only `*.compiled.js` shim below; it is not bundled into production.
      '@ship-set/dom': resolve(__dirname, '../ship-set-dom/src/index.ts'),
    },
  },
  test: {
    include: ['test/**/*.test.ts', 'src/**/*.test.ts'],
    environment: 'jsdom',
  },
  // Two transform shims for `.html.set` ported primitives:
  //
  //  1. `.html.set` raw-text loader so the legacy define-side path
  //     can read templates as strings (still used by anything that
  //     imports the template directly).
  //
  //  2. `<tag>.compiled.js` shim. Production bundles get an actual
  //     AOT-compiled module emitted by the Rust ship-set-compiler at
  //     build time. Vitest doesn't run cargo, so for tests we
  //     synthesize an equivalent side-effect module that re-uses the
  //     runtime hydrate path against the sibling `.html.set`. Net
  //     effect for the test environment: importing `<tag>.ts`
  //     (which side-effect imports `<tag>.compiled.js`) registers
  //     the custom element, exactly like the AOT module would.
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
        // Resolve relative to the importer so the shim sees the
        // sibling .html.set and our virtual module id stays unique.
        const dir = resolve(importer, '..')
        const stem = id.replace(/^\.\//, '').replace(/\.compiled\.js$/, '')
        return `\0compiled-js-shim:${resolve(dir, stem)}`
      },
      load(id: string) {
        const prefix = '\0compiled-js-shim:'
        if (!id.startsWith(prefix)) return null
        const base = id.slice(prefix.length)
        const htmlSet = `${base}.html.set`
        return [
          `import { hydrate } from '@ship-set/dom'`,
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
