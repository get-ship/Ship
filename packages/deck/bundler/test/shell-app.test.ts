// Unit tests for the shell-app build's importmap-derived externals — the rule
// that the build externalizes EXACTLY the specifiers index.html's importmap
// resolves to /_sui URLs, so the two can never drift (#157 SSOT).

import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import { mkdir, mkdtemp, readdir, rm, writeFile } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import {
  findRepoRoot,
  importmapExternals,
  materializeFrameworkSurface,
} from '../shell-app.ts'

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

// The self-contained ("bundled hub") path: every `/_sui/*` URL the importmap +
// IIFE name must land in `<outDir>/_sui/` so the prod Tauri webview (no daemon
// proxy) resolves its own code same-origin under CSP `script-src 'self'`.
describe('materializeFrameworkSurface', () => {
  let root: string
  let frameworkDir: string
  let deckDir: string
  let outDir: string

  const FRAMEWORK = ['sui-runtime.js', 'sui-dom.js', 'sui-sdk-client.js', 'sui-components.js']

  beforeEach(async () => {
    root = await mkdtemp(join(tmpdir(), 'shell-app-'))
    frameworkDir = join(root, 'assets')
    deckDir = join(root, '.deck')
    outDir = join(root, 'dist')
    await mkdir(frameworkDir, { recursive: true })
    await mkdir(join(deckDir, 'sui-components'), { recursive: true })
    for (const f of FRAMEWORK) await writeFile(join(frameworkDir, f), `// ${f}`)
    await writeFile(join(frameworkDir, 'lucide-icons.json'), '{}')
    for (const tag of ['sui-chrome.js', 'sui-launcher.js']) {
      await writeFile(join(deckDir, 'sui-components', tag), `// ${tag}`)
    }
  })

  afterEach(async () => {
    await rm(root, { recursive: true, force: true })
  })

  test('materializes framework bundles + every island into outDir/_sui', async () => {
    const count = await materializeFrameworkSurface(outDir, frameworkDir, deckDir)
    // 5 framework files (4 js + lucide) + 2 island bundles.
    expect(count).toBe(7)
    // Importmap/IIFE URL names — runtime.js NOT sui-runtime.js.
    expect(existsSync(join(outDir, '_sui/runtime.js'))).toBe(true)
    expect(existsSync(join(outDir, '_sui/dom.js'))).toBe(true)
    expect(existsSync(join(outDir, '_sui/sdk-client.js'))).toBe(true)
    expect(existsSync(join(outDir, '_sui/components.js'))).toBe(true)
    expect(existsSync(join(outDir, '_sui/lucide-icons.json'))).toBe(true)
    const islands = await readdir(join(outDir, '_sui/sui-components'))
    expect(islands.sort()).toEqual(['sui-chrome.js', 'sui-launcher.js'])
  })

  test('throws when a framework asset is missing (fail loud, not 404 at runtime)', async () => {
    await rm(join(frameworkDir, 'sui-runtime.js'))
    await expect(materializeFrameworkSurface(outDir, frameworkDir, deckDir)).rejects.toThrow(
      /missing framework asset/,
    )
  })

  test('throws when the island dir is absent (deck build not run)', async () => {
    await rm(join(deckDir, 'sui-components'), { recursive: true })
    await expect(materializeFrameworkSurface(outDir, frameworkDir, deckDir)).rejects.toThrow(
      /run the deck bundler/,
    )
  })

  test('findRepoRoot locates the ancestor holding deck-core/assets', () => {
    expect(findRepoRoot(import.meta.dir)).toBe(findRepoRoot(process.cwd()))
    expect(existsSync(join(findRepoRoot(import.meta.dir), 'crates/deck/deck-core/assets'))).toBe(
      true,
    )
  })
})
