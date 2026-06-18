// Behavioral spec for <ship-space-add-dialog> — the SHELL register-a-directory
// surface. Submitting emits ship-add{path} (trimmed, non-empty); cancel emits
// ship-cancel. Failure is fed back via error + tone props (no daemon call here).
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import controller from '../src/ship-space-add-dialog/ship-space-add-dialog.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-space-add-dialog/ship-space-add-dialog.sui'),
  'utf8',
)
const TAG = 'ship-space-add-dialog'

function mount(attrs: Record<string, string | boolean> = {}) {
  return renderComponent(SRC, TAG, { controller, attrs: { open: true, ...attrs } })
}
function setInput(c: { el: HTMLElement }, value: string) {
  const input = c.el.querySelector<HTMLInputElement>('.input')!
  input.value = value
}

describe('<ship-space-add-dialog>', () => {
  it('emits ship-add{path} with the trimmed input on submit', () => {
    const c = mount()
    setInput(c, '  ~/dev/new-space  ')
    let detail: unknown = null
    c.el.addEventListener('ship-add', (e) => { detail = (e as CustomEvent).detail })
    c.el.querySelector('[data-action="submit"]')!.dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ path: '~/dev/new-space' })
    c.unmount()
  })

  it('does not emit ship-add for an empty path', () => {
    const c = mount()
    setInput(c, '   ')
    let fired = false
    c.el.addEventListener('ship-add', () => { fired = true })
    c.el.querySelector('[data-action="submit"]')!.dispatchEvent(new Event('click', { bubbles: true }))
    expect(fired).toBe(false)
    c.unmount()
  })

  it('emits ship-cancel when Cancel is pressed', () => {
    const c = mount()
    let fired = false
    c.el.addEventListener('ship-cancel', () => { fired = true })
    c.el.querySelector('[data-action="cancel"]')!.dispatchEvent(new Event('click', { bubbles: true }))
    expect(fired).toBe(true)
    c.unmount()
  })

  it('renders the error banner with its tone when a failure is fed back', () => {
    const c = mount({ error: 'Not a directory', tone: 'destructive' })
    const err = c.el.querySelector('[part="error"]')
    expect(err).not.toBeNull()
    expect(err!.getAttribute('data-tone')).toBe('destructive')
    expect(err!.textContent).toContain('Not a directory')
    c.unmount()
  })

  it('shows no error banner when there is no error', () => {
    const c = mount()
    expect(c.el.querySelector('[part="error"]')).toBeNull()
    c.unmount()
  })
})
