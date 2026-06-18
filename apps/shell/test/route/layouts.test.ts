import { describe, expect, it } from 'vitest'
import { resolveLayoutChain, type LayoutShape } from '../../src/route/layouts'

const LAYOUTS: LayoutShape[] = [
  { pathPrefix: '/studio', tag: 'sui-studio-chrome' },
  { pathPrefix: '/studio/agents', tag: 'sui-agents-chrome' },
]

describe('resolveLayoutChain', () => {
  it('returns [] for empty LAYOUTS', () => {
    expect(resolveLayoutChain('/studio/agents/abc', [])).toEqual([])
  })

  it('returns [] for empty URL', () => {
    expect(resolveLayoutChain('', LAYOUTS)).toEqual([])
  })

  it('returns [] when no layout matches', () => {
    expect(resolveLayoutChain('/home', LAYOUTS)).toEqual([])
  })

  it('returns the single matching layout', () => {
    expect(resolveLayoutChain('/studio/pipelines', LAYOUTS)).toEqual([
      { pathPrefix: '/studio', tag: 'sui-studio-chrome' },
    ])
  })

  it('returns the deepest matching layout first (deepest-first ordering)', () => {
    expect(resolveLayoutChain('/studio/agents/abc', LAYOUTS)).toEqual([
      { pathPrefix: '/studio/agents', tag: 'sui-agents-chrome' },
      { pathPrefix: '/studio', tag: 'sui-studio-chrome' },
    ])
  })

  it('returns [] for the root URL when LAYOUTS do not include /', () => {
    expect(resolveLayoutChain('/', LAYOUTS)).toEqual([])
  })

  it('matches / only when a layout declares pathPrefix=/', () => {
    const rootLayout: LayoutShape[] = [{ pathPrefix: '/', tag: 'sui-shell-chrome' }]
    expect(resolveLayoutChain('/', rootLayout)).toEqual([
      { pathPrefix: '/', tag: 'sui-shell-chrome' },
    ])
    expect(resolveLayoutChain('/anything', rootLayout)).toEqual([
      { pathPrefix: '/', tag: 'sui-shell-chrome' },
    ])
  })

  it('does not match /studio-other for /studio prefix', () => {
    expect(resolveLayoutChain('/studio-other/page', LAYOUTS)).toEqual([])
  })

  it('strips query string from URL', () => {
    expect(resolveLayoutChain('/studio/agents/abc?ref=foo', LAYOUTS)).toEqual([
      { pathPrefix: '/studio/agents', tag: 'sui-agents-chrome' },
      { pathPrefix: '/studio', tag: 'sui-studio-chrome' },
    ])
  })

  it('strips hash from URL', () => {
    expect(resolveLayoutChain('/studio/agents/abc#section', LAYOUTS)).toEqual([
      { pathPrefix: '/studio/agents', tag: 'sui-agents-chrome' },
      { pathPrefix: '/studio', tag: 'sui-studio-chrome' },
    ])
  })

  it('accepts a URL object', () => {
    const u = new URL('https://example.test/studio/agents/abc?q=1')
    expect(resolveLayoutChain(u, LAYOUTS)).toEqual([
      { pathPrefix: '/studio/agents', tag: 'sui-agents-chrome' },
      { pathPrefix: '/studio', tag: 'sui-studio-chrome' },
    ])
  })

  it('skips invalid entries (empty pathPrefix, non-string pathPrefix)', () => {
    const mixed: LayoutShape[] = [
      { pathPrefix: '', tag: 'skip-me' },
      // @ts-expect-error - runtime guard for malformed entries
      { pathPrefix: null, tag: 'skip-me-2' },
      { pathPrefix: '/studio', tag: 'sui-studio-chrome' },
    ]
    expect(resolveLayoutChain('/studio/agents/abc', mixed)).toEqual([
      { pathPrefix: '/studio', tag: 'sui-studio-chrome' },
    ])
  })

  it('handles a single-character pathPrefix', () => {
    expect(resolveLayoutChain('/a', [{ pathPrefix: '/a', tag: 'one' }])).toEqual([
      { pathPrefix: '/a', tag: 'one' },
    ])
  })

  it('returns multiple prefix matches ordered by length DESC', () => {
    const many: LayoutShape[] = [
      { pathPrefix: '/', tag: 'root' },
      { pathPrefix: '/a', tag: 'one' },
      { pathPrefix: '/a/b', tag: 'two' },
      { pathPrefix: '/a/b/c', tag: 'three' },
    ]
    expect(resolveLayoutChain('/a/b/c/d', many)).toEqual([
      { pathPrefix: '/a/b/c', tag: 'three' },
      { pathPrefix: '/a/b', tag: 'two' },
      { pathPrefix: '/a', tag: 'one' },
      { pathPrefix: '/', tag: 'root' },
    ])
  })
})
