/* Pure-function coverage for the account helpers consumed by
 * <sui-account-menu> (chrome) and views/settings (settings card).
 * Both surfaces derive from these — drift here ripples to both. */
import { describe, expect, it } from 'vitest'
import type { AccountState } from '@ship/sui-runtime'
import {
  readIdentity,
  displayName,
  initials,
  planLabel,
} from '../src/account'

const loggedIn = (overrides: Record<string, unknown> = {}): AccountState => ({
  state: 'logged_in',
  user_id: 'user-test-1',
  me: {
    id: 'user-test-1',
    name: 'Ada Lovelace',
    email: 'ada@example.com',
    image: 'https://example.com/ada.png',
    ...(overrides.me as object | undefined),
  },
  ...overrides,
} as unknown as AccountState)

const pending: AccountState = {
  state: 'pending',
  state_id: 'state-abc',
} as unknown as AccountState

const loggedOut: AccountState = { state: 'logged_out' } as unknown as AccountState

describe('readIdentity', () => {
  it('returns logged_out for null/undefined state', () => {
    expect(readIdentity(null).status).toBe('logged_out')
    expect(readIdentity(undefined).status).toBe('logged_out')
  })

  it('flattens logged_in tag', () => {
    const id = readIdentity(loggedIn())
    expect(id.status).toBe('logged_in')
    expect(id.user_id).toBe('user-test-1')
    expect(id.me?.email).toBe('ada@example.com')
    expect(id.state_id).toBeNull()
  })

  it('flattens pending tag with state_id', () => {
    const id = readIdentity(pending)
    expect(id.status).toBe('pending')
    expect(id.state_id).toBe('state-abc')
    expect(id.me).toBeNull()
  })

  it('handles logged_in with absent me', () => {
    const id = readIdentity({
      state: 'logged_in',
      user_id: 'u',
      me: null,
    } as unknown as AccountState)
    expect(id.status).toBe('logged_in')
    expect(id.me).toBeNull()
  })
})

describe('displayName', () => {
  it('prefers name', () => {
    expect(displayName(readIdentity(loggedIn()))).toBe('Ada Lovelace')
  })

  it('falls back to email when name is empty', () => {
    expect(
      displayName(readIdentity(loggedIn({ me: { name: '', email: 'ada@example.com', id: 'u', image: '' } }))),
    ).toBe('ada@example.com')
  })

  it('falls back to user_id when name + email empty', () => {
    expect(
      displayName(readIdentity(loggedIn({ me: { name: '', email: '', id: 'inner', image: '' } }))),
    ).toBe('user-test-1') // outer user_id, not me.id
  })

  it('returns empty when not logged in', () => {
    expect(displayName(readIdentity(loggedOut))).toBe('')
    expect(displayName(readIdentity(pending))).toBe('')
  })
})

describe('initials', () => {
  it('takes first letters of two-word names', () => {
    expect(initials(readIdentity(loggedIn()))).toBe('AL')
  })

  it('falls back to two chars of single name', () => {
    expect(
      initials(readIdentity(loggedIn({ me: { name: 'Pi', email: '', id: 'u', image: '' } }))),
    ).toBe('PI')
  })

  it('splits email local-part when no name', () => {
    expect(
      initials(readIdentity(loggedIn({ me: { name: '', email: 'jane.doe@example.com', id: 'u', image: '' } }))),
    ).toBe('JD')
  })

  it('returns empty when not logged in', () => {
    expect(initials(readIdentity(loggedOut))).toBe('')
  })
})

describe('planLabel', () => {
  it('returns empty for every state until gateway surfaces a plan field', () => {
    // Today's contract — see source comment. When MeView gains `plan`,
    // this test changes alongside.
    expect(planLabel(readIdentity(loggedIn()))).toBe('')
    expect(planLabel(readIdentity(pending))).toBe('')
    expect(planLabel(readIdentity(loggedOut))).toBe('')
  })
})
