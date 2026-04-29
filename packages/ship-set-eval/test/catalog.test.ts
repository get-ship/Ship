import { describe, expect, it } from 'vitest'
import { loadCatalog, parseCatalog } from '../src/catalog'
import { mockCatalog } from '../src/mock-catalog'

describe('mockCatalog', () => {
  it('covers the five required primitives', () => {
    const tags = mockCatalog.primitives.map((p) => p.tag).sort()
    expect(tags).toEqual([
      'ship-button',
      'ship-empty-state',
      'ship-input',
      'ship-list-tile',
      'ship-spinner',
    ])
  })

  it('marks ship-input.value as bindable', () => {
    const input = mockCatalog.primitives.find((p) => p.tag === 'ship-input')!
    const value = input.props.find((p) => p.name === 'value')!
    expect(value.bindable).toBe(true)
  })
})

describe('loadCatalog', () => {
  it('returns the mock catalog with stable shape', async () => {
    const cat = await loadCatalog()
    expect(cat.version).toBeTruthy()
    expect(cat.primitives.length).toBeGreaterThan(0)
  })
})

describe('parseCatalog', () => {
  it('round-trips the mock catalog through JSON', () => {
    const json = JSON.parse(JSON.stringify(mockCatalog))
    const parsed = parseCatalog(json)
    expect(parsed.primitives.length).toBe(mockCatalog.primitives.length)
    expect(parsed.primitives[0]?.tag).toBe(mockCatalog.primitives[0]?.tag)
  })

  it('rejects a non-object root', () => {
    expect(() => parseCatalog([] as unknown)).toThrow(/object/)
    expect(() => parseCatalog(null as unknown)).toThrow(/object/)
  })

  it('rejects a missing version', () => {
    expect(() => parseCatalog({ primitives: [] })).toThrow(/version/)
  })

  it('rejects an unknown prop type', () => {
    expect(() =>
      parseCatalog({
        version: 'v1',
        primitives: [{ tag: 'x', props: [{ name: 'p', type: 'date' }] }],
      }),
    ).toThrow(/type/)
  })

  it('requires non-empty values[] on a union prop', () => {
    expect(() =>
      parseCatalog({
        version: 'v1',
        primitives: [{ tag: 'x', props: [{ name: 'p', type: 'union' }] }],
      }),
    ).toThrow(/values/)
  })

  it('keeps optional summary when present', () => {
    const parsed = parseCatalog({
      version: 'v1',
      primitives: [{ tag: 'x', summary: 'a thing' }],
    })
    expect(parsed.primitives[0]?.summary).toBe('a thing')
  })
})
