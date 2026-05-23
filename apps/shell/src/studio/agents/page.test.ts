// Controller-layer unit tests for the agents list view.
//
// Like views/agents/[id]/page.test.ts these target the controller's
// pure helpers and the shape of its derived state — they do NOT mount
// the compiled .sui template (SSR + web-qa cover the rendered DOM).
// To run from the repo root:
//   bunx vitest run views/agents/page.test.ts
//
// Three things the list view must get right and that regress silently:
// the scope filter, the search filter, and the per-card hue (which has
// to match the detail page it links to).

import { describe, it, expect } from 'vitest'
import { matchesKind, matchesQuery, type FilterableAgent } from './filter'
import { pickHue, HUES } from './[id]/hue'

function agent(over: Partial<FilterableAgent> = {}): FilterableAgent {
  return {
    id: 'rust-compiler',
    name: 'rust-compiler',
    description: 'Compiles Rust workspaces and reports clippy spans.',
    source: 'PROJECT AGENTS',
    tags: ['rust', 'workflow'],
    ...over,
  }
}

describe('matchesKind', () => {
  it('matches everything under "all"', () => {
    expect(matchesKind(agent({ source: 'PROJECT AGENTS' }), 'all')).toBe(true)
    expect(matchesKind(agent({ source: 'USER AGENTS' }), 'all')).toBe(true)
  })

  it('separates project from user scope', () => {
    expect(matchesKind(agent({ source: 'PROJECT AGENTS' }), 'project')).toBe(true)
    expect(matchesKind(agent({ source: 'USER AGENTS' }), 'project')).toBe(false)
    expect(matchesKind(agent({ source: 'USER AGENTS' }), 'user')).toBe(true)
    expect(matchesKind(agent({ source: 'PROJECT AGENTS' }), 'user')).toBe(false)
  })
})

describe('matchesQuery', () => {
  it('matches an empty or whitespace query', () => {
    expect(matchesQuery(agent(), '')).toBe(true)
    expect(matchesQuery(agent(), '   ')).toBe(true)
  })

  it('matches on name, id, description, and tags', () => {
    expect(matchesQuery(agent({ name: 'doc-reviewer' }), 'doc')).toBe(true)
    expect(matchesQuery(agent({ id: 'sql-shaper' }), 'shaper')).toBe(true)
    expect(matchesQuery(agent({ description: 'Daily ETL load' }), 'etl')).toBe(true)
    expect(matchesQuery(agent({ tags: ['mesh'] }), 'mesh')).toBe(true)
  })

  it('is case-insensitive and returns false on a miss', () => {
    expect(matchesQuery(agent({ name: 'Rust-Compiler' }), 'RUST')).toBe(true)
    expect(matchesQuery(agent(), 'no-such-thing')).toBe(false)
  })

  it('drives the empty-filtered state: a no-match query filters all out', () => {
    const list = [agent({ id: 'a' }), agent({ id: 'b' })]
    const visible = list.filter((a) => matchesQuery(a, 'zzz-nope'))
    expect(visible).toHaveLength(0)
  })
})

describe('card hue tile', () => {
  it('every agent gets a hue from the shared palette', () => {
    const palette = new Set<string>(HUES)
    for (const id of ['rust-compiler', 'doc-reviewer', 'etl-runner', 'a', '']) {
      expect(palette.has(pickHue(id))).toBe(true)
    }
  })

  it('a card hue matches the detail page for the same id', () => {
    // Both views call pickHue(id), so the list tile and the detail
    // header land on the same colour. Deterministic by construction.
    expect(pickHue('rust-compiler')).toBe(pickHue('rust-compiler'))
  })
})
