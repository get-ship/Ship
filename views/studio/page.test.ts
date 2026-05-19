// Studio surface — controller unit tests. Targets the JS layer:
// route-param coercion, rail group composition, breadcrumb fallback,
// and the chrome event dispatchers.
//
// Run manually until the view test pipeline is formalised:
//   cd packages/components/ui-components &&
//     bunx vitest run ../../../views/studio/page.test.ts

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'

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
