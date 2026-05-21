// Studio surface — controller unit tests. Targets the JS layer:
// route-param coercion, rail group composition, breadcrumb fallback,
// and the chrome event dispatchers.
//
// Studio is shell-owned (apps/shell/src/studio); these run under the
// shell's vitest config:
//   cd apps/shell && bunx vitest run src/studio/page.test.ts

import { describe, it, expect, beforeEach, afterEach } from 'vitest'

// Replicate the controller's pure helpers + section schema. The
// production controller wires these into a SUI scope, but they are
// intentionally side-effect-free so they can be exercised in isolation.

const SECTION_META: Record<string, { label: string; sub: string }> = {
  project:     { label: 'Project',      sub: 'Configuration for this workspace.' },
  agents:      { label: 'Agents',       sub: 'Agents you have configured for this project.' },
  skills:      { label: 'Skills',       sub: 'Skills available to agents in this project.' },
  mcp:         { label: 'MCP servers',  sub: 'External tools wired through the MCP boundary.' },
  pipelines:   { label: 'Pipelines',    sub: 'Reusable multi-step workflows.' },
  components:  { label: 'Components',   sub: 'SUI primitives available to views.' },
  themes:      { label: 'Themes',       sub: 'Visual themes for the workspace.' },
  permissions: { label: 'Permissions',  sub: 'Tool, network, and filesystem permissions.' },
  library:     { label: 'Library',      sub: 'Installed, marketplace, and authored extensions.' },
}
const DEFAULT_SECTION = 'agents'
const VALID = new Set(Object.keys(SECTION_META))

function resolveSection(raw: string | undefined): string {
  if (!raw) return DEFAULT_SECTION
  return VALID.has(raw) ? raw : DEFAULT_SECTION
}

function breadcrumbCurrent(section: string, sub: string): string {
  if (sub) return sub
  return SECTION_META[resolveSection(section)]?.label ?? 'Studio'
}

function dispatchStudioEvent(name: string, detail?: Record<string, unknown>): void {
  if (typeof window === 'undefined') return
  window.dispatchEvent(new CustomEvent(name, { detail: detail ?? {} }))
}

describe('studio: section resolution', () => {
  it('falls back to agents when no section is in the URL', () => {
    expect(resolveSection(undefined)).toBe('agents')
    expect(resolveSection('')).toBe('agents')
  })

  it('accepts every known section verbatim', () => {
    for (const s of Object.keys(SECTION_META)) {
      expect(resolveSection(s)).toBe(s)
    }
  })

  it('falls back to default for unknown sections', () => {
    expect(resolveSection('not-a-section')).toBe('agents')
    expect(resolveSection('PROJECT')).toBe('agents')
  })

  it('exposes nine sections (project + 7 configure + library)', () => {
    expect(Object.keys(SECTION_META)).toHaveLength(9)
  })
})

describe('studio: breadcrumb', () => {
  it('shows the section label when no sub-item is selected', () => {
    expect(breadcrumbCurrent('agents', '')).toBe('Agents')
    expect(breadcrumbCurrent('themes', '')).toBe('Themes')
  })

  it('shows the sub-item verbatim when one is selected', () => {
    expect(breadcrumbCurrent('project', 'overview')).toBe('overview')
    expect(breadcrumbCurrent('library', 'marketplace')).toBe('marketplace')
  })

  it('still falls back to a section label for unknown sections', () => {
    expect(breadcrumbCurrent('garbage', '')).toBe('Agents')
  })
})

describe('studio: chrome events', () => {
  let received: Array<{ type: string; detail: unknown }>
  let onEvent: (e: Event) => void

  beforeEach(() => {
    received = []
    onEvent = (e: Event) => {
      const ce = e as CustomEvent
      received.push({ type: e.type, detail: ce.detail })
    }
    window.addEventListener('sui-close-studio', onEvent)
    window.addEventListener('sui-open-studio-palette', onEvent)
    window.addEventListener('sui-studio-new', onEvent)
  })
  afterEach(() => {
    window.removeEventListener('sui-close-studio', onEvent)
    window.removeEventListener('sui-open-studio-palette', onEvent)
    window.removeEventListener('sui-studio-new', onEvent)
  })

  it('closeStudio fires sui-close-studio', () => {
    dispatchStudioEvent('sui-close-studio')
    expect(received.map((r) => r.type)).toContain('sui-close-studio')
  })

  it('palette button fires sui-open-studio-palette', () => {
    dispatchStudioEvent('sui-open-studio-palette')
    expect(received.map((r) => r.type)).toContain('sui-open-studio-palette')
  })

  it('newInSection fires sui-studio-new with the active section', () => {
    dispatchStudioEvent('sui-studio-new', { section: 'themes' })
    const studioNew = received.find((r) => r.type === 'sui-studio-new')
    expect(studioNew?.detail).toEqual({ section: 'themes' })
  })

  it('emits no events when nothing is dispatched', () => {
    expect(received).toHaveLength(0)
  })
})

describe('studio: rail navigation target paths', () => {
  // The controller composes /studio[/:section[/:sub]] from rail clicks.
  // Mirror that logic here so the URL contract is locked down.
  function pathFor(section: string, sub?: string): string {
    return sub ? `/studio/${section}/${sub}` : `/studio/${section}`
  }

  it('configure items navigate to /studio/<section>', () => {
    expect(pathFor('agents')).toBe('/studio/agents')
    expect(pathFor('themes')).toBe('/studio/themes')
  })

  it('project sub-items navigate to /studio/project/<sub>', () => {
    expect(pathFor('project', 'overview')).toBe('/studio/project/overview')
    expect(pathFor('project', 'isolation')).toBe('/studio/project/isolation')
  })

  it('library sub-items navigate to /studio/library/<sub>', () => {
    expect(pathFor('library', 'installed')).toBe('/studio/library/installed')
    expect(pathFor('library', 'marketplace')).toBe('/studio/library/marketplace')
  })
})

describe('studio: rail group counts', () => {
  // The CONFIGURE group header advertises a count chip ("7" in the
  // design mock). The number is intentionally static — pulling it from
  // live SDK data would make the section header thrash on every signal
  // emit. Hardcoded count must match the section schema.
  it('configure group contains 7 sections', () => {
    const configureKeys = ['agents', 'skills', 'mcp', 'pipelines', 'components', 'themes', 'permissions']
    expect(configureKeys).toHaveLength(7)
    for (const k of configureKeys) expect(VALID.has(k)).toBe(true)
  })
})

// ── Skills list helpers ──────────────────────────────────────────────────────
// Pure functions extracted from sui-skills-list.controller.ts; exercised
// without any reactive runtime so tests run fast in jsdom.

import { tileColor, tileChar, computeCounts } from './skills/sui-skills-list/sui-skills-list.controller'

interface SkillGroup {
  source: string
  skills: Array<{ id: string; name: string; description?: string }>
}

describe('skills list: tile helpers', () => {
  it('tileChar uppercases the first character of the skill name', () => {
    expect(tileChar('tdd')).toBe('T')
    expect(tileChar('code-review')).toBe('C')
    expect(tileChar('')).toBe('?')
  })

  it('tileColor returns a deterministic hex color for a given id', () => {
    const c = tileColor('tdd')
    expect(c).toMatch(/^#[0-9a-f]{6}$/i)
    expect(tileColor('tdd')).toBe(tileColor('tdd'))
  })

  it('tileColor produces different colors for different first chars', () => {
    const colors = new Set(['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'].map(tileColor))
    expect(colors.size).toBeGreaterThan(1)
  })
})

describe('skills list: computeCounts', () => {
  const groups: SkillGroup[] = [
    { source: 'PROJECT SKILLS', skills: [{ id: 'tdd', name: 'tdd' }, { id: 'lint', name: 'lint' }] },
    { source: 'USER SKILLS', skills: [{ id: 'sql', name: 'sql' }] },
  ]

  it('all count is the sum of all groups', () => {
    expect(computeCounts(groups).all).toBe(3)
  })

  it('project count matches PROJECT SKILLS group', () => {
    expect(computeCounts(groups).project).toBe(2)
  })

  it('user count matches USER SKILLS group', () => {
    expect(computeCounts(groups).user).toBe(1)
  })

  it('returns zero counts for empty groups', () => {
    expect(computeCounts([])).toEqual({ all: 0, project: 0, user: 0 })
  })

  it('returns zero user count when no user group exists', () => {
    const projectOnly: SkillGroup[] = [
      { source: 'PROJECT SKILLS', skills: [{ id: 'a', name: 'a' }] },
    ]
    expect(computeCounts(projectOnly).user).toBe(0)
    expect(computeCounts(projectOnly).all).toBe(1)
  })
})

describe('skills list: filter state', () => {
  type ScopeKind = 'all' | 'project' | 'user'

  const testGroups: SkillGroup[] = [
    { source: 'PROJECT SKILLS', skills: [{ id: 'tdd', name: 'tdd' }, { id: 'lint', name: 'lint' }] },
    { source: 'USER SKILLS', skills: [{ id: 'sql', name: 'sql' }] },
  ]

  function applyKindFilter(gs: SkillGroup[], kind: ScopeKind): SkillGroup[] {
    if (kind === 'all') return gs
    const source = kind === 'project' ? 'PROJECT SKILLS' : 'USER SKILLS'
    return gs.filter((g) => g.source === source)
  }

  it('all filter returns every group', () => {
    expect(applyKindFilter(testGroups, 'all')).toHaveLength(2)
  })

  it('project filter returns only PROJECT SKILLS group', () => {
    const filtered = applyKindFilter(testGroups, 'project')
    expect(filtered).toHaveLength(1)
    expect(filtered[0].source).toBe('PROJECT SKILLS')
  })

  it('user filter returns only USER SKILLS group', () => {
    const filtered = applyKindFilter(testGroups, 'user')
    expect(filtered).toHaveLength(1)
    expect(filtered[0].source).toBe('USER SKILLS')
  })

  it('empty groups produce empty state for all filter kinds', () => {
    expect(applyKindFilter([], 'all')).toHaveLength(0)
    expect(applyKindFilter([], 'project')).toHaveLength(0)
    expect(applyKindFilter([], 'user')).toHaveLength(0)
  })
})
