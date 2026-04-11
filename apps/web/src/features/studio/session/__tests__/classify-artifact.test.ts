import { describe, it, expect } from 'vitest'
import { classifyArtifact } from '../classify-artifact'
import type { SessionFile } from '../types'

function makeFile(overrides: Partial<SessionFile> = {}): SessionFile {
  return {
    name: 'doc.md',
    path: '.artifacts/doc.md',
    type: 'markdown',
    size: 512,
    modifiedAt: Date.now(),
    tier: 'workspace',
    ...overrides,
  }
}

describe('classifyArtifact detects spec type from frontmatter', () => {
  it('returns spec when frontmatter has type: spec', () => {
    const content = `---\ntitle: "Auth Flow"\ntype: spec\nstatus: draft\n---\n\n# Auth Flow`
    expect(classifyArtifact(makeFile(), content)).toBe('spec')
  })

  it('returns spec case-insensitively', () => {
    const content = `---\ntype: Spec\n---\n\n# Title`
    expect(classifyArtifact(makeFile(), content)).toBe('spec')
  })

  it('returns spec with +++ delimiters', () => {
    const content = `+++\ntype = "spec"\ntitle = "Auth"\n+++\n\n# Auth`
    expect(classifyArtifact(makeFile(), content)).toBe('spec')
  })
})

describe('classifyArtifact detects adr type from frontmatter', () => {
  it('returns adr when frontmatter has type: adr', () => {
    const content = `---\ntitle: "Use SQLite"\ntype: adr\nstatus: accepted\n---\n\n# ADR-001`
    expect(classifyArtifact(makeFile(), content)).toBe('adr')
  })

  it('returns adr case-insensitively', () => {
    const content = `---\ntype: ADR\n---\n\n# Title`
    expect(classifyArtifact(makeFile(), content)).toBe('adr')
  })
})

describe('classifyArtifact detects type from filename patterns', () => {
  it('returns spec for files matching *-spec.md', () => {
    const file = makeFile({ name: 'auth-spec.md', path: '.artifacts/auth-spec.md' })
    expect(classifyArtifact(file, '# Auth Spec\nSome content')).toBe('spec')
  })

  it('returns adr for files matching adr-*.md', () => {
    const file = makeFile({ name: 'adr-001-use-sqlite.md', path: '.artifacts/adr-001-use-sqlite.md' })
    expect(classifyArtifact(file, '# ADR 001\nSome content')).toBe('adr')
  })

  it('prefers frontmatter type over filename pattern', () => {
    const file = makeFile({ name: 'adr-001.md', path: '.artifacts/adr-001.md' })
    const content = `---\ntype: spec\n---\n\n# Actually a spec`
    expect(classifyArtifact(file, content)).toBe('spec')
  })
})

describe('classifyArtifact returns generic for other files', () => {
  it('returns generic for plain markdown without type frontmatter', () => {
    const content = `---\ntitle: "Notes"\nstatus: draft\n---\n\n# Notes`
    expect(classifyArtifact(makeFile(), content)).toBe('generic')
  })

  it('returns generic for markdown with no frontmatter', () => {
    expect(classifyArtifact(makeFile(), '# Just a heading')).toBe('generic')
  })

  it('returns generic for non-markdown files', () => {
    const file = makeFile({ name: 'data.json', type: 'other' })
    expect(classifyArtifact(file, '{"key": "value"}')).toBe('generic')
  })

  it('returns generic for image files', () => {
    const file = makeFile({ name: 'screenshot.png', type: 'image' })
    expect(classifyArtifact(file, '')).toBe('generic')
  })
})
