import { describe, it, expect } from 'vitest'
import { deleteAccountEmail } from '../src/lib/email.js'

// The deleteAccountEmail template is the body of the Better Auth
// sendDeleteAccountVerification hook — the one piece of the deletion flow
// the user sees. Lock its shape so a future template tweak can't drop the
// link, the recipient, or the "irreversible" wording.

describe('deleteAccountEmail template', () => {
  const args = {
    to: 'someone@example.com',
    url: 'https://api.getship.dev/v1/auth/delete-user/callback?token=abc123&callbackURL=%2F',
  }
  const out = deleteAccountEmail(args)

  it('addresses the recipient and uses the deletion subject', () => {
    expect(out.to).toBe(args.to)
    expect(out.subject).toMatch(/delete|deletion/i)
  })

  it('embeds the callback URL in both text and html bodies', () => {
    expect(out.text).toContain(args.url)
    expect(out.html).toContain(args.url)
  })

  it('warns the user the action is irreversible', () => {
    expect(out.text.toLowerCase()).toMatch(/irreversible|cannot be undone|expires/)
    expect(out.html.toLowerCase()).toMatch(/irreversible|cannot be undone/)
  })
})
