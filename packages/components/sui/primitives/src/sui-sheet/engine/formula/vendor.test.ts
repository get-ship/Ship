// TESTS-FIRST — sui-sheet FORMULA slice (Slice 6a): the vendored formula-core.
//
// Status: RED. The vendor dir `../../vendor/formula-core/` does not exist yet —
// this is a STATIC SCAN of the vendor artifact (not an import of it), so the red
// is "the vendor bundle has not been prepared." Once Slice 6a hand-prepares the
// `fast-formula-parser` ESM bundle + bundled `.d.ts` + verbatim LICENSE (mirroring
// `packages/sui/runtime/src/vendor/virtual-core/` and `packages/arcade-engine/
// vendor/pixi.js/`), this scan asserts the artifact shape + the PERMISSIVE-ONLY
// license law. DO NOT add the impl/bundle here. DO NOT weaken into passing stubs.
//
// Authority: build-spec §"The formula-core vendor boundary" ("Vendored (opaque):
// …/vendor/formula-core/ — the fast-formula-parser ESM bundle (.mjs + bundled
// .d.ts) + the upstream LICENSE verbatim, version pinned in a header comment …
// Hand-prepared bundle, mirroring packages/arcade-engine/vendor/pixi.js/ …
// Permissive-only — fast-formula-parser is expected MIT … HyperFormula is GPLv3
// per upstream and is the banned alternative"), §Slice 6a ("produce the ESM
// bundle + bundled .d.ts as a hand-prepared artifact … record the version in a
// header comment + the lockfile. Confirm the license is MIT (NOT GPL/AGPL) BEFORE
// committing the vendor dir"), Behavioral spec #16 ("the vendor dir carries a
// verbatim LICENSE confirmed permissive (MIT); no GPL/AGPL anywhere in the pack").
//
// The boundary-token scan (no ship.*/fetch in the vendor) + the "only the adapter
// imports the vendor" rule live in `engine/purity.test.ts` (the foundational
// gate, which already recurses into this dir). This file pins the ARTIFACT SHAPE
// and the LICENSE LAW that purity.test.ts does not cover.

import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const HERE = dirname(fileURLToPath(import.meta.url)) // .../sui-sheet/engine/formula
const PACK_DIR = resolve(HERE, '..', '..') // .../sui-sheet
const VENDOR_DIR = join(PACK_DIR, 'vendor', 'formula-core')

const read = (rel: string): string => readFileSync(join(VENDOR_DIR, rel), 'utf8')

describe('vendor/formula-core — the hand-prepared bundle artifact exists', () => {
  it('the vendor dir is present (mirrors the virtual-core precedent)', () => {
    expect(existsSync(VENDOR_DIR)).toBe(true)
  })

  it('ships a single-file ESM bundle (a .mjs), NOT a node_modules dep', () => {
    const files = existsSync(VENDOR_DIR) ? readdirSync(VENDOR_DIR) : []
    expect(files.some((f) => f.endsWith('.mjs'))).toBe(true)
  })

  it('ships a bundled type declaration (.d.ts / .d.mts)', () => {
    const files = existsSync(VENDOR_DIR) ? readdirSync(VENDOR_DIR) : []
    expect(files.some((f) => f.endsWith('.d.ts') || f.endsWith('.d.mts'))).toBe(true)
  })

  it('ships a verbatim upstream LICENSE file', () => {
    expect(existsSync(join(VENDOR_DIR, 'LICENSE'))).toBe(true)
  })

  it('pins the upstream version in a header comment of the bundle', () => {
    const files = existsSync(VENDOR_DIR) ? readdirSync(VENDOR_DIR) : []
    const mjs = files.find((f) => f.endsWith('.mjs'))
    expect(mjs, 'a .mjs bundle must exist to carry the version header').toBeDefined()
    const header = read(mjs!).slice(0, 600)
    // a pinned semver in the leading comment (e.g. "fast-formula-parser X.Y.Z").
    expect(header).toMatch(/\d+\.\d+\.\d+/)
  })
})

describe('vendor/formula-core — PERMISSIVE-ONLY license law (the load-bearing rule)', () => {
  it('the LICENSE is permissive (MIT family), NOT copyleft', () => {
    const license = existsSync(join(VENDOR_DIR, 'LICENSE')) ? read('LICENSE') : ''
    // MIT is the expected outcome; the assertion is the LAW, not the brand —
    // any permissive license that names itself MIT/BSD/ISC/Apache passes.
    expect(license).toMatch(/MIT|BSD|ISC|Apache/i)
  })

  it('the LICENSE is NOT GPL/AGPL/LGPL (HyperFormula-class copyleft is banned)', () => {
    const license = existsSync(join(VENDOR_DIR, 'LICENSE')) ? read('LICENSE') : ''
    expect(license).not.toMatch(/GNU GENERAL PUBLIC LICENSE/i)
    expect(license).not.toMatch(/AFFERO/i)
    expect(license).not.toMatch(/LESSER GENERAL PUBLIC/i)
  })

  it('the LICENSE carries a copyright line (verbatim upstream text, not a stub)', () => {
    const license = existsSync(join(VENDOR_DIR, 'LICENSE')) ? read('LICENSE') : ''
    expect(license).toMatch(/copyright/i)
    expect(license.length).toBeGreaterThan(200) // a real license body, not a placeholder
  })
})
