// TESTS-FIRST — sui-sheet FOUNDATIONAL slice: the ENGINE-PURITY grep gate.
//
// Status: RED until the engine ships. This is a STATIC SCAN of the engine source
// (not an import of it), so its red is "no engine source files exist yet" — the
// precondition `engine source files are present` fails. Once the foundational
// slice lands, the scan asserts the boundary holds (Cardinal 4 + the sui-*
// boundary): the engine is pure reactive state — no SDK, no network, no <data>.
//
// Authority: build-spec §"Backbone — createSheet() cell engine" ("Imports
// { signal, derived, batch, untrack } from @ship/sui-runtime ONLY — no
// @ship/sdk-client, no ship.*, no fetch, no <data>"), §Slices ("Engine purity is
// NOT lint-enforced … Assert via a grep test … the grep also asserts the vendor
// dir holds no ship.*/fetch"), behavioral spec #14 ("a grep/test scan confirms no
// @ship/sdk-client/ship.*/fetch/EventSource/<data> in sui-sheet/ (engine OR
// vendor) … Test-asserted; not sui-lint-asserted").
//
// NOTE — engine purity is TEST-asserted here, NOT sui-lint (sui-lint targets
// .sui/.controller.ts and never sees these .ts modules). The scan recurses so
// it also covers the (future) vendor/formula-core/ dir once it lands.

import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const ENGINE_DIR = dirname(fileURLToPath(import.meta.url))
const PACK_DIR = resolve(ENGINE_DIR, '..')
const VENDOR_DIR = join(PACK_DIR, 'vendor', 'formula-core')

// Recurse a dir collecting every .ts/.mjs/.js EXCEPT the *.test.ts specs (tests
// legitimately name the forbidden tokens in comments/strings; the impl must not).
function sourceFiles(dir: string): string[] {
  if (!existsSync(dir)) return []
  const out: string[] = []
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry)
    if (statSync(full).isDirectory()) {
      out.push(...sourceFiles(full))
      continue
    }
    if (entry.endsWith('.test.ts')) continue
    if (/\.(ts|mjs|js)$/.test(entry) && !entry.endsWith('.d.ts')) out.push(full)
  }
  return out
}

const engineSourceFiles = (): string[] => sourceFiles(ENGINE_DIR)

const FORBIDDEN: { token: string; why: string }[] = [
  { token: '@ship/sdk-client', why: 'SDK import — the engine is pure reactive state' },
  { token: 'ship.', why: 'ship.* call — backend coupling belongs in the consumer controller' },
  { token: 'fetch(', why: 'network call — the engine never performs I/O' },
  { token: 'EventSource', why: "server-sent events — not the engine's concern" },
  { token: '<data', why: '<data> directive — a view/controller concern, never the engine' },
]

describe('engine purity (grep gate — Cardinal 4 + sui-* boundary)', () => {
  it('the engine has SOURCE modules to scan (precondition — RED while greenfield)', () => {
    expect(engineSourceFiles().length).toBeGreaterThan(0)
  })

  it('imports the runtime ONLY — no @ship/sdk-client anywhere under engine/', () => {
    for (const file of engineSourceFiles()) {
      const src = readFileSync(file, 'utf8')
      expect(src, `${file} must not import @ship/sdk-client`).not.toContain('@ship/sdk-client')
    }
  })

  it('contains NONE of the forbidden boundary tokens (ship.*/fetch/EventSource/<data)', () => {
    for (const file of engineSourceFiles()) {
      const src = readFileSync(file, 'utf8')
      for (const { token, why } of FORBIDDEN) {
        expect(src.includes(token), `${file} contains forbidden "${token}" — ${why}`).toBe(false)
      }
    }
  })

  it('every engine module that uses reactivity imports it from @ship/sui-runtime', () => {
    for (const file of engineSourceFiles()) {
      const src = readFileSync(file, 'utf8')
      const importsReactivity = /\b(signal|derived|batch|untrack)\b/.test(src)
      if (importsReactivity) {
        expect(src, `${file} uses reactivity but does not import @ship/sui-runtime`).toContain(
          '@ship/sui-runtime',
        )
      }
    }
  })
})

describe('formula-core vendor boundary (the load-bearing seam)', () => {
  // The vendor dir is opaque math — it must know nothing of Ship. Until the
  // vendor slice lands the dir does not exist; once it does, this scan asserts
  // the bundle carries no ship.* / fetch / network coupling.
  it('the vendor dir, when present, contains no forbidden boundary tokens', () => {
    for (const file of sourceFiles(VENDOR_DIR)) {
      const src = readFileSync(file, 'utf8')
      for (const token of ['@ship/sdk-client', 'ship.', 'fetch(', 'EventSource']) {
        expect(src.includes(token), `${file} (vendor) contains forbidden "${token}"`).toBe(false)
      }
    }
  })

  it('ONLY the adapter imports from vendor/formula-core/ (one seam, swappable)', () => {
    // No file under engine/ may import the vendor EXCEPT the named adapter — the
    // engine binds to the FormulaCore interface, never the vendor API. (This
    // gate stays meaningful even pre-vendor: zero engine files reference it yet.)
    const offenders = engineSourceFiles().filter((file) => {
      if (file.endsWith('adapter-fast-formula-parser.ts')) return false
      return readFileSync(file, 'utf8').includes('vendor/formula-core')
    })
    expect(offenders, `only adapter-fast-formula-parser.ts may import the vendor`).toEqual([])
  })
})
