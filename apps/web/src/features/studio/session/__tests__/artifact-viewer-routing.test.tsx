import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import { ArtifactViewer } from '../ArtifactViewer'
import type { SessionFile } from '../types'

afterEach(cleanup)

// Mock the lazy-loaded editor and sub-renderers
vi.mock('@ship/primitives', () => ({
  MarkdownEditor: () => <div data-testid="markdown-editor" />,
}))

vi.mock('../ImageViewer', () => ({
  ImageViewer: () => <div data-testid="image-viewer" />,
}))

vi.mock('../SpecRenderer', () => ({
  SpecRenderer: ({ file }: { file: SessionFile }) => (
    <div data-testid="spec-renderer">{file.name}</div>
  ),
}))

vi.mock('../AdrRenderer', () => ({
  AdrRenderer: ({ file }: { file: SessionFile }) => (
    <div data-testid="adr-renderer">{file.name}</div>
  ),
}))

vi.mock('../classify-artifact', () => ({
  classifyArtifact: vi.fn(() => 'generic'),
}))

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

describe('ArtifactViewer routes spec files to SpecRenderer', () => {
  it('renders SpecRenderer when classifyArtifact returns spec', async () => {
    const { classifyArtifact } = await import('../classify-artifact')
    vi.mocked(classifyArtifact).mockReturnValue('spec')

    const content = `---\ntype: spec\ntitle: "Auth"\nstatus: draft\n---\n\n# Auth`
    render(<ArtifactViewer file={makeFile({ name: 'auth-spec.md' })} content={content} />)

    expect(screen.getByTestId('spec-renderer')).toBeTruthy()
    expect(screen.getByTestId('spec-renderer').textContent).toBe('auth-spec.md')
  })
})

describe('ArtifactViewer routes adr files to AdrRenderer', () => {
  it('renders AdrRenderer when classifyArtifact returns adr', async () => {
    const { classifyArtifact } = await import('../classify-artifact')
    vi.mocked(classifyArtifact).mockReturnValue('adr')

    const content = `---\ntype: adr\ntitle: "Use SQLite"\nstatus: accepted\n---\n\n# ADR-001`
    render(<ArtifactViewer file={makeFile({ name: 'adr-001.md' })} content={content} />)

    expect(screen.getByTestId('adr-renderer')).toBeTruthy()
    expect(screen.getByTestId('adr-renderer').textContent).toBe('adr-001.md')
  })
})

describe('ArtifactViewer preserves existing fallback behavior', () => {
  it('renders image viewer for image files', async () => {
    const { classifyArtifact } = await import('../classify-artifact')
    vi.mocked(classifyArtifact).mockReturnValue('generic')

    const file = makeFile({ name: 'screenshot.png', type: 'image' })
    render(<ArtifactViewer file={file} content="" />)

    expect(screen.getByTestId('image-viewer')).toBeTruthy()
  })

  it('renders JSON viewer for .json files', async () => {
    const { classifyArtifact } = await import('../classify-artifact')
    vi.mocked(classifyArtifact).mockReturnValue('generic')

    const file = makeFile({ name: 'data.json', type: 'other' })
    render(<ArtifactViewer file={file} content='{"key": "value"}' />)

    expect(screen.getByText(/"key": "value"/)).toBeTruthy()
  })

  it('renders text viewer for unknown file types', async () => {
    const { classifyArtifact } = await import('../classify-artifact')
    vi.mocked(classifyArtifact).mockReturnValue('generic')

    const file = makeFile({ name: 'notes.txt', type: 'other' })
    render(<ArtifactViewer file={file} content="plain text content" />)

    expect(screen.getByText('plain text content')).toBeTruthy()
  })
})
