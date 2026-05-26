import { afterEach, describe, expect, it } from 'vitest'
import { hostContext, setHostContext } from '../src/host-context'

afterEach(() => {
  ;(globalThis as { __shipHostContext?: unknown }).__shipHostContext = undefined
})

describe('HostContext seam', () => {
  it('returns an empty object when no host registered one', () => {
    expect(hostContext()).toEqual({})
  })

  it('reads back what the host registered', () => {
    setHostContext({ projectId: 'proj-1', sessionId: 'sess-1' })
    expect(hostContext()).toEqual({ projectId: 'proj-1', sessionId: 'sess-1' })
  })

  it('merges over previous values (later set wins per key)', () => {
    setHostContext({ projectId: 'proj-1', sessionId: 'sess-1' })
    setHostContext({ projectId: 'proj-2' })
    expect(hostContext()).toEqual({ projectId: 'proj-2', sessionId: 'sess-1' })
  })
})
