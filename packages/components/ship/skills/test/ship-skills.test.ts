// Tests for the @ship/skills pack's pure-derived helpers.
//
// The pack's controllers are thin wrappers around the live SDK
// (ship.skills.list / ship.skills.detail), so the meaningful logic
// is the defensive narrowing + grouping, not the I/O. We test the
// narrowing + grouping helpers directly via the `test/helpers.ts`
// module — they're the contract the templates depend on, and a
// regression in either would surface as silently missing skills or
// a wrong scope label on the hub `/studio/skills` surface.

import { describe, expect, it } from 'vitest'
import {
  buildGroupsFromList,
  frontmatterRowsFrom,
  SCOPE_LABELS,
  SCOPE_ORDER,
  type SkillIndexEntry,
} from './helpers'
import {
  readDetail,
  type SkillDetail,
} from '../src/ship-skill-detail/ship-skill-detail.controller'
import {
  readList,
} from '../src/ship-skill-list/ship-skill-list.controller'

describe('@ship/skills · grouping', () => {
  it('groups known scopes in stable order (project, user, library)', () => {
    const list: SkillIndexEntry[] = [
      { id: 'lib-1', scope: 'library', name: 'Lib 1', description: '' },
      { id: 'user-1', scope: 'user', name: 'User 1', description: '' },
      { id: 'proj-1', scope: 'project', name: 'Proj 1', description: '' },
    ]
    const groups = buildGroupsFromList(list)
    expect(groups.map((g) => g.id)).toEqual(['project', 'user', 'library'])
  })

  it('appends unknown scopes after the known order', () => {
    const list: SkillIndexEntry[] = [
      { id: 'future-1', scope: 'org', name: 'Org', description: '' },
      { id: 'proj-1', scope: 'project', name: 'Proj 1', description: '' },
    ]
    const groups = buildGroupsFromList(list)
    expect(groups.map((g) => g.id)).toEqual(['project', 'org'])
  })

  it('drops empty scope buckets', () => {
    const list: SkillIndexEntry[] = [
      { id: 'proj-1', scope: 'project', name: 'Proj 1', description: '' },
    ]
    const groups = buildGroupsFromList(list)
    expect(groups.map((g) => g.id)).toEqual(['project'])
  })

  it('exposes stable scope labels', () => {
    expect(SCOPE_LABELS.project).toBe('Project')
    expect(SCOPE_LABELS.user).toBe('User')
    expect(SCOPE_LABELS.library).toBe('Library')
  })

  it('the scope order is exactly project, user, library', () => {
    expect(SCOPE_ORDER).toEqual(['project', 'user', 'library'])
  })
})

describe('@ship/skills · readList narrowing', () => {
  it('returns null for non-object payloads', () => {
    expect(readList(null)).toBeNull()
    expect(readList('nope')).toBeNull()
    expect(readList(42)).toBeNull()
  })

  it('returns null when items is not an array', () => {
    expect(readList({ items: 'nope' })).toBeNull()
    expect(readList({})).toBeNull()
  })

  it('skips entries that are not objects', () => {
    const out = readList({ items: [null, 'nope', 42, { id: 'a', scope: 'project', name: 'A', description: '' }] })
    expect(out?.items).toEqual([
      { id: 'a', scope: 'project', name: 'A', description: '' },
    ])
  })

  it('falls back to id when name is missing (defensive)', () => {
    const out = readList({ items: [{ id: 'proj-1', scope: 'project' }] })
    expect(out?.items[0]?.name).toBe('proj-1')
  })

  it('defaults missing scope to "project"', () => {
    const out = readList({ items: [{ id: 'a' }] })
    expect(out?.items[0]?.scope).toBe('project')
  })

  it('defaults missing description to empty string', () => {
    const out = readList({ items: [{ id: 'a', scope: 'project', name: 'A' }] })
    expect(out?.items[0]?.description).toBe('')
  })
})

describe('@ship/skills · readDetail narrowing', () => {
  const valid: SkillDetail = {
    id: 'web-search',
    scope: 'project',
    frontmatter: { name: 'Web Search', description: 'search the web' },
    body: '# hello',
    files: ['assets/x.png'],
  }

  it('returns the detail when id matches', () => {
    expect(readDetail(valid, 'web-search')).toEqual(valid)
  })

  it('returns null when id does not match (stale-id race)', () => {
    expect(readDetail(valid, 'other-skill')).toBeNull()
  })

  it('returns null for non-object payloads', () => {
    expect(readDetail(null, 'x')).toBeNull()
    expect(readDetail('nope', 'x')).toBeNull()
    expect(readDetail(42, 'x')).toBeNull()
  })

  it('returns null for an object missing id', () => {
    expect(readDetail({ scope: 'project', body: 'x' }, 'x')).toBeNull()
  })

  it('defaults missing frontmatter to {} and missing body/files safely', () => {
    const out = readDetail({ id: 'a', scope: 'project' }, 'a')
    expect(out).toEqual({
      id: 'a',
      scope: 'project',
      frontmatter: {},
      body: '',
      files: [],
    })
  })

  it('defaults missing scope to "project" (the daemon default)', () => {
    const out = readDetail({ id: 'a', body: 'x' }, 'a')
    expect(out?.scope).toBe('project')
  })

  it('filters non-string entries in files', () => {
    const out = readDetail(
      { id: 'a', scope: 'project', body: '', files: ['a.png', 42, null, 'b.md'] },
      'a',
    )
    expect(out?.files).toEqual(['a.png', 'b.md'])
  })
})

describe('@ship/skills · frontmatter row rendering', () => {
  it('renders scalars as their string form', () => {
    const rows = frontmatterRowsFrom({
      str: 'hello',
      num: 42,
      bool: true,
      big: 10n,
      nothing: null,
    })
    expect(rows).toEqual([
      { key: 'str', value: 'hello' },
      { key: 'num', value: '42' },
      { key: 'bool', value: 'true' },
      { key: 'big', value: '10' },
      { key: 'nothing', value: '' },
    ])
  })

  it('JSON-encodes objects and arrays so they do not render as [object Object]', () => {
    const rows = frontmatterRowsFrom({
      list: ['a', 'b'],
      obj: { nested: true },
    })
    expect(rows.find((r) => r.key === 'list')?.value).toBe('["a","b"]')
    expect(rows.find((r) => r.key === 'obj')?.value).toBe('{"nested":true}')
  })

  it('preserves the frontmatter insertion order', () => {
    const rows = frontmatterRowsFrom({ z: '1', a: '2', m: '3' })
    expect(rows.map((r) => r.key)).toEqual(['z', 'a', 'm'])
  })
})
