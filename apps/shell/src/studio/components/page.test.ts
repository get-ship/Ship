// Components Studio section — list-surface logic tests.
//
// Run under the shell's vitest config:
//   cd apps/shell && bunx vitest run src/studio/components/page.test.ts
//
// Targets the deterministic core behind <sui-components-list>: the
// editorial category schema, the catalog→category projection, and the
// grouping that produces the scannable grouped index (criteria 1 + 5).

import { describe, it, expect } from 'vitest'
import { CATEGORIES, TAG_META, metaFor, type CategoryId } from './component-meta'
import { groupByCategory, type ComponentEntry } from './catalog'

function entry(tag: string, over: Partial<ComponentEntry> = {}): ComponentEntry {
  return {
    tag,
    category: metaFor(tag).category,
    description: metaFor(tag).blurb,
    props: [],
    slots: [],
    ...over,
  }
}

describe('components: category schema', () => {
  it('exposes the seven design categories plus charts and other', () => {
    expect(CATEGORIES.map((c) => c.id)).toEqual([
      'actions', 'inputs', 'layout', 'feedback',
      'indicators', 'overlays', 'data', 'charts', 'other',
    ])
  })

  it('assigns every curated tag to a known category', () => {
    const ids = new Set<CategoryId>(CATEGORIES.map((c) => c.id))
    for (const [tag, meta] of Object.entries(TAG_META)) {
      expect(ids.has(meta.category), `${tag} → ${meta.category}`).toBe(true)
      expect(meta.blurb.length, `${tag} blurb`).toBeGreaterThan(0)
    }
  })

  it('places the canonical primitives in the expected groups', () => {
    expect(metaFor('sui-button').category).toBe('actions')
    expect(metaFor('sui-input').category).toBe('inputs')
    expect(metaFor('sui-card').category).toBe('layout')
    expect(metaFor('sui-alert').category).toBe('feedback')
    expect(metaFor('sui-badge').category).toBe('indicators')
    expect(metaFor('sui-dialog').category).toBe('overlays')
    expect(metaFor('sui-table').category).toBe('data')
    expect(metaFor('sui-bar-chart').category).toBe('charts')
  })

  it('falls back to "other" for an unknown tag', () => {
    expect(metaFor('sui-not-real').category).toBe('other')
    expect(metaFor('sui-not-real').blurb).toBe('')
  })
})

describe('components: grouping', () => {
  it('groups entries into categories in display order, dropping empties', () => {
    const groups = groupByCategory([
      entry('sui-table'),   // data
      entry('sui-button'),  // actions
      entry('sui-input'),   // inputs
    ])
    expect(groups.map((g) => g.id)).toEqual(['actions', 'inputs', 'data'])
    expect(groups[0].items.map((i) => i.tag)).toEqual(['sui-button'])
    expect(groups.find((g) => g.id === 'feedback')).toBeUndefined()
  })

  it('keeps multiple members within one category', () => {
    const groups = groupByCategory([
      entry('sui-button'), entry('sui-toggle'), entry('sui-toolbar'),
    ])
    expect(groups).toHaveLength(1)
    expect(groups[0].id).toBe('actions')
    expect(groups[0].items).toHaveLength(3)
  })

  it('routes an uncategorized tag into the trailing "other" group', () => {
    const groups = groupByCategory([entry('sui-button'), entry('sui-mystery')])
    expect(groups.map((g) => g.id)).toEqual(['actions', 'other'])
  })
})
