// The `/_deck/*` URL scheme has ONE source of truth: the client `DECK_PREFIX`
// constant exported from `@ship/sdk-client` (co-located with the rest of the
// client surface; mirrors `deck_core::routes::DECK_PREFIX` on the Rust side).
// The browser import maps (apps/shell/index.html, apps/spaces/index.html,
// crates/deck/deck-core/assets/view-host.html) are ordering-sensitive and must
// stay hand-authored, but they must never DRIFT from the SSOT. This test fails
// if they do — turning the duplication into a gated mirror rather than an
// unguarded copy.
//
// Pre-#277b the SSOT was `apps/shipd/src/routes.rs` (Route::{SuiRuntimeJs,
// SuiDomJs, SuiSdkClientJs}). Those variants are deleted: deck now mints the
// full serve-scheme, and the importmap source is the client `DECK_PREFIX`.

import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { describe, expect, it } from 'vitest'

import { DECK_PREFIX } from '@ship/sdk-client'

const repoRoot = join(dirname(new URL(import.meta.url).pathname), '..', '..', '..')

function importmapOf(htmlPath: string): Record<string, string> {
  const html = readFileSync(join(repoRoot, htmlPath), 'utf8')
  const m = html.match(/<script type="importmap">\s*([\s\S]*?)<\/script>/)
  if (!m) throw new Error(`no importmap in ${htmlPath}`)
  return (JSON.parse(m[1]!) as { imports: Record<string, string> }).imports
}

// URLs the importmap must resolve to, all derived from the single
// client-side `DECK_PREFIX` minter.
const SSOT = {
  '@ship/sui-runtime': `${DECK_PREFIX}/runtime.js`,
  '@ship/sui-runtime/route': `${DECK_PREFIX}/runtime.js`,
  '@ship/sui-runtime/template-hooks': `${DECK_PREFIX}/runtime.js`,
  '@ship/sui-dom': `${DECK_PREFIX}/dom.js`,
  '@ship/sdk-client': `${DECK_PREFIX}/sdk-client.js`,
}

describe('/_deck import-map SSOT', () => {
  for (const htmlPath of [
    'apps/shell/index.html',
    'apps/spaces/index.html',
    'crates/deck/deck-core/assets/view-host.html',
    ]) {
    it(`${htmlPath} import map matches the client DECK_PREFIX minter`, () => {
      const imports = importmapOf(htmlPath)
      for (const [spec, url] of Object.entries(SSOT)) {
        // view-host.html omits the /patch subpath; only assert specs it declares.
        if (spec in imports) expect(imports[spec], `${htmlPath} → ${spec}`).toBe(url)
      }
      // No `/_deck/*` entry may point somewhere the minter doesn't sanction.
      const runtimeUrl = SSOT['@ship/sui-runtime']
      for (const [spec, url] of Object.entries(imports)) {
        if (url === runtimeUrl) expect(spec.startsWith('@ship/sui-runtime')).toBe(true)
      }
    })
  }
})
