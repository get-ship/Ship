import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { readFileSync } from 'node:fs'

export default defineConfig({
  resolve: {
    alias: {
      '@ship-set/runtime': resolve(__dirname, '../ship-set-runtime/src/index.ts'),
      '@ship-set/dom': resolve(__dirname, '../ship-set-dom/src/index.ts'),
    },
  },
  test: {
    include: ['test/**/*.test.ts', 'src/**/*.test.ts'],
    environment: 'jsdom',
  },
  // Components import their `.html.set` template as a string. Vitest's
  // build pipeline doesn't know how to handle that extension; teach it
  // via a tiny inline plugin that reads the file and exports its raw
  // contents as the default export.
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
  ],
})
