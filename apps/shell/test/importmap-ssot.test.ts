// The `/_sui/*` URL scheme has ONE source of truth: apps/shipd/src/routes.rs,
// from which build.rs generates the TS builders in @ship/sdk. The browser
// import maps (apps/shell/index.html, crates/deck/deck-core/assets/view-host.html)
// are ordering-sensitive and must stay hand-authored, but they must never DRIFT
// from the SSOT. This test fails if they do — turning the duplication into a
// gated mirror rather than an unguarded copy.

import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { describe, expect, it } from 'vitest'

import { suiRuntimeJs, suiDomJs, suiSdkClientJs } from '@ship/sdk'

const repoRoot = join(dirname(new URL(import.meta.url).pathname), '..', '..', '..')

function importmapOf(htmlPath: string): Record<string, string> {
  const html = readFileSync(join(repoRoot, htmlPath), 'utf8')
  const m = html.match(/<script type="importmap">\s*([\s\S]*?)<\/script>/)
  if (!m) throw new Error(`no importmap in ${htmlPath}`)
  return (JSON.parse(m[1]!) as { imports: Record<string, string> }).imports
}

const SSOT = {
  '@ship/sui-runtime': suiRuntimeJs(),
  '@ship/sui-runtime/route': suiRuntimeJs(),
  '@ship/sui-runtime/template-hooks': suiRuntimeJs(),
  '@ship/sui-dom': suiDomJs(),
  '@ship/sdk-client': suiSdkClientJs(),
}

describe('/_sui import-map SSOT', () => {
  for (const htmlPath of [
    'apps/shell/index.html',
    'crates/deck/deck-core/assets/view-host.html',
  ]) {
    it(`${htmlPath} import map matches the generated route builders`, () => {
      const imports = importmapOf(htmlPath)
      for (const [spec, url] of Object.entries(SSOT)) {
        // view-host.html omits the /patch subpath; only assert specs it declares.
        if (spec in imports) expect(imports[spec], `${htmlPath} → ${spec}`).toBe(url)
      }
      // No `/_sui/*` entry may point somewhere the builders don't sanction.
      for (const [spec, url] of Object.entries(imports)) {
        if (url === suiRuntimeJs()) expect(spec.startsWith('@ship/sui-runtime')).toBe(true)
      }
    })
  }
})
