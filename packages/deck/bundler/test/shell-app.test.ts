// Unit tests for the shell-app build's importmap-derived externals — the rule
// that the build externalizes EXACTLY the specifiers index.html's importmap
// resolves to /_sui URLs, so the two can never drift (#157 SSOT).

import { describe, test, expect } from 'bun:test'
import { importmapExternals } from '../shell-app.ts'

const withImportmap = (json: string) =>
  `<!doctype html><head><script type="importmap">${json}</script></head>`

describe('importmapExternals', () => {
  test('returns the importmap keys as the external specifier list', () => {
    const html = withImportmap(
      JSON.stringify({
        imports: {
          '@ship/sui-runtime': '/_sui/runtime.js',
          '@ship/sui-runtime/route': '/_sui/runtime.js',
          '@ship/sui-dom': '/_sui/dom.js',
          '@ship/sdk-client': '/_sui/sdk-client.js',
        },
      }),
    )
    expect(importmapExternals(html)).toEqual([
      '@ship/sui-runtime',
      '@ship/sui-runtime/route',
      '@ship/sui-dom',
      '@ship/sdk-client',
    ])
  })

  test('throws when there is no importmap', () => {
    expect(() => importmapExternals('<head></head>')).toThrow(/no <script type="importmap">/)
  })

  test('throws on malformed importmap JSON', () => {
    expect(() => importmapExternals(withImportmap('{ not json'))).toThrow(/not valid JSON/)
  })

  test('throws on an empty imports map', () => {
    expect(() => importmapExternals(withImportmap('{"imports":{}}'))).toThrow(/no imports/)
  })
})
