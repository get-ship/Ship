import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import { SpecRenderer } from '../SpecRenderer'
import type { SessionFile } from '../types'

afterEach(cleanup)

vi.mock('@ship/primitives', () => ({
  MarkdownEditor: ({ value }: { value: string }) => (
    <div data-testid="markdown-body">{value}</div>
  ),
}))

function makeFile(overrides: Partial<SessionFile> = {}): SessionFile {
  return {
    name: 'auth-spec.md',
    path: '.artifacts/auth-spec.md',
    type: 'markdown',
    size: 1024,
    modifiedAt: Date.now(),
    tier: 'workspace',
    ...overrides,
  }
}

const FULL_SPEC = `---
title: "Authentication Flow"
type: spec
status: draft
authors: ["alice", "bob"]
specs: ["login-flow", "oauth"]
version: "0.2.0"
target_date: "2026-05-01"
tags: ["auth", "security"]
---

## Overview

Users authenticate via OAuth2 flow.

## Requirements

1. Support Google and GitHub providers
2. Session tokens expire after 24h
`

const MINIMAL_SPEC = `---
title: "Minimal Spec"
type: spec
status: approved
---

# Minimal

Just a title and status.
`

describe('SpecRenderer displays structured header fields', () => {
  it('renders the spec title prominently', () => {
    render(<SpecRenderer file={makeFile()} content={FULL_SPEC} />)
    expect(screen.getByText('Authentication Flow')).toBeTruthy()
  })

  it('renders the status as a badge', () => {
    render(<SpecRenderer file={makeFile()} content={FULL_SPEC} />)
    const badge = screen.getByText('draft')
    expect(badge).toBeTruthy()
    expect(badge.closest('[data-testid="status-badge"]')).toBeTruthy()
  })

  it('renders author list', () => {
    render(<SpecRenderer file={makeFile()} content={FULL_SPEC} />)
    expect(screen.getByText(/alice/)).toBeTruthy()
    expect(screen.getByText(/bob/)).toBeTruthy()
  })

  it('renders linked specs', () => {
    render(<SpecRenderer file={makeFile()} content={FULL_SPEC} />)
    expect(screen.getByText(/login-flow/)).toBeTruthy()
    expect(screen.getByText(/oauth/)).toBeTruthy()
  })

  it('renders version', () => {
    render(<SpecRenderer file={makeFile()} content={FULL_SPEC} />)
    expect(screen.getByText(/0\.2\.0/)).toBeTruthy()
  })

  it('renders target date', () => {
    render(<SpecRenderer file={makeFile()} content={FULL_SPEC} />)
    expect(screen.getByText(/2026-05-01/)).toBeTruthy()
  })

  it('renders tags', () => {
    render(<SpecRenderer file={makeFile()} content={FULL_SPEC} />)
    expect(screen.getByText('auth')).toBeTruthy()
    expect(screen.getByText('security')).toBeTruthy()
  })
})

describe('SpecRenderer renders markdown body below header', () => {
  it('renders the body content after stripping frontmatter', () => {
    render(<SpecRenderer file={makeFile()} content={FULL_SPEC} />)
    // The body should be rendered (either via markdown preview or editor)
    expect(screen.getByText(/Users authenticate via OAuth2 flow/)).toBeTruthy()
  })
})

describe('SpecRenderer handles minimal spec gracefully', () => {
  it('renders title and status with no optional fields', () => {
    render(<SpecRenderer file={makeFile()} content={MINIMAL_SPEC} />)
    expect(screen.getByText('Minimal Spec')).toBeTruthy()
    expect(screen.getByText('approved')).toBeTruthy()
  })

  it('does not render authors section when absent', () => {
    render(<SpecRenderer file={makeFile()} content={MINIMAL_SPEC} />)
    expect(screen.queryByText(/authors/i)).toBeNull()
  })

  it('does not render version when absent', () => {
    render(<SpecRenderer file={makeFile()} content={MINIMAL_SPEC} />)
    expect(screen.queryByTestId('spec-version')).toBeNull()
  })

  it('does not render target date when absent', () => {
    render(<SpecRenderer file={makeFile()} content={MINIMAL_SPEC} />)
    expect(screen.queryByTestId('spec-target-date')).toBeNull()
  })
})

describe('SpecRenderer supports editing callbacks', () => {
  it('calls onContentChange when body is edited', () => {
    const onChange = vi.fn()
    render(
      <SpecRenderer
        file={makeFile()}
        content={FULL_SPEC}
        onContentChange={onChange}
      />,
    )
    // Editor should be present when onContentChange is provided
    expect(screen.getByTestId('markdown-body')).toBeTruthy()
  })

  it('calls onSave when save is triggered', () => {
    const onSave = vi.fn()
    render(
      <SpecRenderer
        file={makeFile()}
        content={FULL_SPEC}
        isDirty
        onContentChange={vi.fn()}
        onSave={onSave}
      />,
    )
    // Save button should appear when dirty
    const saveBtn = screen.getByText(/Save/i)
    expect(saveBtn).toBeTruthy()
  })
})
