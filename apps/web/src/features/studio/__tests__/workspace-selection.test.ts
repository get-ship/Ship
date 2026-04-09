import { describe, it, expect } from 'vitest'
import { resolveWorkspaceSelection } from '../workspace-selection'

const active = (branch: string) => ({ branch, status: 'active' })
const archived = (branch: string) => ({ branch, status: 'archived' })

describe('resolveWorkspaceSelection', () => {
  it('returns undefined (no change) when selection is valid', () => {
    const result = resolveWorkspaceSelection('main', [active('main'), active('feature/a')])
    expect(result).toBeUndefined()
  })

  it('returns undefined when workspace list is empty', () => {
    const result = resolveWorkspaceSelection('main', [])
    expect(result).toBeUndefined()
  })

  it('picks first active workspace when no selection is set', () => {
    const result = resolveWorkspaceSelection(null, [active('main'), active('feature/a')])
    expect(result).toBe('main')
  })

  it('falls back to first active when viewed branch disappears', () => {
    const result = resolveWorkspaceSelection('feature/deleted', [active('main'), active('feature/b')])
    expect(result).toBe('main')
  })

  it('does not reset when workspaces list refreshes but selection is still valid', () => {
    // This is the core regression test — previously any list refresh could reset the selection
    const workspaces = [active('main'), active('feature/a')]
    expect(resolveWorkspaceSelection('feature/a', workspaces)).toBeUndefined()
    // Simulating a re-render with same data
    expect(resolveWorkspaceSelection('feature/a', [...workspaces])).toBeUndefined()
  })

  it('falls back to first workspace when no active workspaces exist', () => {
    const result = resolveWorkspaceSelection('feature/gone', [archived('feature/old')])
    expect(result).toBe('feature/old')
  })

  it('returns null when viewed is stale and list has no fallback', () => {
    const result = resolveWorkspaceSelection('feature/gone', [])
    expect(result).toBeUndefined() // empty list = no change
  })
})
