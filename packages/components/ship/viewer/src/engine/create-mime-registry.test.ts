// The createMimeRegistry() engine driven by THIS router's default kind→TAG
// table. The pure resolution engine + MIME sniffers live in @ship/sui-media; the
// `defaultEntries()` dispatch table (which renderer a kind routes to) lives here
// in @ship/viewer. This spec exercises the engine seeded with that table — the
// behavior the <ship-viewer> dispatcher relies on.
//
// Authority: build-spec §"Backbone #1 — createMimeRegistry() + the <ship-viewer>
// dispatcher", §"API sketch (the load-bearing surface)", §"Resolution order is
// FIXED and validated", §"The default registry table (the eight kinds)",
// Slices 1 + 1b; behavioral specs #1, #2, #12.

import { describe, expect, it, vi } from 'vitest'
// Import the PURE engine pieces directly from @ship/sui-media's engine tree (by
// path) — NOT the package barrel `@ship/sui-media`, whose side-effect imports of
// the renderer `.compiled.js` would compile the leaf `.sui` files (a registration
// concern this pure-resolver spec has no business triggering).
import { createMimeRegistry } from '../../../../sui/media/src/engine/create-mime-registry'
import { normalizeContentType, sniffMagicBytes } from '../../../../sui/media/src/engine/detect'
import type {
  SourceDescriptor,
  ViewerKind,
  ViewerSource,
} from '../../../../sui/media/src/engine/types'
import { defaultEntries } from './registry'

// The engine is generic over the entry list; the router seeds it with its
// default kind→TAG table. Every spec here exercises that seeded registry.
const createMimeRegistry_ = () => createMimeRegistry({ entries: defaultEntries() })

// A magic-byte fixture: the first bytes of a real file. `head` is what the
// sniffer inspects (no I/O — the consumer already resolved these bytes).
const bytesOf = (...nums: number[]): Uint8Array => Uint8Array.from(nums)
const PDF_HEAD = bytesOf(0x25, 0x50, 0x44, 0x46, 0x2d) // "%PDF-"
const PNG_HEAD = bytesOf(0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a)
const JPEG_HEAD = bytesOf(0xff, 0xd8, 0xff, 0xe0)

describe('normalizeContentType — strips params, lowercases', () => {
  it('drops the charset/params and trims', () => {
    expect(normalizeContentType('text/csv;charset=utf-8')).toBe('text/csv')
    expect(normalizeContentType('Application/PDF')).toBe('application/pdf')
    expect(normalizeContentType('  text/markdown ; boundary=x ')).toBe('text/markdown')
  })

  it('returns empty string for an empty/blank content-type', () => {
    expect(normalizeContentType('')).toBe('')
    expect(normalizeContentType('   ')).toBe('')
  })
})

describe('sniffMagicBytes — pure byte inspection, no I/O', () => {
  it('recognizes %PDF- as pdf', () => {
    expect(sniffMagicBytes(PDF_HEAD)).toBe('pdf')
  })

  it('recognizes the PNG and JPEG signatures as image', () => {
    expect(sniffMagicBytes(PNG_HEAD)).toBe('image')
    expect(sniffMagicBytes(JPEG_HEAD)).toBe('image')
  })

  it('returns null for bytes with no known signature', () => {
    expect(sniffMagicBytes(bytesOf(0x00, 0x01, 0x02, 0x03))).toBeNull()
  })
})

describe('createMimeRegistry — resolve() one-shot (pure)', () => {
  it('resolves application/json text to the json renderer', () => {
    const reg = createMimeRegistry_()
    const d = reg.resolve({ contentType: 'application/json', text: '{"a":1}' })
    expect(d.kind).toBe('json')
    expect(d.renderer).toBe('sui-json-view')
  })

  it('resolves text/csv to the tabular renderer (composed sui-table)', () => {
    const reg = createMimeRegistry_()
    const d = reg.resolve({ contentType: 'text/csv', text: 'a,b\n1,2' })
    expect(d.kind).toBe('tabular')
    expect(d.renderer).toBe('sui-table')
  })

  it('resolves text/markdown to the markdown renderer', () => {
    const reg = createMimeRegistry_()
    expect(reg.resolve({ contentType: 'text/markdown', text: '# Hi' }).renderer).toBe('sui-markdown')
  })

  it('resolves a unified-diff content-type to the diff renderer', () => {
    const reg = createMimeRegistry_()
    expect(reg.resolve({ contentType: 'text/x-diff', text: '@@ -1 +1 @@' }).renderer).toBe('sui-diff')
  })

  it('normalizes the content-type before matching (charset params ignored)', () => {
    const reg = createMimeRegistry_()
    const d = reg.resolve({ contentType: 'text/csv;charset=utf-8', text: 'a,b' })
    expect(d.kind).toBe('tabular')
    expect(d.normalizedType).toBe('text/csv')
  })

  it('falls back to extension when no content-type is given', () => {
    const reg = createMimeRegistry_()
    const d = reg.resolve({ filename: 'report.csv', text: 'a,b\n1,2' })
    expect(d.kind).toBe('tabular')
    expect(d.reason).toBe('extension')
  })

  it('content-type BEATS a conflicting extension (precedence pinned)', () => {
    const reg = createMimeRegistry_()
    const d = reg.resolve({ contentType: 'application/pdf', filename: 'x.png' })
    expect(d.kind).toBe('pdf')
    expect(d.renderer).toBe('sui-media-pdf')
    expect(d.reason).toBe('content-type')
  })

  it('magic-byte sniff is the tiebreaker when type+ext are absent', () => {
    const reg = createMimeRegistry_()
    const d = reg.resolve({ bytes: PDF_HEAD.buffer })
    expect(d.kind).toBe('pdf')
    expect(d.reason).toBe('magic-bytes')
  })

  it('an unknown content-type resolves to unsupported with a null renderer', () => {
    const reg = createMimeRegistry_()
    const d = reg.resolve({ contentType: 'application/x-unknown' })
    expect(d.kind).toBe('unsupported')
    expect(d.renderer).toBeNull()
  })
})

describe('createMimeRegistry — reactive source → descriptor', () => {
  it('setSource updates the descriptor / kind / renderer deriveds', () => {
    const reg = createMimeRegistry_()
    reg.setSource({ contentType: 'application/json', text: '{"a":1}' })
    expect(reg.kind()).toBe('json')
    expect(reg.renderer()).toBe('sui-json-view')
    expect(reg.isSupported()).toBe(true)

    reg.setSource({ contentType: 'text/markdown', text: '# Hi' })
    expect(reg.kind()).toBe('markdown')
    expect(reg.renderer()).toBe('sui-markdown')
  })

  it('descriptor recomputes only on setSource (memoized between reads)', () => {
    const reg = createMimeRegistry_()
    reg.setSource({ contentType: 'text/csv', text: 'a,b' })
    const first = reg.descriptor()
    const second = reg.descriptor()
    // No intervening setSource → identical descriptor object identity.
    expect(second).toBe(first)
  })

  it('a null source is unsupported (no renderer, not supported)', () => {
    const reg = createMimeRegistry_()
    reg.setSource(null)
    expect(reg.kind()).toBe('unsupported')
    expect(reg.renderer()).toBeNull()
    expect(reg.isSupported()).toBe(false)
  })
})

describe('createMimeRegistry — register() extends the table (one entry = one type)', () => {
  it('a new entry changes resolution WITHOUT touching the dispatcher (#12)', () => {
    const reg = createMimeRegistry_()
    // Before: application/wasm is unknown.
    expect(reg.resolve({ contentType: 'application/wasm' }).kind).toBe('unsupported')

    reg.register({
      id: 'wasm',
      kind: 'unsupported' as ViewerKind,
      renderer: 'sui-viewer-wasm',
      matchesType: (t) => t === 'application/wasm',
    })
    const d = reg.resolve({ contentType: 'application/wasm' })
    expect(d.renderer).toBe('sui-viewer-wasm')
  })

  it('a registered renderer entry routes a fresh source to the new tag', () => {
    const reg = createMimeRegistry_()
    reg.register({
      id: 'tabular',
      kind: 'tabular',
      renderer: 'sui-media-csv',
      matchesType: (t) => t === 'application/x-acme-table',
    })
    reg.setSource({ contentType: 'application/x-acme-table', text: 'a,b' })
    expect(reg.renderer()).toBe('sui-media-csv')
  })
})

describe('createMimeRegistry — never fetches (boundary)', () => {
  it('resolve() inspects supplied bytes and makes no network call', () => {
    const fetchSpy = vi.spyOn(globalThis, 'fetch' as never).mockImplementation((() => {
      throw new Error('viewers engine must never fetch')
    }) as never)
    const reg = createMimeRegistry_()
    const d: SourceDescriptor = reg.resolve({ bytes: PNG_HEAD.buffer } as ViewerSource)
    expect(d.kind).toBe('image')
    expect(fetchSpy).not.toHaveBeenCalled()
    fetchSpy.mockRestore()
  })
})
