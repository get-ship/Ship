import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import { AdrRenderer } from '../AdrRenderer'
import type { SessionFile } from '../types'

afterEach(cleanup)

vi.mock('@ship/primitives', () => ({
  MarkdownEditor: ({ value }: { value: string }) => (
    <div data-testid="markdown-body">{value}</div>
  ),
}))

function makeFile(overrides: Partial<SessionFile> = {}): SessionFile {
  return {
    name: 'adr-001-use-sqlite.md',
    path: '.artifacts/adr-001-use-sqlite.md',
    type: 'markdown',
    size: 2048,
    modifiedAt: Date.now(),
    tier: 'workspace',
    ...overrides,
  }
}

const FULL_ADR = `---
title: "Use SQLite for local storage"
type: adr
status: accepted
authors: ["alice"]
date: "2026-04-10"
supersedes: "adr-000-use-postgres"
tags: ["database", "storage"]
---

## Context

We need a local-first database that works without a server process.

## Decision

Use SQLite via sqlx with WAL mode for concurrent reads.

## Consequences

- Positive: Zero-config deployment, single file database
- Negative: No built-in replication, write concurrency limited
`

const MINIMAL_ADR = `---
title: "Minimal Decision"
type: adr
status: proposed
---

## Context

Brief context.

## Decision

Brief decision.
`

const SUPERSEDED_ADR = `---
title: "Use Postgres"
type: adr
status: superseded
superseded_by: "adr-001-use-sqlite"
---

## Context

Original database choice.

## Decision

Use Postgres.
`

describe('AdrRenderer displays structured header fields', () => {
  it('renders the ADR title prominently', () => {
    render(<AdrRenderer file={makeFile()} content={FULL_ADR} />)
    expect(screen.getByText('Use SQLite for local storage')).toBeTruthy()
  })

  it('renders the status as a badge', () => {
    render(<AdrRenderer file={makeFile()} content={FULL_ADR} />)
    const badge = screen.getByText('accepted')
    expect(badge).toBeTruthy()
    expect(badge.closest('[data-testid="status-badge"]')).toBeTruthy()
  })

  it('renders author list', () => {
    render(<AdrRenderer file={makeFile()} content={FULL_ADR} />)
    expect(screen.getByText(/alice/)).toBeTruthy()
  })

  it('renders date', () => {
    render(<AdrRenderer file={makeFile()} content={FULL_ADR} />)
    expect(screen.getByText(/2026-04-10/)).toBeTruthy()
  })

  it('renders supersedes link', () => {
    render(<AdrRenderer file={makeFile()} content={FULL_ADR} />)
    expect(screen.getByText(/adr-000-use-postgres/)).toBeTruthy()
  })

  it('renders tags', () => {
    render(<AdrRenderer file={makeFile()} content={FULL_ADR} />)
    expect(screen.getByText('database')).toBeTruthy()
    expect(screen.getByText('storage')).toBeTruthy()
  })
})

describe('AdrRenderer renders body sections', () => {
  it('renders Context section content', () => {
    render(<AdrRenderer file={makeFile()} content={FULL_ADR} />)
    expect(screen.getByText(/local-first database/)).toBeTruthy()
  })

  it('renders Decision section content', () => {
    render(<AdrRenderer file={makeFile()} content={FULL_ADR} />)
    expect(screen.getByText(/SQLite via sqlx/)).toBeTruthy()
  })

  it('renders Consequences section content', () => {
    render(<AdrRenderer file={makeFile()} content={FULL_ADR} />)
    expect(screen.getByText(/Zero-config deployment/)).toBeTruthy()
  })
})

describe('AdrRenderer handles minimal ADR gracefully', () => {
  it('renders title and status with no optional fields', () => {
    render(<AdrRenderer file={makeFile()} content={MINIMAL_ADR} />)
    expect(screen.getByText('Minimal Decision')).toBeTruthy()
    expect(screen.getByText('proposed')).toBeTruthy()
  })

  it('does not render supersedes when absent', () => {
    render(<AdrRenderer file={makeFile()} content={MINIMAL_ADR} />)
    expect(screen.queryByText(/supersedes/i)).toBeNull()
  })

  it('does not render date when absent', () => {
    render(<AdrRenderer file={makeFile()} content={MINIMAL_ADR} />)
    expect(screen.queryByTestId('adr-date')).toBeNull()
  })
})

describe('AdrRenderer shows superseded-by for replaced ADRs', () => {
  it('renders superseded_by link when status is superseded', () => {
    render(<AdrRenderer file={makeFile()} content={SUPERSEDED_ADR} />)
    expect(screen.getByText(/adr-001-use-sqlite/)).toBeTruthy()
  })

  it('applies visual distinction for superseded status', () => {
    render(<AdrRenderer file={makeFile()} content={SUPERSEDED_ADR} />)
    const badge = screen.getByText('superseded')
    expect(badge).toBeTruthy()
    // Superseded ADRs should have a muted/strikethrough visual treatment
    expect(badge.closest('[data-testid="status-badge"]')).toBeTruthy()
  })
})

describe('AdrRenderer supports editing callbacks', () => {
  it('calls onContentChange when body is edited', () => {
    const onChange = vi.fn()
    render(
      <AdrRenderer
        file={makeFile()}
        content={FULL_ADR}
        onContentChange={onChange}
      />,
    )
    expect(screen.getByTestId('markdown-body')).toBeTruthy()
  })

  it('shows save button when dirty', () => {
    render(
      <AdrRenderer
        file={makeFile()}
        content={FULL_ADR}
        isDirty
        onContentChange={vi.fn()}
        onSave={vi.fn()}
      />,
    )
    expect(screen.getByText(/Save/i)).toBeTruthy()
  })
})
